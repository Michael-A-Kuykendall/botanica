//! Phase 2: schema harden, plants, identifiers, cultivars, quarantine, version

use crate::create_test_database;
use crate::migrations::{runner, schemas::SCHEMA_VERSION};
use crate::queries::{family, genus, species, plants, cultivars, identifiers};
use crate::types::{
    Family, Genus, Species, Plant, HealthStatus, Cultivar, SpeciesIdentifier,
};

#[tokio::test]
async fn test_schema_version_stamped() {
    let db = create_test_database().await.unwrap();
    let v = runner::check_schema_version(&db).await.unwrap();
    assert_eq!(v, SCHEMA_VERSION);
    assert!(runner::validate_migrations(&db).await.unwrap());
}

#[tokio::test]
async fn test_phase2_tables_exist() {
    let db = create_test_database().await.unwrap();
    let conn = db.conn().await;
    for table in [
        "species_identifiers",
        "cultivars",
        "ingest_quarantine",
        "schema_meta",
        "plants",
    ] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "missing table {}", table);
    }
}

#[tokio::test]
async fn test_nine_plants_different_health() {
    let db = create_test_database().await.unwrap();
    let fam = Family::new("Solanaceae".into(), "Juss.".into());
    family::insert_family(&db, &fam).await.unwrap();
    let gen = Genus::new(fam.id, "Solanum".into(), "L.".into());
    genus::insert_genus(&db, &gen).await.unwrap();
    let sp = Species::new(gen.id, "lycopersicum".into(), "L.".into(), Some(1753), None)
        .with_taxonomy(Some("Solanum lycopersicum".into()), "accepted", "species");
    species::insert_species(&db, &sp).await.unwrap();

    let statuses = [
        HealthStatus::Healthy,
        HealthStatus::Stressed,
        HealthStatus::Declining,
        HealthStatus::Dormant,
        HealthStatus::Unknown,
        HealthStatus::Healthy,
        HealthStatus::Stressed,
        HealthStatus::Healthy,
        HealthStatus::Dead,
    ];

    for (i, st) in statuses.iter().enumerate() {
        let p = Plant::new(format!("Tomato #{}", i + 1), Some(sp.id))
            .with_health(st.clone())
            .with_location(format!("bed {}", i % 3));
        plants::insert_plant(&db, &p).await.unwrap();
    }

    let list = plants::get_plants_by_species(&db, sp.id).await.unwrap();
    assert_eq!(list.len(), 9);
    assert!(list.iter().any(|p| p.health_status == HealthStatus::Dead));
}

#[tokio::test]
async fn test_cultivar_and_identifier() {
    let db = create_test_database().await.unwrap();
    let fam = Family::new("Rosaceae".into(), "Juss.".into());
    family::insert_family(&db, &fam).await.unwrap();
    let gen = Genus::new(fam.id, "Malus".into(), "Mill.".into());
    genus::insert_genus(&db, &gen).await.unwrap();
    let sp = Species::new(gen.id, "domestica".into(), "Borkh.".into(), None, None);
    species::insert_species(&db, &sp).await.unwrap();

    let cv = Cultivar::new(sp.id, "Honeycrisp".into())
        .with_trade_name("Honeycrisp")
        .with_source("demo");
    cultivars::insert_cultivar(&db, &cv).await.unwrap();
    let found = cultivars::get_cultivars_by_species(&db, sp.id).await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].cultivar_name, "Honeycrisp");

    let ident = SpeciesIdentifier::new(sp.id, "usda", "MADO4").primary();
    identifiers::insert_identifier(&db, &ident).await.unwrap();
    let sid = identifiers::find_species_by_external_id(&db, "usda", "MADO4")
        .await
        .unwrap();
    assert_eq!(sid, Some(sp.id));

    let plant = Plant::new("Orchard tree 1".into(), Some(sp.id)).with_cultivar(cv.id);
    plants::insert_plant(&db, &plant).await.unwrap();
    let got = plants::get_plant_by_id(&db, plant.id).await.unwrap().unwrap();
    assert_eq!(got.cultivar_id, Some(cv.id));
}

#[tokio::test]
async fn test_plant_update_sync_hooks() {
    let db = create_test_database().await.unwrap();
    let mut plant = Plant::new("Fern".into(), None)
        .with_sync_ids(Some("user-1".into()), Some("device-a".into()));
    plants::insert_plant(&db, &plant).await.unwrap();
    plant.health_status = HealthStatus::Stressed;
    plant.device_id = Some("device-b".into());
    assert!(plants::update_plant(&db, &plant).await.unwrap());
    let got = plants::get_plant_by_id(&db, plant.id).await.unwrap().unwrap();
    assert_eq!(got.health_status, HealthStatus::Stressed);
    assert_eq!(got.device_id.as_deref(), Some("device-b"));
}

#[cfg(feature = "ingestion")]
mod bulk_tests {
    use super::*;
    use crate::ingestion::bulk::{parse_binomial, bulk_ingest_cultivated};

    #[test]
    fn test_parse_binomial() {
        let (g, e) = parse_binomial("Rosa rubiginosa L.").unwrap();
        assert_eq!(g, "Rosa");
        assert_eq!(e, "rubiginosa");
        assert!(parse_binomial("Monotypic").is_none());
    }

    #[tokio::test]
    async fn test_bulk_demo_no_unknown_family() {
        let db = create_test_database().await.unwrap();
        bulk_ingest_cultivated(&db, Some(3), None).await.unwrap();
        let conn = db.conn().await;
        let unknown: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM families WHERE name = 'Unknown'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unknown, 0);
        let species_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM species", [], |r| r.get(0))
            .unwrap();
        assert!(species_count >= 1);
        let idents: i64 = conn
            .query_row("SELECT COUNT(*) FROM species_identifiers", [], |r| r.get(0))
            .unwrap();
        assert!(idents >= 1);
    }
}
