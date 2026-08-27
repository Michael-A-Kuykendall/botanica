#[cfg(feature = "ingestion")]
mod powo_ingestion_tests {
    use botanica::{DatabaseConfig, BotanicalDatabase};
    use botanica::ingestion::powo::{PowoClient, ingest_powo_for_species};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_powo_ingest_basic() {
        let db = BotanicalDatabase::new(DatabaseConfig::memory()).await.unwrap();
        db.migrate().await.unwrap();

        let family_id = "fam1";
        let genus_id = "gen1";
        let species_id = "sp1";

        // Insert minimal taxonomy via the duckdb connection
        {
            let conn = db.conn().await;
            conn.execute(
                "INSERT INTO families (id, name, authority) VALUES (?, 'TestFam', NULL)",
                duckdb::params![family_id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO genera (id, family_id, name, authority) VALUES (?, ?, 'TestGen', NULL)",
                duckdb::params![genus_id, family_id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO species (id, genus_id, specific_epithet, authority) VALUES (?, ?, 'speciosa', 'Auth')",
                duckdb::params![species_id, genus_id],
            )
            .unwrap();
        }

        // Mock POWO API
        let server = MockServer::start();
        let powo_id = "POWO123";

        let body = serde_json::json!({
            "name": "TestGen speciosa",
            "authorship": "Auth",
            "synonyms": [ {"name": "AltName speciosa", "authorship": "AltAuth", "id": "SYN1"} ],
            "distribution": [ {"region_code": "AFR", "source": "WGSRPD"} ],
            "uses": [ {"category": "medicinal", "description": "Used for tests"} ]
        });

        server.mock(|when, then| {
            when.method(GET).path(format!("/api/2/taxon/{}", powo_id));
            then.status(200).json_body(body);
        });

        let mut client = PowoClient::default();
        client.base_url = server.url("/api/2");

        ingest_powo_for_species(&db, species_id, powo_id, &client).await.unwrap();

        // Verify rows
        let conn = db.conn().await;
        let syn_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM synonyms WHERE species_id = ?",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(syn_count, 1);
        let dist_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM distribution_regions WHERE species_id = ?",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(dist_count, 1);
        let uses_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM uses WHERE species_id = ?",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(uses_count, 1);
        let prov_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM provenance WHERE species_id = ? AND source = 'POWO'",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prov_count, 1);
    }
}
