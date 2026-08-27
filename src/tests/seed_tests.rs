use crate::seed::lookup::parse_scientific_name;
use crate::create_test_database;
use crate::seed::gate2;
use crate::queries::search;
use std::path::PathBuf;

#[test]
fn test_parse_scientific_name() {
    let (g, e, sci) = parse_scientific_name("Acer negundo L.").unwrap();
    assert_eq!(g, "Acer");
    assert_eq!(e, "negundo");
    assert_eq!(sci, "Acer negundo");
    assert!(parse_scientific_name("Monotypic").is_none());
}

#[tokio::test]
async fn test_gate2_ingest_if_fixture_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = root.join("data/bronze/gate2/USDA_PLANTS_norm.json");
    let lookup = root.join("data/lookups/genus_family.csv");
    if !json.exists() || !lookup.exists() {
        eprintln!("skip: gate2 fixture not present");
        return;
    }

    let db = create_test_database().await.unwrap();
    let stats = gate2::ingest_gate2_json(&db, &json, &lookup).await.unwrap();
    assert!(
        stats.accepted >= 90,
        "expected most of gate2 accepted, got {}",
        stats.accepted
    );

    let hits = search::search_species(&db, "Acer").await.unwrap();
    assert!(!hits.is_empty(), "scientific search should find Acer");

    let common = search::search_species_by_common_name(&db, "boxelder")
        .await
        .unwrap();
    assert!(!common.is_empty(), "common name search should find boxelder");

    let conn = db.conn().await;
    let plants: i64 = conn
        .query_row("SELECT COUNT(*) FROM plants", [], |r| r.get(0))
        .unwrap();
    assert_eq!(plants, 0, "seed must not write L3 plants");
}
