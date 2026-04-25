use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

fn write_fixture(path: &PathBuf, prefix: &[u8]) {
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(prefix).unwrap();
    file.write_all(b"\nRecovered Title\nSome readable text content for phase ten.")
        .unwrap();
}

#[test]
fn test_exotic_metadata_recovery_titles() {
    let dir = tempfile::tempdir().unwrap();

    let chm = dir.path().join("sample.chm");
    write_fixture(&chm, b"ITSF");
    assert_eq!(
        metadata::chm::extract(&chm).unwrap().title.as_deref(),
        Some("Recovered Title"),
    );

    let lit = dir.path().join("sample.lit");
    write_fixture(&lit, b"ITOLITLS");
    assert_eq!(
        metadata::lit::extract(&lit).unwrap().title.as_deref(),
        Some("Recovered Title"),
    );

    let lrf = dir.path().join("sample.lrf");
    write_fixture(&lrf, b"L\0R\0F\0");
    assert_eq!(
        metadata::lrf::extract(&lrf).unwrap().title.as_deref(),
        Some("Recovered Title"),
    );
}

#[test]
fn test_exotic_text_recovery() {
    let dir = tempfile::tempdir().unwrap();

    let chm = dir.path().join("sample.chm");
    write_fixture(&chm, b"ITSF");
    assert!(text::chm::extract(&chm).unwrap().word_count > 0);

    let lit = dir.path().join("sample.lit");
    write_fixture(&lit, b"ITOLITLS");
    assert!(text::lit::extract(&lit).unwrap().word_count > 0);

    let lrf = dir.path().join("sample.lrf");
    write_fixture(&lrf, b"L\0R\0F\0");
    assert!(text::lrf::extract(&lrf).unwrap().word_count > 0);

    let snb = dir.path().join("sample.snb");
    write_fixture(&snb, b"SNBP");
    assert!(text::snb::extract(&snb).unwrap().word_count > 0);

    let azw4 = dir.path().join("sample.azw4");
    write_fixture(&azw4, b"AZW4");
    assert!(text::azw4::extract(&azw4).unwrap().word_count > 0);

    let djvu = dir.path().join("sample.djvu");
    write_fixture(&djvu, b"AT&TFORM");
    assert!(text::djvu::extract(&djvu).unwrap().word_count > 0);
}
