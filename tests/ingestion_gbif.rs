#[cfg(feature = "ingestion")]
mod gbif_ingestion_tests {
    use botanica::{DatabaseConfig, BotanicalDatabase};
    use botanica::ingestion::gbif::{GbifClient, ingest_gbif_vernacular};
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_gbif_ingest_vernacular() {
        let db = BotanicalDatabase::new(DatabaseConfig::memory()).await.unwrap();
        db.migrate().await.unwrap();
        let pool = db.pool();

        // Insert minimal taxonomy
        let family_id = "fam1";
        sqlx::query("INSERT INTO families (id, name, authority) VALUES (?1,'TestFam',NULL)")
            .bind(family_id)
            .execute(pool).await.unwrap();
        let genus_id = "gen1";
        sqlx::query("INSERT INTO genera (id, family_id, name, authority) VALUES (?1, ?2,'TestGen',NULL)")
            .bind(genus_id).bind(family_id).execute(pool).await.unwrap();
        let species_id = "sp1";
        sqlx::query("INSERT INTO species (id, genus_id, specific_epithet, authority) VALUES (?1, ?2,'speciosa','Auth')")
            .bind(species_id).bind(genus_id).execute(pool).await.unwrap();

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

        ingest_gbif_vernacular(pool, species_id, gbif_id, &client).await.unwrap();

        // Verify rows
        let name_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vernacular_names WHERE species_id = ?1")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(name_count.0, 2);
        let prov_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM provenance WHERE species_id = ?1 AND source='GBIF'")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(prov_count.0, 1);
    }
}
