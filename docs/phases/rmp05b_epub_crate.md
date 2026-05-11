# RMP-05b — xcalibre-epub Manipulation Library (Green: Implementation)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> Prerequisite: rmp05a complete (crate scaffold + all failing tests committed).
> TDD role: GREEN — implement each module until its tests pass.
> NOTE: This phase is 6–10 weeks of work. Implement one module at a time
> in the order below; each task's milestone check must be green before moving on.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R05b-T01 | Container: open, manifest, spine, read_item | ⬜ |
| R05b-T02 | Container: write_item, remove_item, save, save_as | ⬜ |
| R05b-T03 | EpubOPF: parse (2.x + 3.x) + to_xml() | ⬜ |
| R05b-T04 | cover: extract_cover + replace_cover | ⬜ |
| R05b-T05 | font: list_fonts + embed_font | ⬜ |
| R05b-T06 | image_opt: rescale + optimize images in container | ⬜ |
| R05b-T07 | css: parse + rewrite CSS in container | ⬜ |
| R05b-T08 | split: split container at heading boundary | ⬜ |
| R05b-T09 | validation: link, CSS, font, image, OPF checks | ⬜ |
| R05b-T10 | upgrade: EPUB 2→3 (NCX → nav, version attr) | ⬜ |
| R05b-T11 | Milestone check + integration smoke test | ⬜ |

---

## R05b-T01

Replace the stub `Container::open`, `manifest_items`, `spine_hrefs`, `opf_path`,
and `read_item` in `xcalibre-epub/src/container.rs` with a real implementation.

The `Container` struct holds an in-memory map of all ZIP entries so that
`read_item` and `write_item` operate on the in-memory copy; `save` rewrites
the ZIP on disk.

```rust
use crate::EpubError;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// In-memory EPUB container.
pub struct Container {
    /// Path to the on-disk ZIP file.
    path: PathBuf,
    /// All entries: zip entry name → raw bytes.
    entries: HashMap<String, Vec<u8>>,
    /// OPF file path within the ZIP (from META-INF/container.xml).
    opf_path: String,
    /// Manifest: id → (href, media_type)
    manifest: Vec<(String, String, String)>,
    /// Spine hrefs in reading order.
    spine: Vec<String>,
}

impl Container {
    pub fn open(path: &Path) -> Result<Self, EpubError> {
        let file = std::fs::File::open(path).map_err(EpubError::Io)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| EpubError::Zip(e.to_string()))?;

        let mut entries: HashMap<String, Vec<u8>> = HashMap::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| EpubError::Zip(e.to_string()))?;
            if entry.is_dir() { continue; }
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(EpubError::Io)?;
            entries.insert(name, data);
        }

        let opf_path = {
            let container_xml = entries.get("META-INF/container.xml")
                .ok_or_else(|| EpubError::MissingElement("META-INF/container.xml".into()))?;
            let xml = std::str::from_utf8(container_xml)
                .map_err(|e| EpubError::Xml(e.to_string()))?;
            let doc = roxmltree::Document::parse(xml)
                .map_err(|e| EpubError::Xml(e.to_string()))?;
            doc.descendants()
                .find(|n| n.tag_name().name() == "rootfile")
                .and_then(|n| n.attribute("full-path"))
                .map(String::from)
                .ok_or_else(|| EpubError::MissingElement("rootfile full-path".into()))?
        };

        let (manifest, spine) = parse_opf_for_manifest(&entries, &opf_path)?;

        Ok(Self { path: path.to_path_buf(), entries, opf_path, manifest, spine })
    }

    pub fn manifest_items(&self) -> Vec<(String, String, String)> {
        self.manifest.clone()
    }

    pub fn opf_path(&self) -> &str { &self.opf_path }

    pub fn spine_hrefs(&self) -> Vec<String> { self.spine.clone() }

    pub fn read_item(&self, href: &str) -> Result<Vec<u8>, EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.get(&full)
            .or_else(|| self.entries.get(href))
            .cloned()
            .ok_or_else(|| EpubError::ItemNotFound(href.to_string()))
    }

    pub fn write_item(&mut self, href: &str, data: &[u8], _media_type: &str) -> Result<(), EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.insert(full, data.to_vec());
        Ok(())
    }

    pub fn remove_item(&mut self, href: &str) -> Result<(), EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.remove(&full)
            .or_else(|| self.entries.remove(href))
            .map(|_| ())
            .ok_or_else(|| EpubError::ItemNotFound(href.to_string()))
    }

    pub fn save(&mut self) -> Result<(), EpubError> {
        self.write_zip(&self.path.clone())
    }

    pub fn save_as(&self, path: &Path) -> Result<(), EpubError> {
        self.write_zip(path)
    }

    fn write_zip(&self, path: &Path) -> Result<(), EpubError> {
        let tmp = path.with_extension("epub.tmp");
        {
            let f = std::fs::File::create(&tmp).map_err(EpubError::Io)?;
            let mut z = zip::ZipWriter::new(f);

            // mimetype MUST be first and stored (uncompressed)
            let stored = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            let deflated = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            if let Some(mt) = self.entries.get("mimetype") {
                z.start_file("mimetype", stored).map_err(|e| EpubError::Zip(e.to_string()))?;
                use std::io::Write;
                z.write_all(mt).map_err(EpubError::Io)?;
            }
            for (name, data) in &self.entries {
                if name == "mimetype" { continue; }
                let opts = if name.ends_with(".xhtml") || name.ends_with(".html")
                    || name.ends_with(".css") || name.ends_with(".opf")
                    || name.ends_with(".xml") || name.ends_with(".ncx")
                { deflated } else { deflated };
                z.start_file(name, opts).map_err(|e| EpubError::Zip(e.to_string()))?;
                use std::io::Write;
                z.write_all(data).map_err(EpubError::Io)?;
            }
            z.finish().map_err(|e| EpubError::Zip(e.to_string()))?;
        }
        std::fs::rename(&tmp, path).map_err(EpubError::Io)
    }
}

fn resolve_href(opf_path: &str, href: &str) -> String {
    if href.starts_with('/') || href.contains("://") { return href.to_string(); }
    let opf_dir = Path::new(opf_path).parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    if opf_dir.is_empty() { href.to_string() } else { format!("{}/{}", opf_dir, href) }
}

fn parse_opf_for_manifest(
    entries: &HashMap<String, Vec<u8>>, opf_path: &str,
) -> Result<(Vec<(String, String, String)>, Vec<String>), EpubError> {
    let opf_data = entries.get(opf_path)
        .ok_or_else(|| EpubError::MissingElement(format!("OPF file: {}", opf_path)))?;
    let xml = std::str::from_utf8(opf_data).map_err(|e| EpubError::Xml(e.to_string()))?;
    let doc = roxmltree::Document::parse(xml).map_err(|e| EpubError::Xml(e.to_string()))?;

    let manifest: Vec<(String, String, String)> = doc.descendants()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| {
            let id = n.attribute("id")?.to_string();
            let href = n.attribute("href")?.to_string();
            let mt = n.attribute("media-type").unwrap_or("application/octet-stream").to_string();
            Some((id, href, mt))
        })
        .collect();

    let id_to_href: HashMap<_, _> = manifest.iter()
        .map(|(id, href, _)| (id.clone(), href.clone())).collect();

    let spine: Vec<String> = doc.descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| {
            let idref = n.attribute("idref")?;
            id_to_href.get(idref).cloned()
        })
        .collect();

    Ok((manifest, spine))
}
```

Then run:
```bash
cargo test -p xcalibre-epub -- test_container
git add xcalibre-epub/src/container.rs
git commit -m "R05b-T01: Container open/read/spine — container read tests green"
```

---

## R05b-T02

The `write_item`, `remove_item`, `save`, and `save_as` methods were included in T01.
Run the write round-trip test now:

```bash
cargo test -p xcalibre-epub -- test_container
```

All container tests should be green. If any fail, fix them before proceeding.

```bash
git add xcalibre-epub/src/container.rs
git commit -m "R05b-T02: Container write/save — all container tests green"
```

---

## R05b-T03

Replace the stub `EpubOPF::parse` and `to_xml` in `xcalibre-epub/src/opf.rs`
with real implementations using `roxmltree` for parsing and `quick-xml` for writing.

Key implementation notes:
- Parse `<package version="...">` for epub_version
- Parse `dc:title`, `dc:creator`, `dc:language`, `dc:publisher` from `<metadata>`
- Parse `<meta name="calibre:series" content="..."/>` for series
- Parse `<meta name="calibre:series_index" content="..."/>` for series_index
- Parse `<item>` elements for manifest, `<itemref>` for spine
- `to_xml()` must produce valid OPF 2.x XML that round-trips cleanly

Then run:
```bash
cargo test -p xcalibre-epub -- test_opf
git add xcalibre-epub/src/opf.rs
git commit -m "R05b-T03: EpubOPF parse/serialize — OPF tests green"
```

---

## R05b-T04

Implement `cover::extract_cover` and `cover::replace_cover` in `xcalibre-epub/src/cover.rs`.

- `extract_cover`: parse OPF manifest for an item with `properties="cover-image"` or `<meta name="cover">`, return its bytes
- `replace_cover`: write the JPEG bytes to the cover href, update OPF if needed

```bash
cargo test -p xcalibre-epub -- test_cover
git add xcalibre-epub/src/cover.rs
git commit -m "R05b-T04: cover extract/replace — cover tests green"
```

---

## R05b-T05

Implement `font::list_fonts` in `xcalibre-epub/src/font.rs`:
- Scan manifest for items with media-type `font/ttf`, `font/otf`,
  `application/font-sfnt`, `application/vnd.ms-opentype`, `font/woff`, `font/woff2`

Implement `font::embed_font`:
- Write font bytes to container at `Fonts/<name>`
- Add manifest item if not already present

```bash
cargo test -p xcalibre-epub -- test_font
git add xcalibre-epub/src/font.rs
git commit -m "R05b-T05: font list/embed — font tests green"
```

---

## R05b-T06

Implement `xcalibre-epub/src/image_opt.rs`:
```rust
use crate::{Container, EpubError};
use image::imageops::FilterType;

pub struct ImageOptConfig { pub max_width: u32, pub max_height: u32, pub jpeg_quality: u8 }
impl Default for ImageOptConfig {
    fn default() -> Self { Self { max_width: 1920, max_height: 1080, jpeg_quality: 85 } }
}

/// Rescale all images in the container exceeding max_width × max_height.
pub fn optimize_images(container: &mut Container, cfg: &ImageOptConfig) -> Result<u32, EpubError> {
    let image_hrefs: Vec<String> = container.manifest_items().into_iter()
        .filter(|(_, _, mt)| mt.starts_with("image/"))
        .map(|(_, href, _)| href)
        .collect();
    let mut count = 0u32;
    for href in image_hrefs {
        if let Ok(data) = container.read_item(&href) {
            if let Ok(img) = image::load_from_memory(&data) {
                if img.width() > cfg.max_width || img.height() > cfg.max_height {
                    let resized = img.resize(cfg.max_width, cfg.max_height, FilterType::Lanczos3);
                    let mut buf = std::io::Cursor::new(Vec::new());
                    resized.write_to(&mut buf, image::ImageFormat::Jpeg)
                        .map_err(|e| EpubError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                    container.write_item(&href, &buf.into_inner(), "image/jpeg")?;
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}
```

```bash
cargo build -p xcalibre-epub
cargo clippy -p xcalibre-epub -- -D warnings
git add xcalibre-epub/src/image_opt.rs
git commit -m "R05b-T06: image optimization in container"
```

---

## R05b-T07

Implement `xcalibre-epub/src/css.rs` — basic CSS rewriting:
```rust
use crate::{Container, EpubError};

/// Replace a CSS property value across all stylesheets in the container.
pub fn replace_css_property(
    container: &mut Container,
    property: &str,
    new_value: &str,
) -> Result<u32, EpubError> {
    let css_hrefs: Vec<String> = container.manifest_items().into_iter()
        .filter(|(_, _, mt)| mt == "text/css")
        .map(|(_, href, _)| href)
        .collect();
    let mut count = 0u32;
    let pattern = format!("{}:", property);
    for href in css_hrefs {
        if let Ok(data) = container.read_item(&href) {
            if let Ok(css) = std::str::from_utf8(&data) {
                let updated = rewrite_property(css, &pattern, new_value);
                if updated != css {
                    container.write_item(&href, updated.as_bytes(), "text/css")?;
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn rewrite_property(css: &str, property_colon: &str, new_value: &str) -> String {
    let re = regex::Regex::new(&format!(
        r"(?i)({})\s*:[^;}}]+([;}}])", regex::escape(property_colon.trim_end_matches(':'))
    )).unwrap();
    re.replace_all(css, |caps: &regex::Captures| {
        format!("{}: {}{}", &caps[1], new_value, &caps[2])
    }).into_owned()
}
```

```bash
cargo build -p xcalibre-epub
cargo clippy -p xcalibre-epub -- -D warnings
git add xcalibre-epub/src/css.rs
git commit -m "R05b-T07: CSS property rewriting in container"
```

---

## R05b-T08

Implement `xcalibre-epub/src/split.rs` — split at heading boundary:
The function reads the spine, finds the first `<h1>`/`<h2>` heading in each
spine document beyond the first, and packages the before/after halves into two
new `Container` instances written to separate temp files.

This is complex — implement as a best-effort split:
1. Read all spine documents as strings
2. Find split point (configurable: heading level, character count threshold)
3. Produce two EPUBs with updated OPF spine entries

```bash
cargo build -p xcalibre-epub
cargo clippy -p xcalibre-epub -- -D warnings
git add xcalibre-epub/src/split.rs
git commit -m "R05b-T08: EPUB split at heading boundary"
```

---

## R05b-T09

Implement `xcalibre-epub/src/validation.rs` — all five check categories:
1. **Link checker**: for each spine/manifest item, parse all `href`/`src` attributes and verify the target exists in the container
2. **CSS validator**: try parsing each stylesheet; report parse errors
3. **Font validator**: for each font declared in CSS @font-face, verify the file exists in the container
4. **Image validator**: for each image in the manifest, verify magic bytes match declared media-type
5. **OPF validator**: verify required OPF elements (`<dc:title>`, `<spine>` with at least one `<itemref>`)

```bash
cargo test -p xcalibre-epub -- test_validation
git add xcalibre-epub/src/validation.rs
git commit -m "R05b-T09: EPUB validation — validation tests green"
```

---

## R05b-T10

Implement `xcalibre-epub/src/upgrade.rs` — EPUB 2→3:
1. Read the OPF XML and change `version="2.0"` to `version="3.0"`
2. Add `xmlns:epub="http://www.idpf.org/2007/ops"` namespace
3. If an NCX file exists in the manifest, generate an EPUB 3 `nav.xhtml`
   with the same TOC structure and add it to the manifest with
   `properties="nav"`
4. Keep the NCX for backwards compatibility (mark with `media-type` only)

```bash
cargo test -p xcalibre-epub -- test_upgrade
git add xcalibre-epub/src/upgrade.rs
git commit -m "R05b-T10: EPUB 2→3 upgrade — upgrade tests green"
```

---

## R05b-T11 — Milestone Check + Integration Smoke Test

```bash
cargo test -p xcalibre-epub
cargo clippy -p xcalibre-epub -- -D warnings
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

All tests green. Zero warnings.

**Integration smoke test** (manual):
1. Download a real EPUB from Project Gutenberg
2. Open it with `Container::open`
3. Run `validation::validate` — expect ≤ 3 warnings, 0 errors
4. Run `cover::extract_cover` — expect Some or None (no panic)
5. Run `font::list_fonts` — expect a Vec (any length, no panic)
6. Run `upgrade::epub2_to_epub3` — save as new file, verify it opens

```bash
git add -A
git commit -m "R05b-T11: RMP-05 xcalibre-epub — all tests green, integration smoke verified"
```
