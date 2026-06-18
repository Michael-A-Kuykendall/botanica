/// Discovery CLI tool: extract master species list from USDA PLANTS
use botanica::discovery::{parse_usda_plants, export_master_list};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage:");
        eprintln!("  discover --usda-file <path_to_usda_plants.csv> <output_master_list.csv>");
        eprintln!();
        eprintln!("Instructions:");
        eprintln!("  1. Download USDA PLANTS data from:");
        eprintln!("     https://plants.sc.egov.usda.gov/adv_search.html");
        eprintln!("  2. Click 'Download plant list' → 'Download all data'");
        eprintln!("  3. Save CSV locally");
        eprintln!("  4. Run this tool to extract actively cultivated species");
        std::process::exit(2);
    }

    let usda_file = if args.get(0).map(|s| s.as_str()) == Some("--usda-file") {
        args.get(1).ok_or("Missing --usda-file path")?
    } else {
        eprintln!("Expected --usda-file flag");
        std::process::exit(2);
    };

    let output_file = args.get(2).ok_or("Missing output CSV path")?;

    // Parse USDA PLANTS CSV
    let records = parse_usda_plants(Some(usda_file)).await?;

    // Export master list
    export_master_list(records, output_file).await?;

    println!("\n✓ Master list generated successfully!");
    println!("  Use with: ingest <db> bulk-load --from-csv {}", output_file);

    Ok(())
}
