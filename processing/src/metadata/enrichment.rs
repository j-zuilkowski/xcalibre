use crate::metadata::BookMetadata;
use xcalibre_api::enrichment::{google_books::GBBook, open_library::OLBook};

#[derive(Debug, Clone)]
pub struct EnrichmentSuggestion {
    pub field:  String,
    pub source: String,
    pub value:  String,
}

pub fn build_suggestions(
    existing: &BookMetadata,
    ol: &OLBook,
    gb: &GBBook,
) -> Vec<EnrichmentSuggestion> {
    let mut suggestions = Vec::new();

    fn add(
        suggestions: &mut Vec<EnrichmentSuggestion>,
        field: &str,
        existing: Option<&String>,
        ol_val: Option<String>,
        gb_val: Option<String>,
    ) {
        if existing.map(|s| s.is_empty()).unwrap_or(true) {
            if let Some(val) = ol_val {
                suggestions.push(EnrichmentSuggestion {
                    field:  field.to_string(),
                    source: "Open Library".to_string(),
                    value:  val,
                });
            } else if let Some(val) = gb_val {
                suggestions.push(EnrichmentSuggestion {
                    field:  field.to_string(),
                    source: "Google Books".to_string(),
                    value:  val,
                });
            }
        }
    }

    add(&mut suggestions, "title",       existing.title.as_ref(),       ol.title.clone(),                     gb.title.clone());
    add(&mut suggestions, "publisher",   existing.publisher.as_ref(),   ol.publishers.first().cloned(),       gb.publisher.clone());
    add(&mut suggestions, "published",   existing.published.as_ref(),   ol.publish_date.clone(),              gb.published.clone());
    add(&mut suggestions, "description", existing.description.as_ref(), ol.description.clone(),               gb.description.clone());

    suggestions
}

pub fn apply_suggestions(meta: &mut BookMetadata, accepted: &[EnrichmentSuggestion]) {
    for s in accepted {
        match s.field.as_str() {
            "title"       => meta.title       = Some(s.value.clone()),
            "publisher"   => meta.publisher   = Some(s.value.clone()),
            "published"   => meta.published   = Some(s.value.clone()),
            "description" => meta.description = Some(s.value.clone()),
            _ => {}
        }
    }
}