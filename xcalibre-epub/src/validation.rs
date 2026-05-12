// stub — implemented in rmp05b
use crate::EpubError;

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message:  String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity { Error, Warning }

pub fn validate(_container: &crate::Container) -> Result<Vec<ValidationIssue>, EpubError> {
    unimplemented!("rmp05b")
}
