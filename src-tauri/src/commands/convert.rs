// Conversion Tauri command.
use tauri::AppHandle;
use xcalibre_processing::convert::{docx, html, txt};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OutputFormat {
    Txt,
    Html,
    Docx,
}

#[tauri::command]
pub async fn convert_book(
    epub_path: String,
    output_format: OutputFormat,
    output_dir: Option<String>,
    app: AppHandle,
) -> Result<String, String> {
    let epub = std::path::PathBuf::from(&epub_path);
    if !epub.exists() {
        return Err(format!("source file not found: {epub_path}"));
    }

    let stem = epub.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let dir = output_dir
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            app.path().download_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
        });

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let out_path = match output_format {
        OutputFormat::Txt  => dir.join(format!("{stem}.txt")),
        OutputFormat::Html => dir.join(format!("{stem}.html")),
        OutputFormat::Docx => dir.join(format!("{stem}.docx")),
    };

    match output_format {
        OutputFormat::Txt  => txt::epub_to_txt(&epub, &out_path).map_err(|e| e.to_string())?,
        OutputFormat::Html => html::epub_to_html(&epub, &out_path).map_err(|e| e.to_string())?,
        OutputFormat::Docx => docx::epub_to_docx(&epub, &out_path).map_err(|e| e.to_string())?,
    }

    Ok(out_path.to_string_lossy().to_string())
}
