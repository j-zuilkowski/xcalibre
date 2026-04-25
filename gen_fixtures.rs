use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() {
    // fixture_epub.epub — minimal but structurally complete EPUB
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
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();

        zip.start_file("OEBPS/content.opf", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>Fixture Book</dc:title>
    <dc:creator opf:role="aut">Test Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier id="uid">urn:isbn:9780000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
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
<body><p>This is the fixture chapter one body text for testing.</p></body>
</html>"#).unwrap();

        zip.finish().unwrap();
    }

    // fixture_pdf.pdf — minimal valid PDF with %%EOF
    {
        let mut f = File::create("tests/fixtures/fixture_pdf.pdf").unwrap();
        f.write_all(b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
xref\n0 4\n0000000000 65535 f \n0000000010 00000 n \n\
0000000053 00000 n \n0000000102 00000 n \n\
trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n149\n%%EOF").unwrap();
    }

    // fixture_mobi.mobi — 60 zero bytes + BOOKMOBI + 64 zero bytes (>= 132 bytes total)
    {
        let mut f = File::create("tests/fixtures/fixture_mobi.mobi").unwrap();
        f.write_all(&vec![0u8; 60]).unwrap();
        f.write_all(b"BOOKMOBI").unwrap();
        f.write_all(&vec![0u8; 64]).unwrap();
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

    // fixture_cbr.cbr — RAR v4 magic
    {
        let mut f = File::create("tests/fixtures/fixture_cbr.cbr").unwrap();
        f.write_all(b"Rar!\x1a\x07\x00").unwrap();
    }

    // fixture_txt.txt
    {
        let mut f = File::create("tests/fixtures/fixture_txt.txt").unwrap();
        f.write_all(b"Hello world\n").unwrap();
    }

    // fixture_zero.bin — all zero bytes (no recognisable magic)
    {
        let mut f = File::create("tests/fixtures/fixture_zero.bin").unwrap();
        f.write_all(&vec![0u8; 128]).unwrap();
    }

    println!("Fixtures written to tests/fixtures/");
}
