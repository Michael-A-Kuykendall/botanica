use crate::error::DatabaseError;
use std::collections::HashMap;
use std::path::Path;

/// Load genus → family map from CSV (`genus,family` headers).
pub fn load_genus_family_map(path: &Path) -> Result<HashMap<String, String>, DatabaseError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| DatabaseError::validation(format!("read genus_family map: {}", e)))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());
    let mut map = HashMap::new();
    for rec in rdr.deserialize() {
        let row: HashMap<String, String> = rec
            .map_err(|e| DatabaseError::validation(format!("genus_family csv: {}", e)))?;
        let genus = row.get("genus").cloned().unwrap_or_default();
        let family = row.get("family").cloned().unwrap_or_default();
        if !genus.is_empty() && !family.is_empty() {
            map.insert(genus, family);
        }
    }
    Ok(map)
}

/// Parse "Genus epithet …" stripping common authority suffixes.
/// Returns (genus, epithet, scientific_name without authority).
pub fn parse_scientific_name(raw: &str) -> Option<(String, String, String)> {
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }
    let genus = tokens[0].trim_matches(|c: char| !c.is_alphabetic()).to_string();
    let epithet = tokens[1].trim_matches(|c: char| !c.is_alphabetic()).to_string();
    if genus.is_empty() || epithet.is_empty() {
        return None;
    }
    // Drop trailing authority-like tokens (L., Mill., Borkh., etc.)
    let mut end = 2;
    while end < tokens.len() {
        let t = tokens[end];
        let looks_auth = t.ends_with('.')
            || t.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            || t.starts_with('(');
        if looks_auth {
            break;
        }
        end += 1;
    }
    let sci = tokens[..end.min(2)].join(" ");
    Some((genus, epithet, sci))
}
