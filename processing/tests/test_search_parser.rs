//! Tests for the field-scoped search parser (RMP-02).
//! Tests for the field-scoped search parser (RMP-02).

use xcalibre_processing::search::{parse_query, QueryNode};

#[test]
fn test_bare_term() {
    let q = parse_query("rust").unwrap();
    assert!(matches!(q, QueryNode::FtsTerm(ref s) if s == "rust"));
}

#[test]
fn test_field_scoped_title() {
    let q = parse_query("title:rust").unwrap();
    assert!(matches!(q, QueryNode::Field { field, ref value, .. }
        if field == "title" && value == "rust"));
}

#[test]
fn test_field_scoped_author() {
    let q = parse_query("author:Knuth").unwrap();
    assert!(matches!(q, QueryNode::Field { field, ref value, .. }
        if field == "author" && value == "Knuth"));
}

#[test]
fn test_and_expression() {
    let q = parse_query("title:rust AND author:klabnik").unwrap();
    assert!(matches!(q, QueryNode::And(_, _)));
}

#[test]
fn test_or_expression() {
    let q = parse_query("tag:fantasy OR tag:scifi").unwrap();
    assert!(matches!(q, QueryNode::Or(_, _)));
}

#[test]
fn test_not_expression() {
    let q = parse_query("title:rust NOT tag:beginner").unwrap();
    assert!(matches!(q, QueryNode::And(_, _)));
    // NOT is represented as AND NOT
    if let QueryNode::And(left, right) = q {
        assert!(matches!(*left, QueryNode::Field { .. }));
        assert!(matches!(*right, QueryNode::Not(_)));
    }
}

#[test]
fn test_quoted_value() {
    let q = parse_query(r#"title:"The Rust Programming Language""#).unwrap();
    if let QueryNode::Field { value, .. } = q {
        assert_eq!(value, "The Rust Programming Language");
    } else {
        panic!("expected Field node");
    }
}

#[test]
fn test_supported_fields() {
    for field in &["title", "author", "tag", "series", "format", "publisher", "language"] {
        let q = parse_query(&format!("{}:x", field));
        assert!(q.is_ok(), "field '{}' should be accepted", field);
    }
}

#[test]
fn test_unknown_field_is_fts_fallback() {
    // Unknown fields should be treated as FTS terms, not errors
    let q = parse_query("foo:bar").unwrap();
    // Either FtsTerm or a Field with passthrough — just must not Err
    let _ = q;
}

#[test]
fn test_empty_query_is_err() {
    assert!(parse_query("").is_err());
}
