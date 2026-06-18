#[cfg(feature = "ingestion")]
mod powo_ingestion_tests {
    use botanica::{DatabaseConfig, BotanicalDatabase};
    use botanica::ingestion::powo::{PowoClient, ingest_powo_for_species};
    use httpmock::prelude::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_powo_ingest_basic() {
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

        ingest_powo_for_species(pool, species_id, powo_id, &client).await.unwrap();

        // Verify rows
        let syn_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM synonyms WHERE species_id = ?1")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(syn_count.0, 1);
        let dist_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM distribution_regions WHERE species_id = ?1")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(dist_count.0, 1);
        let uses_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM uses WHERE species_id = ?1")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(uses_count.0, 1);
        let prov_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM provenance WHERE species_id = ?1 AND source='POWO'")
            .bind(species_id).fetch_one(pool).await.unwrap();
        assert_eq!(prov_count.0, 1);
    }
}
