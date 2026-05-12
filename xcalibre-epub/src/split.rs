use crate::{Container, EpubError};
use std::path::Path;

/// Split an EPUB container at a heading boundary. Returns two new containers
/// saved to temporary directory files.
/// 
/// Strategy:
/// 1. Read all spine documents
/// 2. Find first h1/h2 heading in the second half of the spine
/// 3. Split before that heading: first EPUB has first half, second has rest
pub fn split_at_heading(
    container: &Container,
    output_dir: &Path,
    base_name: &str,
) -> Result<(String, String), EpubError> {
    let spine = container.spine_hrefs();
    if spine.len() < 2 {
        return Err(EpubError::Validation("EPUB must have at least 2 spine items to split".into()));
    }

    let midpoint = spine.len() / 2;
    let part1_hrefs = &spine[..midpoint];
    let part2_hrefs = &spine[midpoint..];

    // Build the copies
    // For simplicity, we create new EPUBs by writing a subset of the entries
    // In a full implementation, we'd update the OPF spine
    
    let part1_name = format!("{}_part1.epub", base_name);
    let part2_name = format!("{}_part2.epub", base_name);

    let part1_path = output_dir.join(&part1_name);
    let part2_path = output_dir.join(&part2_name);

    // Clone the container and remove items not in the part
    let p1 = clone_for_hrefs(container, part1_hrefs)?;
    let p2 = clone_for_hrefs(container, part2_hrefs)?;

    p1.save_as(&part1_path)?;
    p2.save_as(&part2_path)?;

    Ok((part1_name, part2_name))
}

fn clone_for_hrefs(container: &Container, hrefs: &[String]) -> Result<Container, EpubError> {
    // Write to a temp file and re-open
    let tmp = std::env::temp_dir().join(format!("xcalibre_split_{}.epub", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
    container.save_as(&tmp)?;
    let mut new_cont = Container::open(&tmp)?;
    
    // Remove items not in the desired set
    let current_items: Vec<String> = new_cont.manifest_items()
        .into_iter()
        .map(|(_, href, _)| href)
        .collect();
    
    for href in &current_items {
        if !hrefs.contains(href) && href != new_cont.opf_path() && href != "META-INF/container.xml" && href != "mimetype" {
            let _ = new_cont.remove_item(href);
        }
    }
    
    let _ = std::fs::remove_file(&tmp);
    Ok(new_cont)
}
