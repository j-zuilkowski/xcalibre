pub mod execute;
pub use execute::execute_query;

use crate::error::ProcessingError;

/// AST node produced by parse_query().
#[derive(Debug, Clone, PartialEq)]
pub enum QueryNode {
    /// Bare term — falls through to FTS5
    FtsTerm(String),
    /// Field-scoped match: field:value
    Field { field: String, value: String, negate: bool },
    /// Logical AND of two nodes
    And(Box<QueryNode>, Box<QueryNode>),
    /// Logical OR of two nodes
    Or(Box<QueryNode>, Box<QueryNode>),
    /// Logical NOT of a node
    Not(Box<QueryNode>),
}

/// Supported field names.
const KNOWN_FIELDS: &[&str] = &[
    "title", "author", "tag", "series", "format",
    "publisher", "language", "rating",
];

/// Parse a query string into a QueryNode AST.
/// Grammar (simplified LL(1)):
///   query  := or_expr
///   or_expr  := and_expr ( "OR" and_expr )*
///   and_expr := not_expr ( "AND" not_expr )*
///   not_expr := "NOT" atom | atom
///   atom     := field_term | fts_term | "(" query ")"
///   field_term := IDENT ":" value
///   value     := quoted_string | bare_word
pub fn parse_query(input: &str) -> Result<QueryNode, ProcessingError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ProcessingError::MetadataError("empty query".into()));
    }
    let tokens = tokenize(input);
    let mut pos = 0usize;
    let node = parse_or_expr(&tokens, &mut pos)?;
    Ok(node)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Word(String),
    Quoted(String),
    Colon,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => { chars.next(); }
            ':' => { chars.next(); tokens.push(Token::Colon); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' { chars.next(); break; }
                    s.push(ch); chars.next();
                }
                tokens.push(Token::Quoted(s));
            }
            _ => {
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == ':' || ch == '(' || ch == ')' { break; }
                    word.push(ch); chars.next();
                }
                tokens.push(Token::Word(word));
            }
        }
    }
    tokens
}

fn parse_or_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    let mut left = parse_and_expr(tokens, pos)?;
    while *pos < tokens.len() {
        if let Token::Word(w) = &tokens[*pos] {
            if w.eq_ignore_ascii_case("OR") {
                *pos += 1;
                let right = parse_and_expr(tokens, pos)?;
                left = QueryNode::Or(Box::new(left), Box::new(right));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

fn parse_and_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    let mut left = parse_not_expr(tokens, pos)?;
    while *pos < tokens.len() {
        if let Token::Word(w) = &tokens[*pos] {
            if w.eq_ignore_ascii_case("AND") {
                *pos += 1;
                let right = parse_not_expr(tokens, pos)?;
                left = QueryNode::And(Box::new(left), Box::new(right));
                continue;
            }
            // "NOT" is treated as implicit AND NOT
            if w.eq_ignore_ascii_case("NOT") {
                *pos += 1;
                let inner = parse_atom(tokens, pos)?;
                left = QueryNode::And(Box::new(left), Box::new(QueryNode::Not(Box::new(inner))));
                continue;
            }
            // implicit AND for consecutive terms (skip OR keyword)
            if !w.eq_ignore_ascii_case("OR") {
                let right = parse_not_expr(tokens, pos)?;
                left = QueryNode::And(Box::new(left), Box::new(right));
                continue;
            }
        }
        break;
    }
    Ok(left)
}

fn parse_not_expr(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    if let Some(Token::Word(w)) = tokens.get(*pos) {
        if w.eq_ignore_ascii_case("NOT") {
            *pos += 1;
            let inner = parse_atom(tokens, pos)?;
            return Ok(QueryNode::Not(Box::new(inner)));
        }
    }
    parse_atom(tokens, pos)
}

fn parse_atom(tokens: &[Token], pos: &mut usize) -> Result<QueryNode, ProcessingError> {
    if *pos >= tokens.len() {
        return Err(ProcessingError::MetadataError("unexpected end of query".into()));
    }
    match &tokens[*pos] {
        Token::LParen => {
            *pos += 1;
            let inner = parse_or_expr(tokens, pos)?;
            if let Some(Token::RParen) = tokens.get(*pos) { *pos += 1; }
            Ok(inner)
        }
        Token::Word(w) => {
            let word = w.clone();
            *pos += 1;
            // Check for field:value
            if let Some(Token::Colon) = tokens.get(*pos) {
                *pos += 1;
                let value = match tokens.get(*pos) {
                    Some(Token::Word(v))   => { *pos += 1; v.clone() }
                    Some(Token::Quoted(v)) => { *pos += 1; v.clone() }
                    _ => String::new(),
                };
                let field_lower = word.to_lowercase();
                // Unknown fields fall through as FTS terms
                if KNOWN_FIELDS.contains(&field_lower.as_str()) {
                    return Ok(QueryNode::Field { field: field_lower, value, negate: false });
                } else {
                    return Ok(QueryNode::FtsTerm(format!("{}:{}", word, value)));
                }
            }
            Ok(QueryNode::FtsTerm(word))
        }
        Token::Quoted(q) => {
            let q = q.clone(); *pos += 1;
            Ok(QueryNode::FtsTerm(q))
        }
        _ => Err(ProcessingError::MetadataError("unexpected token in query".into())),
    }
}
