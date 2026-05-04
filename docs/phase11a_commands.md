# Phase 11 — Exotic Format Metadata & Text Extraction Improvements

> HOW TO USE: For every "Write `path`" line → call your file-write tool with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 10 complete and all tests green.
> Status: 🔴 in progress (Phase 11a)

Phase 11 upgrades stub-format handlers to extract real metadata and text from
binary container formats. Focused on high-impact formats where the binary structure
is documented and test fixtures can be generated programmatically.

## Status

| Task | Title | Status |
|------|-------|--------|
| P11-T01 | PDB/eReader metadata extraction | 🔴 |
| P11-T02 | SNB metadata extraction | ⬜ |
| P11-T03 | TCR text decoding | ⬜ |
| P11-T04 | AZW4 MOBI header extraction | ⬜ |

---

## P11-T01 — PDB/eReader Metadata Extraction

### Current State

PDB metadata reads only the 32-byte name field from the PDB header. All PDB
sub-formats (eReader, Plucker, Haodoo) are treated identically.

### Target

Read the PDB creator ID (8 bytes at offset 60). For eReader PDBs (creator
`PNPdPPrs` or `PNRdPPrs`), parse:
- Record 0 (132-byte eReader header) for compression, has_metadata, and
  metadata_offset fields
- The metadata record for title, author, publisher, and ISBN

### Tests

Write `processing/tests/test_formats_phase11.rs`:

```rust
use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

/// Build a minimal PDB with a creator ID and a 32-byte name field.
fn write_pdb(path: &PathBuf, creator: &[u8; 8], name: &[u8; 32],
             record_offsets: &[u32], record_data: &[&[u8]]) {
    let mut file = std::fs::File::create(path).unwrap();
    // 32-byte name
    file.write_all(name).unwrap();
    // 28 bytes of reserved PDB header fields
    file.write_all(&[0u8; 28]).unwrap();
    // creator ID (8 bytes at offset 60)
    file.write_all(creator).unwrap();
    // 8 bytes: num_sections + padding
    let num_records = record_offsets.len() as u16;
    let num_records_bytes = num_records.to_be_bytes();
    file.write_all(&[0u8; 6]).unwrap();
    file.write_all(&num_records_bytes).unwrap();
    // Record list: 8 bytes per record (offset: u32be, flags: u8, val: u24be)
    let mut header_end = 78u32 + (num_records as u32 * 8) + 2;
    for &data_len in record_offsets {
        let offset_bytes = header_end.to_be_bytes();
        file.write_all(&offset_bytes).unwrap();
        file.write_all(&[0u8; 4]).unwrap(); // flags + val = 0
        header_end += data_len;
    }
    // 2-byte gap
    file.write_all(&[0u8; 2]).unwrap();
    // Record data
    for data in record_data {
        file.write_all(data).unwrap();
    }
}

#[test]
fn test_pdb_ereader_metadata_title() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_ereader.pdb");

    // eReader header record (132 bytes)
    let mut ereader_hdr = [0u8; 132];
    // compression = 2 (PalmDOC), at offset 0 as u16
    ereader_hdr[0..2].copy_from_slice(&2u16.to_be_bytes());
    // has_metadata = 1, at offset 24 as u16
    ereader_hdr[24..26].copy_from_slice(&1u16.to_be_bytes());
    // metadata_offset = 1 (second record), at offset 44 as u16
    ereader_hdr[44..46].copy_from_slice(&1u16.to_be_bytes());
    // last_data_offset = 2, at offset 52 as u16
    ereader_hdr[52..54].copy_from_slice(&2u16.to_be_bytes());

    // metadata record: title\0author\0\0publisher\0isbn\0
    let metadata = b"Test eBook Title\x00Jane Author\x00\x00Test Publisher\x001234567890\x00";

    let mut name = [0u8; 32];
    name[..10].copy_from_slice(b"Test_eBook");

    write_pdb(&path, b"PNPdPPrs", &name, &[132, metadata.len() as u32],
              &[&ereader_hdr, metadata]);

    let meta = metadata::pdb::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Test eBook Title"));
    assert!(meta.authors.contains(&"Jane Author".to_string()));
    assert_eq!(meta.publisher.as_deref(), Some("Test Publisher"));
    assert_eq!(meta.isbn.as_deref(), Some("1234567890"));
}

#[test]
fn test_pdb_ereader_falls_back_to_name_when_no_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_no_meta.pdb");

    // eReader header without metadata flag
    let mut ereader_hdr = [0u8; 132];
    ereader_hdr[0..2].copy_from_slice(&2u16.to_be_bytes());
    // has_metadata = 0
    ereader_hdr[24..26].copy_from_slice(&0u16.to_be_bytes());

    let mut name = [0u8; 32];
    name[..12].copy_from_slice(b"Fallback_Name");

    write_pdb(&path, b"PNPdPPrs", &name, &[132], &[&ereader_hdr]);

    let meta = metadata::pdb::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Fallback_Name"));
}

#[test]
fn test_pdb_non_ereader_uses_name_field() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_palmdoc.pdb");

    // PalmDOC PDB with TEXtREAd creator
    let mut name = [0u8; 32];
    name[..9].copy_from_slice(b"Palm_Book");

    write_pdb(&path, b"TEXtREAd", &name, &[100], &[b"hello world"]);

    let meta = metadata::pdb::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Palm_Book"));
}
```

Then run:
```bash
cd /Users/jonzuilkowski/Documents/localProject/xcalibre/processing
cargo test -- test_pdb_ereader
```

Expected: compilation errors (phase 11a — tests fail until 11b implements the fix).
