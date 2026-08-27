/// API discovery: query POWO, GBIF, USDA for scope of cultivated species
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpeciesRecord {
    pub name: String,
    pub powo_id: Option<String>,
    pub gbif_id: Option<String>,
    pub usda_symbol: Option<String>,
    pub sources: HashSet<String>,
}

/// Query POWO for cultivated plant families (sample approach)
pub async fn discover_powo_species(http: &reqwest::Client, limit: usize) -> anyhow::Result<Vec<SpeciesRecord>> {
    println!("Discovering POWO species (querying common cultivated genera)...");
    
    // Query common cultivated genera to bootstrap
    let genera = vec![
        "Rosa", "Solanum", "Capsicum", "Allium", "Petroselinum",
        "Ocimum", "Origanum", "Thymus", "Salvia", "Mentha",
        "Prunus", "Malus", "Pyrus", "Fragaria", "Musa",
        "Manihot", "Ipomoea", "Daucus", "Brassica", "Lactuca",
    ];

    let mut all_species = Vec::new();
    for genus in genera {
        if all_species.len() >= limit {
            break;
        }
        // In production: query POWO API for species in each genus
        // For now: placeholder to show structure
        println!("  {} - would query POWO API", genus);
    }
    
    println!("POWO discovery: found {} species", all_species.len());
    Ok(all_species)
}

/// Query GBIF backbone for cultivated species
pub async fn discover_gbif_species(http: &reqwest::Client, limit: usize) -> anyhow::Result<Vec<SpeciesRecord>> {
    println!("Discovering GBIF species (querying backbone taxonomy)...");
    
    // Query GBIF backbone API
    // Endpoint: https://api.gbif.org/v1/species/suggest?q=...
    // Filter for kingdom=Plantae, status=ACCEPTED
    
    let all_species = Vec::new();
    // Placeholder
    println!("GBIF discovery: would fetch {} species", limit);
    
    Ok(all_species)
}

/// Query USDA PLANTS for all plant symbols
pub async fn discover_usda_species(http: &reqwest::Client, limit: usize) -> anyhow::Result<Vec<SpeciesRecord>> {
    println!("Discovering USDA PLANTS species...");
    
    // USDA publishes full CSV dumps. Recommend downloading locally:
    // https://plants.sc.egov.usda.gov/adv_search.html
    // "Download plant list as CSV" → species data
    
    // For now: show structure
    let mut all_species = Vec::new();
    println!("USDA discovery: would read {} species from CSV dump", limit);
    
    Ok(all_species)
}

/// Merge all discoveries into a master list, showing source coverage
pub async fn generate_master_list(
    http: &reqwest::Client,
    output_path: &str,
) -> anyhow::Result<()> {
    println!("\n=== Starting API Discovery ===\n");

    let powo_species = discover_powo_species(http, 500).await?;
    let gbif_species = discover_gbif_species(http, 500).await?;
    let usda_species = discover_usda_species(http, 500).await?;

    // Merge by name
    let mut merged: HashMap<String, SpeciesRecord> = HashMap::new();

    for sp in powo_species {
        merged.entry(sp.name.clone())
            .or_insert_with(|| sp.clone())
            .sources.insert("POWO".to_string());
    }

    for sp in gbif_species {
        merged.entry(sp.name.clone())
            .or_insert_with(|| sp.clone())
            .sources.insert("GBIF".to_string());
    }

    for sp in usda_species {
        merged.entry(sp.name.clone())
            .or_insert_with(|| sp.clone())
            .sources.insert("USDA".to_string());
    }

    // Stats
    let total = merged.len();
    let in_all_three = merged.values().filter(|s| s.sources.len() == 3).count();
    let in_powo = merged.values().filter(|s| s.sources.contains("POWO")).count();
    let in_gbif = merged.values().filter(|s| s.sources.contains("GBIF")).count();
    let in_usda = merged.values().filter(|s| s.sources.contains("USDA")).count();

    println!("\n=== Discovery Results ===");
    println!("Total unique species: {}", total);
    println!("  In all 3 sources: {}", in_all_three);
    println!("  POWO only: {}", in_powo - in_all_three);
    println!("  GBIF only: {}", in_gbif - in_all_three);
    println!("  USDA only: {}", in_usda - in_all_three);
    println!();

    // Export to CSV for review
    let csv_header = "name,powo_id,gbif_id,usda_symbol,sources\n";
    let mut csv_content = csv_header.to_string();
    
    let mut records: Vec<_> = merged.values().collect();
    records.sort_by(|a, b| a.name.cmp(&b.name));

    for rec in records {
        let sources_str = rec.sources.iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(";");
        csv_content.push_str(&format!(
            "{},{},{},{},{}\n",
            rec.name,
            rec.powo_id.as_deref().unwrap_or(""),
            rec.gbif_id.as_deref().unwrap_or(""),
            rec.usda_symbol.as_deref().unwrap_or(""),
            sources_str
        ));
    }

    tokio::fs::write(output_path, csv_content).await?;
    println!("Master list exported to: {}", output_path);

    Ok(())
}
