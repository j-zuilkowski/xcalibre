use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() {
    std::fs::create_dir_all("tests/fixtures").unwrap();
    std::fs::create_dir_all("processing/tests/fixtures").unwrap();

    // fixture_epub.epub — complete EPUB with spine, metadata, and body text
    {
        let f = File::create("tests/fixtures/fixture_epub.epub").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);

        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        zip.start_file("META-INF/container.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"
              media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();

        zip.start_file("OEBPS/content.opf", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>Fixture Book</dc:title>
    <dc:creator opf:role="aut">Test Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier id="uid">urn:isbn:9780000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="chapter1.xhtml"
          media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#).unwrap();

        zip.start_file("OEBPS/chapter1.xhtml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body><p>This is the fixture chapter one body text for testing purposes.</p></body>
</html>"#).unwrap();

        zip.finish().unwrap();
    }

    // fixture_pdf.pdf
    {
        let mut f = File::create("tests/fixtures/fixture_pdf.pdf").unwrap();
        f.write_all(b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
xref\n0 4\n0000000000 65535 f \n0000000010 00000 n \n\
0000000053 00000 n \n0000000102 00000 n \n\
trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n149\n%%EOF").unwrap();
    }

    // fixture_mobi.mobi — 60 zeros + BOOKMOBI + 64 zeros (>= 132 bytes)
    {
        let mut f = File::create("tests/fixtures/fixture_mobi.mobi").unwrap();
        f.write_all(&[0u8; 60]).unwrap();
        f.write_all(b"BOOKMOBI").unwrap();
        f.write_all(&[0u8; 64]).unwrap();
    }

    // fixture_cbz.cbz — ZIP without mimetype entry
    {
        let f = File::create("tests/fixtures/fixture_cbz.cbz").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("page001.jpg", stored).unwrap();
        zip.write_all(b"JFIF_placeholder").unwrap();
        zip.finish().unwrap();
    }

    // fixture_cbr.cbr — RAR v4 magic bytes only
    {
        let mut f = File::create("tests/fixtures/fixture_cbr.cbr").unwrap();
        f.write_all(b"Rar!\x1a\x07\x00").unwrap();
    }

    // fixture_txt.txt
    {
        let mut f = File::create("tests/fixtures/fixture_txt.txt").unwrap();
        f.write_all(b"Hello world\n").unwrap();
    }

    // fixture_zero.bin — all zeros (unknown format)
    {
        let mut f = File::create("tests/fixtures/fixture_zero.bin").unwrap();
        f.write_all(&[0u8; 128]).unwrap();
    }

    // fixture_html.html
    {
        let mut f = File::create("tests/fixtures/fixture_html.html").unwrap();
        f.write_all(b"<!DOCTYPE html><html><head><title>HTML Fixture</title><meta name='author' content='Jane Doe'></head><body><p>Hello from HTML fixture.</p></body></html>").unwrap();
    }

    // fixture_rtf.rtf
    {
        let mut f = File::create("tests/fixtures/fixture_rtf.rtf").unwrap();
        f.write_all(br"{\rtf1\ansi {\title RTF Fixture}{\author Test Author}\par Hello RTF world.\par}").unwrap();
    }

    // fixture_fb2.fb2
    {
        let mut f = File::create("tests/fixtures/fixture_fb2.fb2").unwrap();
        f.write_all(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\"><description><title-info><author><first-name>Test</first-name><last-name>Author</last-name></author><book-title>FB2 Fixture</book-title><lang>en</lang></title-info></description><body><section><p>Fixture body text.</p></section></body></FictionBook>").unwrap();
    }

    // fixture_docx.docx — minimal OOXML ZIP
    {
        let f = File::create("tests/fixtures/fixture_docx.docx").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("docProps/core.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
    xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>DOCX Fixture</dc:title>
  <dc:creator>Test Author</dc:creator>
</cp:coreProperties>"#).unwrap();
        zip.start_file("word/document.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>Hello from DOCX fixture.</w:t></w:r></w:p></w:body>
</w:document>"#).unwrap();
        zip.finish().unwrap();
    }

    // fixture_odt.odt — minimal ODF ZIP
    {
        let f = File::create("tests/fixtures/fixture_odt.odt").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/vnd.oasis.opendocument.text").unwrap();
        zip.start_file("meta.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0">
  <office:meta>
    <dc:title>ODT Fixture</dc:title>
    <dc:creator>Test Author</dc:creator>
  </office:meta>
</office:document-meta>"#).unwrap();
        zip.start_file("content.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body><office:text><text:p>Hello from ODT fixture.</text:p></office:text></office:body>
</office:document-content>"#).unwrap();
        zip.finish().unwrap();
    }

    for name in [
        "fixture_html.html",
        "fixture_rtf.rtf",
        "fixture_fb2.fb2",
        "fixture_docx.docx",
        "fixture_odt.odt",
    ] {
        let src = format!("tests/fixtures/{}", name);
        let dst = format!("processing/tests/fixtures/{}", name);
        let _ = std::fs::copy(src, dst);
    }

    println!("Fixtures written to tests/fixtures/");
}
