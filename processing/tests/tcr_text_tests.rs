use std::io::Write;
use tempfile::NamedTempFile;
use xcalibre_processing::text::tcr::extract;

/// Build a minimal valid TCR file.
/// Format: 256 variable-length C-strings (lookup table), then body bytes (indices into table).
fn make_tcr(table: &[&str; 256], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in table {
        out.extend_from_slice(entry.as_bytes());
        out.push(0u8); // null terminator
    }
    out.extend_from_slice(body);
    out
}

#[test]
fn tcr_decodes_simple_text() {
    // Build table: entry 0 = "Hello", entry 1 = " world", rest = empty strings
    let mut table = [""; 256];
    table[0] = "Hello";
    table[1] = " world";
    // body: indices [0, 1] => "Hello world"
    let data = make_tcr(&table, &[0u8, 1u8]);

    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let result = extract(f.path()).unwrap();
    assert!(result.full_text.contains("Hello"), "expected 'Hello' in {:?}", result.full_text);
    assert!(result.full_text.contains("world"), "expected 'world' in {:?}", result.full_text);
    assert!(result.word_count >= 2);
}

#[test]
fn tcr_empty_body_returns_empty() {
    let table = [""; 256];
    let data = make_tcr(&table, &[]);
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let result = extract(f.path()).unwrap();
    assert_eq!(result.full_text, "");
    assert_eq!(result.word_count, 0);
}

#[test]
fn tcr_too_short_returns_empty() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"ab").unwrap();
    let result = extract(f.path()).unwrap();
    assert_eq!(result.full_text, "");
}
