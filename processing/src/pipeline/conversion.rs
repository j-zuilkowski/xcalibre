use crate::db::{conversion_queries, extended_queries, format_queries};
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use crate::utils::hash::sha256_file;
use sqlx::SqlitePool;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionOutcome {
    pub job_id:       String,
    pub book_id:      String,
    pub output_path:  String,
    pub target_format:String,
    pub mode:         String,
}

pub async fn convert_book_to_epub(
    pool: &SqlitePool,
    book_id: &str,
    tweak: bool,
) -> Result<ConversionOutcome, ProcessingError> {
    let details = extended_queries::get_book_details(pool, book_id)
        .await?
        .ok_or_else(|| ProcessingError::MetadataError("book not found".into()))?;

    let source_path = resolve_source_path(pool, book_id, details.local_path.as_deref()).await?;
    let output_path = build_output_path(&source_path, &details.title, tweak);
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(ProcessingError::IoError)?;
    }

    let job_id = conversion_queries::create_job(
        pool,
        book_id,
        &details.format,
        "EPUB",
        if tweak { "TWEAK" } else { "CONVERT" },
        &output_path.to_string_lossy(),
    )
    .await?;

    conversion_queries::update_status(pool, &job_id, "RUNNING", None).await?;

    let extracted = extract_text_for_format(&details.format, &source_path)?;
    let metadata_view: BookMetadataView = details.into();
    let epub_bytes = build_epub_package(&metadata_view, &extracted)?;
    let output_path_for_io = output_path.clone();
    let (sha256, file_size) = tokio::task::spawn_blocking(move || -> Result<(String, i64), ProcessingError> {
        std::fs::write(&output_path_for_io, epub_bytes).map_err(ProcessingError::IoError)?;
        let sha256 = sha256_file(&output_path_for_io)?;
        let file_size = std::fs::metadata(&output_path_for_io)
            .map_err(ProcessingError::IoError)?
            .len() as i64;
        Ok((sha256, file_size))
    })
    .await
    .map_err(|_| ProcessingError::IoError(std::io::Error::other("blocking conversion task failed")))??;
    format_queries::insert_book_format(
        pool,
        book_id,
        "EPUB",
        &output_path.to_string_lossy(),
        &sha256,
        file_size,
    )
    .await?;

    conversion_queries::update_status(pool, &job_id, "COMPLETED", None).await?;

    Ok(ConversionOutcome {
        job_id,
        book_id: book_id.to_string(),
        output_path: output_path.to_string_lossy().into_owned(),
        target_format: "EPUB".to_string(),
        mode: if tweak { "TWEAK".to_string() } else { "CONVERT".to_string() },
    })
}

async fn resolve_source_path(
    pool: &SqlitePool,
    book_id: &str,
    local_path: Option<&str>,
) -> Result<PathBuf, ProcessingError> {
    if let Some(path) = local_path {
        let candidate = PathBuf::from(path);
        if tokio::fs::try_exists(&candidate)
            .await
            .map_err(ProcessingError::IoError)?
        {
            return Ok(candidate);
        }
    }

    let formats = format_queries::get_formats_for_book(pool, book_id).await?;
    for format in formats {
        let candidate = PathBuf::from(format.file_path);
        if tokio::fs::try_exists(&candidate)
            .await
            .map_err(ProcessingError::IoError)?
        {
            return Ok(candidate);
        }
    }

    Err(ProcessingError::MetadataError(
        "book has no usable source path".to_string(),
    ))
}

fn build_output_path(source_path: &Path, title: &str, tweak: bool) -> PathBuf {
    let parent = source_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(title);
    let suffix = if tweak { "tweaked" } else { "converted" };
    parent.join(format!("{}-{}.epub", sanitize_component(stem), suffix))
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn extract_text_for_format(
    format: &str,
    path: &Path,
) -> Result<ExtractedText, ProcessingError> {
    match format.to_uppercase().as_str() {
        "EPUB" => crate::text::epub::extract(path),
        "PDF" => crate::text::pdf::extract(path),
        "MOBI" | "AZW3" => crate::text::mobi::extract(path),
        "AZW4" => crate::text::azw4::extract(path),
        "FB2" => crate::text::fb2::extract(path),
        "HTML" | "HTMLZ" => crate::text::html::extract(path),
        "RTF" => crate::text::rtf::extract(path),
        "DOCX" => crate::text::docx::extract(path),
        "ODT" => crate::text::odt::extract(path),
        "CHM" => crate::text::chm::extract(path),
        "LRF" | "LRX" => crate::text::lrf::extract(path),
        "PDB" | "PML" | "RB" => crate::text::pdb::extract(path),
        "SNB" => crate::text::snb::extract(path),
        "TCR" => crate::text::tcr::extract(path),
        "DJVU" => crate::text::djvu::extract(path),
        "LIT" => crate::text::lit::extract(path),
        "CBZ" | "CBR" | "TXT" => Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        }),
        _ => Ok(ExtractedText {
            full_text: std::fs::read_to_string(path).unwrap_or_default(),
            word_count: 0,
        }),
    }
}

fn build_epub_package(
    details: &BookMetadataView,
    extracted: &ExtractedText,
) -> Result<Vec<u8>, ProcessingError> {
    let buffer = {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);

        writer
            .start_file(
                "mimetype",
                FileOptions::default().compression_method(zip::CompressionMethod::Stored),
            )
            .map_err(|e| ProcessingError::IoError(std::io::Error::other(e)))?;
        writer
            .write_all(b"application/epub+zip")
            .map_err(ProcessingError::IoError)?;

        writer
            .start_file("META-INF/container.xml", FileOptions::default())
            .map_err(|e| ProcessingError::IoError(std::io::Error::other(e)))?;
        writer
            .write_all(container_xml().as_bytes())
            .map_err(ProcessingError::IoError)?;

        writer
            .start_file("OEBPS/content.opf", FileOptions::default())
            .map_err(|e| ProcessingError::IoError(std::io::Error::other(e)))?;
        writer
            .write_all(build_opf(details).as_bytes())
            .map_err(ProcessingError::IoError)?;

        writer
            .start_file("OEBPS/text/chapter1.xhtml", FileOptions::default())
            .map_err(|e| ProcessingError::IoError(std::io::Error::other(e)))?;
        writer
            .write_all(build_xhtml(details, extracted).as_bytes())
            .map_err(ProcessingError::IoError)?;

        writer
            .finish()
            .map_err(|e| ProcessingError::IoError(std::io::Error::other(e)))?
            .into_inner()
    };

    Ok(buffer)
}

#[derive(Debug, Clone)]
struct BookMetadataView {
    id:           String,
    title:        String,
    authors:      Vec<String>,
    description:  Option<String>,
    publisher:    Option<String>,
    pubdate:      Option<String>,
    series_name:  Option<String>,
    series_index: Option<f64>,
    tags:         Vec<String>,
    identifiers:  Vec<crate::db::extended_queries::BookIdentifier>,
}

impl From<crate::db::extended_queries::BookDetails> for BookMetadataView {
    fn from(value: crate::db::extended_queries::BookDetails) -> Self {
        Self {
            id: value.id,
            title: value.title,
            authors: value.authors,
            description: value.description,
            publisher: value.publisher,
            pubdate: value.pubdate,
            series_name: value.series_name,
            series_index: value.series_index,
            tags: value.tags,
            identifiers: value.identifiers,
        }
    }
}

fn build_opf(details: &BookMetadataView) -> String {
    let title = escape_xml(&details.title);
    let authors = if details.authors.is_empty() {
        vec!["Unknown Author".to_string()]
    } else {
        details.authors.clone()
    };
    let mut metadata = String::new();
    metadata.push_str(&format!("<dc:title>{}</dc:title>", title));
    for author in &authors {
        metadata.push_str(&format!(
            "<dc:creator>{}</dc:creator>",
            escape_xml(author)
        ));
    }
    if let Some(description) = &details.description {
        metadata.push_str(&format!(
            "<dc:description>{}</dc:description>",
            escape_xml(description)
        ));
    }
    if let Some(publisher) = &details.publisher {
        metadata.push_str(&format!(
            "<dc:publisher>{}</dc:publisher>",
            escape_xml(publisher)
        ));
    }
    if let Some(pubdate) = &details.pubdate {
        metadata.push_str(&format!("<dc:date>{}</dc:date>", escape_xml(pubdate)));
    }
    for tag in &details.tags {
        metadata.push_str(&format!("<dc:subject>{}</dc:subject>", escape_xml(tag)));
    }
    metadata.push_str(&format!(
        "<dc:identifier id=\"bookid\">urn:xcalibre:{}</dc:identifier>",
        escape_xml(&details.id)
    ));
    for identifier in &details.identifiers {
        metadata.push_str(&format!(
            "<dc:identifier opf:scheme=\"{}\">{}</dc:identifier>",
            escape_xml(&identifier.id_type),
            escape_xml(&identifier.value)
        ));
    }
    if let Some(series_name) = &details.series_name {
        metadata.push_str(&format!(
            "<meta name=\"calibre:series\" content=\"{}\"/>",
            escape_xml(series_name)
        ));
    }
    if let Some(series_index) = details.series_index {
        metadata.push_str(&format!(
            "<meta name=\"calibre:series_index\" content=\"{}\"/>",
            series_index
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="bookid" version="3.0" xml:lang="en">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    {}
  </metadata>
  <manifest>
    <item id="chapter1" href="text/chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chapter1"/>
  </spine>
</package>"#,
        metadata
    )
}

fn build_xhtml(details: &BookMetadataView, extracted: &ExtractedText) -> String {
    let title = escape_xml(&details.title);
    let body = if extracted.full_text.trim().is_empty() {
        format!(
            "<p>{}</p>",
            escape_xml("No extractable text was available for this source.")
        )
    } else {
        extracted
            .full_text
            .split("\n\n")
            .filter_map(|paragraph| {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(format!("<p>{}</p>", escape_xml(trimmed)))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head>
    <title>{}</title>
    <meta charset="utf-8" />
    <style>
      body {{ font-family: serif; line-height: 1.5; margin: 2rem auto; max-width: 42rem; padding: 0 1rem; }}
      h1 {{ font-size: 1.6rem; margin-bottom: 1rem; }}
      p {{ margin: 0 0 1rem 0; }}
    </style>
  </head>
  <body>
    <h1>{}</h1>
    {}
  </body>
</html>"#,
        title, title, body
    )
}

fn container_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
