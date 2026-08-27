//! Species CRUD operation tests
//! 
//! Tests all Species operations including create, read, update, delete, and search functionality.

use super::{setup_test_database, setup_sample_taxonomy, assert_species_eq, create_test_species};
use crate::queries::species::*;
use crate::types::Species;
use uuid::Uuid;

#[tokio::test]
async fn test_insert_species() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let new_species = Species::new(
        genus.id,
        "gallica".to_string(),
        "Linnaeus".to_string(),
        Some(1753),
        Some("LC".to_string())
    );
    
    let result = insert_species(&db, &new_species).await;
    assert!(result.is_ok(), "Failed to insert species: {:?}", result.err());
}

#[tokio::test]
async fn test_get_species_by_id_existing() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let result = get_species_by_id(&db, species.id).await;
    assert!(result.is_ok(), "Failed to get species by id: {:?}", result.err());
    
    let found_species = result.unwrap();
    assert!(found_species.is_some(), "Species should be found");
    
    let found_species = found_species.unwrap();
    assert_species_eq(&species, &found_species);
}

#[tokio::test]
async fn test_get_species_by_id_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = get_species_by_id(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent id");
    
    let found_species = result.unwrap();
    assert!(found_species.is_none(), "No species should be found for nonexistent id");
}

#[tokio::test]
async fn test_get_species_by_name_exact_match() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let result = get_species_by_name(&db, "rubiginosa").await;
    assert!(result.is_ok(), "Failed to get species by name: {:?}", result.err());
    
    let found_species = result.unwrap();
    assert_eq!(found_species.len(), 1, "Should find exactly one species");
    assert_species_eq(&species, &found_species[0]);
}

#[tokio::test]
async fn test_get_species_by_name_partial_match() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Insert multiple species with similar names
    let species1 = Species::new(
        genus.id,
        "gallica".to_string(),
        "Linnaeus".to_string(),
        Some(1753),
        Some("LC".to_string())
    );
    let species2 = Species::new(
        genus.id,
        "canina".to_string(),
        "Linnaeus".to_string(),
        Some(1753),
        Some("LC".to_string())
    );
    
    insert_species(&db, &species1).await.expect("Failed to insert species1");
    insert_species(&db, &species2).await.expect("Failed to insert species2");
    
    let result = get_species_by_name(&db, "a").await;
    assert!(result.is_ok(), "Failed to get species by partial name: {:?}", result.err());
    
    let found_species = result.unwrap();
    assert!(found_species.len() >= 2, "Should find multiple species with 'a' in name: {:?}", found_species.len());
}

#[tokio::test]
async fn test_get_species_by_name_no_match() {
    let db = setup_test_database().await;
    setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let result = get_species_by_name(&db, "nonexistent").await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent name");
    
    let found_species = result.unwrap();
    assert!(found_species.is_empty(), "No species should be found for nonexistent name");
}

#[tokio::test]
async fn test_update_species_existing() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let mut updated_species = species.clone();
    updated_species.conservation_status = Some("NT".to_string());
    updated_species.publication_year = Some(1754);
    
    let result = update_species(&db, species.id, &updated_species).await;
    assert!(result.is_ok(), "Failed to update species: {:?}", result.err());
    assert!(result.unwrap(), "Update should return true for existing species");
    
    // Verify the update
    let retrieved = get_species_by_id(&db, species.id).await
        .expect("Failed to retrieve updated species")
        .expect("Updated species should exist");
    
    assert_eq!(retrieved.conservation_status, Some("NT".to_string()));
    assert_eq!(retrieved.publication_year, Some(1754));
}

#[tokio::test]
async fn test_update_species_nonexistent() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let nonexistent_id = Uuid::new_v4();
    let fake_species = create_test_species(genus.id);
    
    let result = update_species(&db, nonexistent_id, &fake_species).await;
    assert!(result.is_ok(), "Update query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Update should return false for nonexistent species");
}

#[tokio::test]
async fn test_delete_species_existing() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let result = delete_species(&db, species.id).await;
    assert!(result.is_ok(), "Failed to delete species: {:?}", result.err());
    assert!(result.unwrap(), "Delete should return true for existing species");
    
    // Verify the deletion
    let retrieved = get_species_by_id(&db, species.id).await
        .expect("Query should succeed after deletion");
    assert!(retrieved.is_none(), "Deleted species should not be found");
}

#[tokio::test]
async fn test_delete_species_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = delete_species(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Delete query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Delete should return false for nonexistent species");
}

#[tokio::test]
async fn test_species_foreign_key_constraint() {
    let db = setup_test_database().await;
    let fake_genus_id = Uuid::new_v4();
    
    let invalid_species = Species::new(
        fake_genus_id,
        "invalid".to_string(),
        "Test".to_string(),
        None,
        None
    );
    
    let result = insert_species(&db, &invalid_species).await;
    assert!(result.is_err(), "Insert should fail due to foreign key constraint");
}

#[tokio::test]
async fn test_species_data_integrity() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Test with empty specific epithet
    let species_empty_name = Species::new(
        genus.id,
        "".to_string(),
        "Test".to_string(),
        None,
        None
    );
    
    let result = insert_species(&db, &species_empty_name).await;
    assert!(result.is_ok(), "Insert should succeed with empty name");
    
    // Test with empty authority
    let species_empty_authority = Species::new(
        genus.id,
        "testspecies".to_string(),
        "".to_string(),
        None,
        None
    );
    
    let result = insert_species(&db, &species_empty_authority).await;
    assert!(result.is_ok(), "Insert should succeed with empty authority");
}

#[tokio::test]
async fn test_species_with_optional_fields() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Test species with no publication year or conservation status
    let minimal_species = Species::new(
        genus.id,
        "minimalis".to_string(),
        "Tester".to_string(),
        None,
        None
    );
    
    let result = insert_species(&db, &minimal_species).await;
    assert!(result.is_ok(), "Failed to insert minimal species: {:?}", result.err());
    
    let retrieved = get_species_by_id(&db, minimal_species.id).await
        .expect("Failed to retrieve minimal species")
        .expect("Minimal species should exist");
    
    assert_eq!(retrieved.publication_year, None);
    assert_eq!(retrieved.conservation_status, None);
}

#[tokio::test]
async fn test_species_with_negative_publication_year() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Test species with negative publication year (BCE)
    let ancient_species = Species::new(
        genus.id,
        "ancientus".to_string(),
        "Fossil".to_string(),
        Some(-1000),
        Some("EX".to_string())
    );
    
    let result = insert_species(&db, &ancient_species).await;
    assert!(result.is_ok(), "Failed to insert ancient species: {:?}", result.err());
    
    let retrieved = get_species_by_id(&db, ancient_species.id).await
        .expect("Failed to retrieve ancient species")
        .expect("Ancient species should exist");
    
    assert_eq!(retrieved.publication_year, Some(-1000));
}

#[tokio::test]
async fn test_species_methods() {
    let genus_id = Uuid::new_v4();
    let species = Species::new(
        genus_id,
        "testspecies".to_string(),
        "Test Author".to_string(),
        Some(2023),
        Some("VU".to_string())
    );
    
    assert_eq!(species.get_specific_epithet(), "testspecies");
    assert_eq!(species.get_authority(), "Test Author");
    assert_eq!(species.get_publication_year(), Some(2023));
    assert_eq!(species.get_conservation_status(), Some("VU"));
    assert!(species.has_conservation_status());
    
    let mut species_no_status = Species::new(
        genus_id,
        "nostatus".to_string(),
        "Test".to_string(),
        None,
        None
    );
    
    assert!(!species_no_status.has_conservation_status());
    
    species_no_status.set_conservation_status(Some("CR".to_string()));
    assert!(species_no_status.has_conservation_status());
    assert_eq!(species_no_status.get_conservation_status(), Some("CR"));
}
