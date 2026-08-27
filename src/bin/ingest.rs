// Simple real-world ingestion CLI (no mocks)
// Build with: cargo build --features ingestion --bin ingest

use std::env;
#[cfg(feature = "ingestion")]
use botanica::ingestion::{powo, gbif, usda, fts, usda_csv, perf, bulk};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!(
            "Usage:\n  ingest <db_path> powo <species_id> <powo_id>\n  ingest <db_path> gbif <species_id> <gbif_id>\n  ingest <db_path> usda <species_id> <usda_key>\n  ingest <db_path> usda-csv <csv_path>\n  ingest <db_path> fts-rebuild\n  ingest <db_path> perf\n  ingest <db_path> bulk-load [max_species]\n"
        );
        std::process::exit(2);
    }

    let db_path = &args[0];
    let cmd = &args[1];

    let config = botanica::database::DatabaseConfig::file(db_path);
    let db = botanica::database::BotanicalDatabase::new(config).await?;
    db.migrate().await?;

    match cmd.as_str() {
        "powo" => {
            if args.len() < 4 { eprintln!("Missing args: <species_id> <powo_id>"); std::process::exit(2); }
            let species_id = args[2].clone();
            let powo_id = args[3].clone();
            #[cfg(feature = "ingestion")]
            {
                let mut client = powo::PowoClient::default();
                if let Ok(override_url) = env::var("POWO_BASE_URL") { client.base_url = override_url; }
                powo::ingest_powo_for_species(&db, &species_id, &powo_id, &client).await?;
                println!("POWO ingestion completed: species={} powo_id={}", species_id, powo_id);
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion to enable POWO ingestion");
                std::process::exit(3);
            }
        }
        "gbif" => {
            if args.len() < 4 { eprintln!("Missing args: <species_id> <gbif_id>"); std::process::exit(2); }
            let species_id = args[2].clone();
            let gbif_id = args[3].clone();
            #[cfg(feature = "ingestion")]
            {
                let mut client = gbif::GbifClient::default();
                if let Ok(override_url) = env::var("GBIF_BASE_URL") { client.base_url = override_url; }
                gbif::ingest_gbif_vernacular(&db, &species_id, &gbif_id, &client).await?;
                println!("GBIF ingestion completed: species={} gbif_id={}", species_id, gbif_id);
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion to enable GBIF ingestion");
                std::process::exit(3);
            }
        }
        "fts-rebuild" => {
            #[cfg(feature = "ingestion")]
            {
                fts::rebuild_species_name_fts(&db).await?;
                println!("Done: FTS rebuilt");
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion");
                std::process::exit(3);
            }
        }
        "perf" => {
            #[cfg(feature = "ingestion")]
            {
                perf::benchmark_search(&db).await?;
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion");
                std::process::exit(3);
            }
        }
        "usda" => {
            if args.len() < 4 { eprintln!("Missing args: <species_id> <usda_key>"); std::process::exit(2); }
            let species_id = args[2].clone();
            let usda_key = args[3].clone();
            #[cfg(feature = "ingestion")]
            {
                let mut client = usda::UsdaClient::default();
                if let Ok(override_url) = env::var("USDA_BASE_URL") { client.base_url = override_url; }
                usda::ingest_usda_traits(&db, &species_id, &usda_key, &client).await?;
                println!("USDA ingestion completed: species={} usda_key={}", species_id, usda_key);
            }
        }
        "usda-csv" => {
            if args.len() < 3 { eprintln!("Missing args: <csv_path>"); std::process::exit(2); }
            let csv_path = args[2].clone();
            #[cfg(feature = "ingestion")]
            {
                usda_csv::ingest_usda_csv(&db, &csv_path).await?;
                println!("Done: CSV {}", csv_path);
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion");
                std::process::exit(3);
            }
        }
        "bulk-load" => {
            #[cfg(feature = "ingestion")]
            {
                let master_list_path = args.get(2).cloned();
                let max_species = args.get(3).and_then(|s| s.parse::<usize>().ok());
                bulk::bulk_ingest_cultivated(&db, max_species, master_list_path.as_deref()).await?;
                println!("Done: Bulk ingestion complete");
            }
            #[cfg(not(feature = "ingestion"))]
            {
                eprintln!("Rebuild with --features ingestion");
                std::process::exit(3);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(2);
        }
    }

    Ok(())
}
