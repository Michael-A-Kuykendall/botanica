/// USDA PLANTS database parser: download and extract actively cultivated species
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsadPlantRecord {
    pub symbol: String,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub family: Option<String>,
    pub is_actively_cultivated: bool,
}

/// Filter plant records to actively cultivated species (exclude extinct, historical, invasive-only)
fn is_actively_cultivated(
    symbol: &str,
    sci_name: &str,
    common_name: &Option<String>,
) -> bool {
    // Exclude obvious non-cultivated patterns
    let lower_sci = sci_name.to_lowercase();
    let lower_common = common_name.as_ref().map(|c| c.to_lowercase());

    // Reject if marked as extinct, historical, or invasive only
    let reject_patterns = vec![
        "extinct", "historical", "no longer", "obsolete",
        "invasive only", "noxious", "eradicated",
    ];

    for pattern in reject_patterns {
        if lower_sci.contains(pattern) {
            return false;
        }
        if let Some(ref c) = lower_common {
            if c.contains(pattern) {
                return false;
            }
        }
    }

    // Accept if it has common names (indicates use)
    if common_name.is_some() && !common_name.as_ref().unwrap().is_empty() {
        return true;
    }

    // Accept well-known cultivated genera
    let cultivated_genera = vec![
        "solanum", "capsicum", "allium", "petroselinum", "ocimum",
        "origanum", "thymus", "salvia", "mentha", "prunus", "malus",
        "pyrus", "fragaria", "musa", "manihot", "ipomoea", "daucus",
        "brassica", "lactuca", "rosa", "oryza", "triticum", "zea",
        "hordeum", "coffea", "theobroma", "vanilla", "citrus",
    ];

    for genus in cultivated_genera {
        if lower_sci.starts_with(genus) {
            return true;
        }
    }

    false
}

/// Download and parse USDA PLANTS CSV (or use local file if provided)
pub async fn parse_usda_plants(
    source: Option<&str>, // Some("path/to/file.csv") or None (download)
) -> anyhow::Result<Vec<UsadPlantRecord>> {
    println!("Parsing USDA PLANTS database...");

    let csv_content = if let Some(path) = source {
        // Use local file
        println!("Reading from local file: {}", path);
        tokio::fs::read_to_string(path).await?
    } else {
        // Download from USDA
        println!("Downloading USDA PLANTS CSV from official source...");
        println!("Note: This requires manual download from https://plants.sc.egov.usda.gov/");
        println!("Download 'Plant List' → 'Download all data' → save to a local file.");
        println!("Then run: discover --usda-file <path>");
        return Err(anyhow::anyhow!(
            "USDA CSV download not yet implemented. Please download manually and provide file path."
        ));
    };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    let mut records = Vec::new();
    let mut skipped = 0;

    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;

        let symbol = record.get("Symbol").cloned().unwrap_or_default();
        let sci_name = record
            .get("Scientific Name")
            .or_else(|| record.get("Sci_Name"))
            .cloned()
            .unwrap_or_default();
        let common_name = record
            .get("Common Names")
            .or_else(|| record.get("CommonNames"))
            .cloned();
        let family = record.get("Family").cloned();

        if sci_name.is_empty() {
            skipped += 1;
            continue;
        }

        if is_actively_cultivated(&symbol, &sci_name, &common_name) {
            records.push(UsadPlantRecord {
                symbol,
                scientific_name: sci_name,
                common_name,
                family,
                is_actively_cultivated: true,
            });
        } else {
            skipped += 1;
        }
    }

    println!(
        "USDA PLANTS: found {} actively cultivated species (skipped {})",
        records.len(),
        skipped
    );
    Ok(records)
}

/// Export parsed records as CSV master list
pub async fn export_master_list(
    records: Vec<UsadPlantRecord>,
    output_path: &str,
) -> anyhow::Result<()> {
    let csv_header = "symbol,scientific_name,common_name,family\n";
    let mut csv_content = csv_header.to_string();

    for rec in records {
        csv_content.push_str(&format!(
            "{},{},{},{}\n",
            rec.symbol,
            rec.scientific_name,
            rec.common_name.unwrap_or_default(),
            rec.family.unwrap_or_default()
        ));
    }

    tokio::fs::write(output_path, csv_content).await?;
    println!("Master list exported to: {}", output_path);

    Ok(())
}
