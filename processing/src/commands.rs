use sqlx::SqlitePool;
use std::path::Path;

// This function is meant to be used in a Tauri context but can also be used as a library function
// It's structured to work with both Tauri's command system and standalone usage
pub async fn get_spine(
    pool: &SqlitePool,
    job_id: String,
) -> Result<Vec<String>, String> {
    use std::io::Read;

    let row: Option<(String,)> =
        sqlx::query_as("SELECT file_path FROM jobs WHERE id = ?")
            .bind(&job_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    let file_path = row.ok_or("job not found")?.0;
    let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| e.to_string())?;

    let opf_path = {
        let mut entry = archive.by_name("META-INF/container.xml").map_err(|e| e.to_string())?;
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        let doc = roxmltree::Document::parse(&s).map_err(|e| e.to_string())?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .ok_or("no rootfile")?
            .to_string()
    };
    let opf_dir = Path::new(&opf_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    let opf_xml = {
        let mut entry = archive.by_name(&opf_path).map_err(|e| e.to_string())?;
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        s
    };
    let doc = roxmltree::Document::parse(&opf_xml).map_err(|e| e.to_string())?;
    let hrefs: Vec<String> = doc
        .descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| n.attribute("idref"))
        .filter_map(|idref| {
                doc.descendants()
                    .find(|n| n.tag_name().name() == "item" && n.attribute("id") == Some(idref))
                    .and_then(|n| n.attribute("href"))
                    .map(|href| {
                        if opf_dir.as_os_str().is_empty() {
                            href.to_string()
                        } else {
                            opf_dir.join(href).to_string_lossy().into_owned()
                        }
                    })
            })
        .collect();

    Ok(hrefs)
}
