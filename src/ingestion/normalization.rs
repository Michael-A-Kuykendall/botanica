/// Trait value normalization to controlled vocabulary
pub struct TraitNormalizer;

impl TraitNormalizer {
    /// Normalize growth_habit raw text to controlled vocab
    pub fn normalize_growth_habit(raw: &str) -> String {
        let lower = raw.to_lowercase();
        match lower.as_str() {
            s if s.contains("tree") => "tree".to_string(),
            s if s.contains("shrub") => "shrub".to_string(),
            s if s.contains("herb") || s.contains("herbaceous") => "herb".to_string(),
            s if s.contains("vine") || s.contains("climbing") => "vine".to_string(),
            s if s.contains("grass") => "grass".to_string(),
            s if s.contains("sedge") => "sedge".to_string(),
            s if s.contains("fern") => "fern".to_string(),
            s if s.contains("succulent") => "succulent".to_string(),
            s if s.contains("tree") || s.contains("shrub") => {
                // If it's a compound like "tree/shrub" take the first
                if let Some(first) = s.split('/').next() {
                    Self::normalize_growth_habit(first)
                } else {
                    "other".to_string()
                }
            }
            _ if !raw.trim().is_empty() => "other".to_string(),
            _ => "".to_string(),
        }
    }

    /// Normalize duration to controlled vocab
    pub fn normalize_duration(raw: &str) -> String {
        let lower = raw.to_lowercase();
        match lower.as_str() {
            s if s.contains("annual") => "annual".to_string(),
            s if s.contains("biennial") => "biennial".to_string(),
            s if s.contains("perennial") => "perennial".to_string(),
            _ if !raw.trim().is_empty() => "other".to_string(),
            _ => "".to_string(),
        }
    }

    /// Normalize tolerance levels
    pub fn normalize_tolerance(raw: &str) -> String {
        let lower = raw.to_lowercase();
        match lower.as_str() {
            s if s.contains("none") || s == "0" || s.contains("no ") => "none".to_string(),
            s if s.contains("low") || s == "1" => "low".to_string(),
            s if s.contains("medium") || s.contains("moderate") || s == "2" => "medium".to_string(),
            s if s.contains("high") || s.contains("very high") || s == "3" => "high".to_string(),
            _ if !raw.trim().is_empty() => "unknown".to_string(),
            _ => "".to_string(),
        }
    }

    /// Normalize wetland indicator codes (USDA specific)
    pub fn normalize_wetland_indicator(raw: &str) -> String {
        let lower = raw.to_lowercase().trim().to_string();
        match lower.as_str() {
            "ob" | "obligate_wetland" | "obligate wetland" => "obligate_wetland".to_string(),
            "facw" | "facultative_wetland" | "facultative wetland" => "facultative_wetland".to_string(),
            "fac" | "facultative" => "facultative".to_string(),
            "facu" | "facultative_upland" | "facultative upland" => "facultative_upland".to_string(),
            "up" | "upland" => "upland".to_string(),
            _ if !raw.trim().is_empty() => "unknown".to_string(),
            _ => "".to_string(),
        }
    }

    /// Parse numeric height (meters). Accepts formats like "10", "10.5", "10-15" (takes first).
    pub fn parse_height_meters(raw: &str) -> Option<f64> {
        let trimmed = raw.trim();
        // Try parsing as single number first
        if let Ok(val) = trimmed.parse::<f64>() {
            if val > 0.0 && val < 10000.0 {
                return Some((val * 100.0).round() / 100.0); // round to 2 decimals
            }
        }
        // Try extracting first number from ranges like "10-15"
        for part in trimmed.split(|c| c == '-' || c == '/' || c == ' ') {
            if let Ok(val) = part.trim().parse::<f64>() {
                if val > 0.0 && val < 10000.0 {
                    return Some((val * 100.0).round() / 100.0);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_growth_habit() {
        assert_eq!(TraitNormalizer::normalize_growth_habit("Tree"), "tree");
        assert_eq!(TraitNormalizer::normalize_growth_habit("shrub"), "shrub");
        assert_eq!(TraitNormalizer::normalize_growth_habit("herbaceous"), "herb");
        assert_eq!(TraitNormalizer::normalize_growth_habit("tree/shrub"), "tree");
    }

    #[test]
    fn test_normalize_duration() {
        assert_eq!(TraitNormalizer::normalize_duration("Annual"), "annual");
        assert_eq!(TraitNormalizer::normalize_duration("Perennial"), "perennial");
    }

    #[test]
    fn test_normalize_tolerance() {
        assert_eq!(TraitNormalizer::normalize_tolerance("High"), "high");
        assert_eq!(TraitNormalizer::normalize_tolerance("none"), "none");
        assert_eq!(TraitNormalizer::normalize_tolerance("3"), "high");
    }

    #[test]
    fn test_parse_height_meters() {
        assert_eq!(TraitNormalizer::parse_height_meters("10.5"), Some(10.5));
        assert_eq!(TraitNormalizer::parse_height_meters("10-15"), Some(10.0));
        assert_eq!(TraitNormalizer::parse_height_meters("0"), None);
    }
}
