use xcalibre_epub::opf::EpubOPF;

const SAMPLE_OPF_2: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>Sample Book</dc:title>
    <dc:creator opf:role="aut">Jane Doe</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ch1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="ch1"/></spine>
</package>"#;

#[test]
fn test_parse_opf2_title() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    assert_eq!(opf.title.as_deref(), Some("Sample Book"));
    assert_eq!(opf.authors, vec!["Jane Doe"]);
    assert_eq!(opf.epub_version, "2.0");
}

#[test]
fn test_parse_opf2_manifest() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    assert_eq!(opf.manifest.len(), 1);
    assert_eq!(opf.manifest[0].href, "chapter1.xhtml");
}

#[test]
fn test_opf_to_xml_round_trip() {
    let opf = EpubOPF::parse(SAMPLE_OPF_2).unwrap();
    let xml = opf.to_xml().unwrap();
    let re_parsed = EpubOPF::parse(&xml).unwrap();
    assert_eq!(re_parsed.title, opf.title);
    assert_eq!(re_parsed.authors, opf.authors);
}
