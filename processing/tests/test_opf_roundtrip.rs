//! Round-trip test: import Calibre metadata.opf → edit → write → re-read.

use xcalibre_processing::opf::{read_opf, write_opf_sidecar};
use std::path::PathBuf;

fn write_sample_calibre_opf(path: &PathBuf) {
    // Minimal Calibre-compatible OPF 2.x
    std::fs::write(path, r#"<?xml version='1.0' encoding='utf-8'?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uuid_id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf"
            xmlns:calibre="http://calibre.kovidgoyal.net/2009/metadata">
    <dc:title>Original Title</dc:title>
    <dc:creator opf:role="aut">Original Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:publisher>Original Press</dc:publisher>
    <meta name="calibre:series" content="Original Series"/>
    <meta name="calibre:series_index" content="1"/>
  </metadata>
</package>"#).unwrap();
}

#[test]
fn test_read_calibre_opf() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_sample_calibre_opf(&opf_path);
    let meta = read_opf(&opf_path).expect("read");
    assert_eq!(meta.title.as_deref(), Some("Original Title"));
    assert_eq!(meta.authors, vec!["Original Author"]);
    assert_eq!(meta.series.as_deref(), Some("Original Series"));
    assert_eq!(meta.series_index, Some(1.0));
}

#[test]
fn test_round_trip_preserves_fields() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_sample_calibre_opf(&opf_path);

    let mut meta = read_opf(&opf_path).unwrap();
    meta.title = Some("New Title".into());
    meta.authors = vec!["New Author".into()];
    write_opf_sidecar(&opf_path, &meta).unwrap();

    let re_read = read_opf(&opf_path).unwrap();
    assert_eq!(re_read.title.as_deref(), Some("New Title"));
    assert_eq!(re_read.authors, vec!["New Author"]);
    // Series must survive the round-trip
    assert_eq!(re_read.series.as_deref(), Some("Original Series"));
}
