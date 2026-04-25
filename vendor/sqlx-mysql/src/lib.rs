#![deny(unsafe_code)]

// This local patch replaces the upstream mysql backend package so the
// workspace can resolve sqlx without pulling in the rsa advisory path.
