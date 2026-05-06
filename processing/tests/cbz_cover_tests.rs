use std::io::Write;
use xcalibre_processing::cover::cbz::extract;

#[test]
fn cbz_cover_extracts_first_image() {
    // fixture_cbz.cbz must contain at least one image entry
    let path = std::path::PathBuf::from("tests/fixtures/fixture_cbz.cbz");
    if !path.exists() { return; } // skip if fixture absent
    let result = extract(&path).unwrap();
    assert!(result.is_some(), "expected a cover image from CBZ");
    let cover = result.unwrap();
    assert!(!cover.data.is_empty());
    assert!(cover.mime_type.starts_with("image/"));
}

#[test]
fn cbz_empty_zip_returns_none() {
    use std::io::Cursor;
    use tempfile::NamedTempFile;
    let buf = Cursor::new(Vec::new());
    let zip = zip::ZipWriter::new(buf);
    let data = zip.finish().unwrap().into_inner();
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let result = extract(f.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn cbz_non_image_entries_skipped() {
    use std::io::Cursor;
    use tempfile::NamedTempFile;
    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let opts = zip::write::FileOptions::default();
    zip.start_file("readme.txt", opts).unwrap();
    zip.write_all(b"not an image").unwrap();
    let data = zip.finish().unwrap().into_inner();
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let result = extract(f.path()).unwrap();
    assert!(result.is_none());
}
