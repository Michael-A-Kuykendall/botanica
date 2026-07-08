//! Build cultivated seed artifacts.
//!
//! Modes:
//!   cargo run --bin build_seed -- gate2 [repo_root]
//!   cargo run --bin build_seed -- usda   [repo_root]
//!
//! `usda` loads PlantSearch master CSV (after scripts/fetch_genus_families.py + build_usda_master.py).

use botanica::database::{BotanicalDatabase, DatabaseConfig};
use botanica::seed::{export, gate2, manifest, usda_catalog};
use std::path::{Path, PathBuf};

/// Newest `USDA_PLANTS_norm_*.json` under a gate output dir (botanica_usda layout).
fn find_latest_norm(dir: &Path) -> Option<PathBuf> {
    let norm = dir.join("normalized");
    let search = if norm.is_dir() { norm } else { dir.to_path_buf() };
    let mut files: Vec<_> = std::fs::read_dir(&search)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|x| x.to_str()) == Some("json")
                && p.file_name()
                    .and_then(|x| x.to_str())
                    .map(|n| n.contains("norm") || n.starts_with("USDA_PLANTS_norm"))
                    .unwrap_or(false)
        })
        .collect();
    files.sort();
    files.pop()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mode = if args.first().map(|s| s.as_str()) == Some("gate2")
        || args.first().map(|s| s.as_str()) == Some("usda")
    {
        args.remove(0)
    } else {
        "gate2".to_string()
    };
    let root = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));

    let duckdb_path = root.join("data/botanica-cultivated-v0.1.duckdb");
    let silver_dir = root.join("data/silver");
    let manifest_path = root.join("data/manifests/botanica-cultivated-v0.1.json");

    if duckdb_path.exists() {
        std::fs::remove_file(&duckdb_path)?;
    }

    println!("mode={mode} db={}", duckdb_path.display());
    let db = BotanicalDatabase::new(DatabaseConfig::file(duckdb_path.to_string_lossy())).await?;
    db.migrate().await?;

    let mut sources = Vec::new();

    match mode.as_str() {
        "gate2" => {
            let bronze = root.join("data/bronze/gate2/USDA_PLANTS_norm.json");
            let lookup = root.join("data/lookups/genus_family.csv");
            if !bronze.exists() {
                eprintln!("Missing {}", bronze.display());
                std::process::exit(2);
            }
            let stats = gate2::ingest_gate2_json(&db, &bronze, &lookup).await?;
            println!(
                "gate2 accepted={} quarantined={}",
                stats.accepted, stats.quarantined
            );
            sources.push(manifest::SeedSource {
                name: "USDA_PLANTS_GATE2".into(),
                license: "Public Domain".into(),
                record_count: stats.accepted as i64,
                notes: format!("quarantine={}", stats.quarantined),
            });
        }
        "usda" => {
            // Prefer full USDA genus map if present
            let master = root.join("data/bronze/usda_catalog/master_species.csv");
            if !master.exists() {
                eprintln!(
                    "Missing {}. Run:\n  python scripts/fetch_genus_families.py\n  python scripts/build_usda_master.py",
                    master.display()
                );
                std::process::exit(2);
            }
            // Also fold gate2 traits if present (enrichment)
            let lookup = if root.join("data/lookups/genus_family_usda.csv").exists() {
                root.join("data/lookups/genus_family_usda.csv")
            } else {
                root.join("data/lookups/genus_family.csv")
            };

            let stats = usda_catalog::ingest_master_csv(&db, &master).await?;
            println!(
                "usda catalog accepted={} skipped_existing={} quarantined={}",
                stats.accepted, stats.skipped_existing, stats.quarantined
            );
            sources.push(manifest::SeedSource {
                name: "USDA_PLANTS_CATALOG".into(),
                license: "Public Domain".into(),
                record_count: stats.accepted as i64,
                notes: format!(
                    "master={} skipped_existing={} skipped_rows={}",
                    master.display(),
                    stats.skipped_existing,
                    stats.quarantined
                ),
            });

            // Trait enrich: prefer Gate3 (1k) then Gate2 pilot if present
            let enrich_paths = [
                (
                    "USDA_PLANTS_GATE3_ENRICH",
                    find_latest_norm(&root.join("data/bronze/gate3")),
                ),
                (
                    "USDA_PLANTS_GATE2_ENRICH",
                    Some(root.join("data/bronze/gate2/USDA_PLANTS_norm.json"))
                        .filter(|p| p.exists()),
                ),
            ];
            for (name, path_opt) in enrich_paths {
                let Some(json_path) = path_opt else { continue };
                if !json_path.exists() {
                    continue;
                }
                let gstats = gate2::ingest_gate2_json(&db, &json_path, &lookup).await?;
                println!(
                    "{} traits≈{} vernacular≈{} path={}",
                    name, gstats.traits, gstats.vernacular, json_path.display()
                );
                sources.push(manifest::SeedSource {
                    name: name.into(),
                    license: "Public Domain".into(),
                    record_count: gstats.traits as i64,
                    notes: format!("enrich {}", json_path.display()),
                });
            }
        }
        other => {
            eprintln!("Unknown mode: {other} (use gate2|usda)");
            std::process::exit(2);
        }
    }

    println!("Exporting silver parquet → {}", silver_dir.display());
    let silver_files = export::export_silver_parquet(&db, &silver_dir).await?;
    for f in &silver_files {
        println!("  {f}");
    }

    let m = manifest::write_manifest(
        &db,
        &manifest_path,
        "botanica-cultivated-v0.1",
        silver_files,
        sources,
    )
    .await?;

    println!("MANIFEST → {}", manifest_path.display());
    println!(
        "counts: species={} families={} genera={} traits={} vernacular={} plants(L3)={} quarantine={}",
        m.counts.species,
        m.counts.families,
        m.counts.genera,
        m.counts.traits,
        m.counts.vernacular_names,
        m.counts.plants,
        m.counts.quarantine
    );
    println!("Done.");
    Ok(())
}
