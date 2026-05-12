fn sanitize_sort_field(field: &str) -> Result<&'static str, crate::error::ProcessingError> {
    match field {
        "title" => Ok("title"),
        "authors" => Ok("authors"),
        "format" => Ok("format"),
        "last_opened_at" => Ok("last_opened_at"),
        "progress_percent" => Ok("progress_percent"),
        _ => Err(crate::error::ProcessingError::InvalidInput(
            format!("unknown sort field: {field}"),
        )),
    }
}
