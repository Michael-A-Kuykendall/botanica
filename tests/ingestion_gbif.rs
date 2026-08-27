#[cfg(feature = "ingestion")]
mod gbif_ingestion_tests {
    use botanica::{DatabaseConfig, BotanicalDatabase};
    use botanica::ingestion::gbif::{GbifClient, ingest_gbif_vernacular};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_gbif_ingest_vernacular() {
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

        // Mock GBIF API
        let server = MockServer::start();
        let gbif_id = "GBIF123";

        let body = serde_json::json!({
            "results": [
                {"vernacularName": "Test Plant", "language": "en", "isPreferredName": true, "source": "GBIF"},
                {"vernacularName": "Planta de Prueba", "language": "es", "isPreferredName": false, "source": "GBIF"}
            ]
        });

        server.mock(|when, then| {
            when.method(GET).path(format!("/v1/species/{}/vernacularNames", gbif_id));
            then.status(200).json_body(body);
        });

        let mut client = GbifClient::default();
        client.base_url = server.url("/v1");

        ingest_gbif_vernacular(&db, species_id, gbif_id, &client).await.unwrap();

        // Verify rows
        let conn = db.conn().await;
        let name_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM vernacular_names WHERE species_id = ?",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name_count, 2);
        let prov_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM provenance WHERE species_id = ? AND source = 'GBIF'",
                duckdb::params![species_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prov_count, 1);
    }
}
