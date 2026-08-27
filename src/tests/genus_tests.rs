//! Genus CRUD operation tests
//! 
//! Tests all Genus operations including create, read, update, delete, and relationship functionality.

use super::{setup_test_database, setup_sample_taxonomy, assert_genus_eq, create_test_genus};
use crate::queries::genus::*;
use crate::queries::family::insert_family;
use crate::types::{Family, Genus};
use uuid::Uuid;

#[tokio::test]
async fn test_insert_genus() {
    let db = setup_test_database().await;
    let family = Family::new("Poaceae".to_string(), "Barnhart".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let genus = Genus::new(
        family.id,
        "Triticum".to_string(),
        "Linnaeus".to_string()
    );
    
    let result = insert_genus(&db, &genus).await;
    assert!(result.is_ok(), "Failed to insert genus: {:?}", result.err());
}

#[tokio::test]
async fn test_get_genus_by_id_existing() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let result = get_genus_by_id(&db, genus.id).await;
    assert!(result.is_ok(), "Failed to get genus by id: {:?}", result.err());
    
    let found_genus = result.unwrap();
    assert!(found_genus.is_some(), "Genus should be found");
    
    let found_genus = found_genus.unwrap();
    assert_genus_eq(&genus, &found_genus);
}

#[tokio::test]
async fn test_get_genus_by_id_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = get_genus_by_id(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent id");
    
    let found_genus = result.unwrap();
    assert!(found_genus.is_none(), "No genus should be found for nonexistent id");
}

#[tokio::test]
async fn test_get_genera_by_family_id() {
    let db = setup_test_database().await;
    let family = Family::new("Rosaceae".to_string(), "Jussieu".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    // Insert multiple genera in the same family
    let genus1 = Genus::new(family.id, "Rosa".to_string(), "Linnaeus".to_string());
    let genus2 = Genus::new(family.id, "Prunus".to_string(), "Linnaeus".to_string());
    let genus3 = Genus::new(family.id, "Malus".to_string(), "Miller".to_string());
    
    insert_genus(&db, &genus1).await.expect("Failed to insert genus1");
    insert_genus(&db, &genus2).await.expect("Failed to insert genus2");
    insert_genus(&db, &genus3).await.expect("Failed to insert genus3");
    
    let result = get_genera_by_family_id(&db, family.id).await;
    assert!(result.is_ok(), "Failed to get genera by family id: {:?}", result.err());
    
    let found_genera = result.unwrap();
    assert_eq!(found_genera.len(), 3, "Should find exactly 3 genera");
    
    // Results should be ordered by name
    assert_eq!(found_genera[0].name, "Malus");
    assert_eq!(found_genera[1].name, "Prunus");
    assert_eq!(found_genera[2].name, "Rosa");
}

#[tokio::test]
async fn test_get_genera_by_family_id_empty() {
    let db = setup_test_database().await;
    let family = Family::new("Emptyaceae".to_string(), "Test".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let result = get_genera_by_family_id(&db, family.id).await;
    assert!(result.is_ok(), "Query should succeed for family with no genera");
    
    let found_genera = result.unwrap();
    assert!(found_genera.is_empty(), "Should find no genera for empty family");
}

#[tokio::test]
async fn test_get_genera_by_family_id_nonexistent_family() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = get_genera_by_family_id(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent family id");
    
    let found_genera = result.unwrap();
    assert!(found_genera.is_empty(), "Should find no genera for nonexistent family");
}

#[tokio::test]
async fn test_update_genus_existing() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    use crate::queries::species::delete_species;
    delete_species(&db, species.id).await.expect("Failed to delete dependent species");
    
    let mut updated_genus = genus.clone();
    updated_genus.name = "Updated_Rosa".to_string();
    updated_genus.authority = "Updated Authority".to_string();
    
    let result = update_genus(&db, genus.id, &updated_genus).await;
    assert!(result.is_ok(), "Failed to update genus: {:?}", result.err());
    assert!(result.unwrap(), "Update should return true for existing genus");
    
    // Verify the update
    let retrieved = get_genus_by_id(&db, genus.id).await
        .expect("Failed to retrieve updated genus")
        .expect("Updated genus should exist");
    
    assert_eq!(retrieved.name, "Updated_Rosa");
    assert_eq!(retrieved.authority, "Updated Authority");
}

#[tokio::test]
async fn test_update_genus_nonexistent() {
    let db = setup_test_database().await;
    let (family, _, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    let nonexistent_id = Uuid::new_v4();
    let fake_genus = create_test_genus(family.id);
    
    let result = update_genus(&db, nonexistent_id, &fake_genus).await;
    assert!(result.is_ok(), "Update query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Update should return false for nonexistent genus");
}

#[tokio::test]
async fn test_update_genus_change_family() {
    let db = setup_test_database().await;
    let (family1, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    use crate::queries::species::delete_species;
    delete_species(&db, species.id).await.expect("Failed to delete dependent species");
    
    // Create a second family
    let family2 = Family::new("Poaceae".to_string(), "Barnhart".to_string());
    insert_family(&db, &family2).await.expect("Failed to insert second family");
    
    let mut updated_genus = genus.clone();
    updated_genus.family_id = family2.id;
    
    let result = update_genus(&db, genus.id, &updated_genus).await;
    assert!(result.is_ok(), "Failed to update genus family: {:?}", result.err());
    assert!(result.unwrap(), "Update should return true");
    
    // Verify the family change
    let retrieved = get_genus_by_id(&db, genus.id).await
        .expect("Failed to retrieve updated genus")
        .expect("Updated genus should exist");
    
    assert_eq!(retrieved.family_id, family2.id);
}

#[tokio::test]
async fn test_delete_genus_existing() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // First delete the dependent species
    use crate::queries::species::delete_species;
    let species_delete = delete_species(&db, species.id).await;
    assert!(species_delete.is_ok() && species_delete.unwrap(), "Failed to delete dependent species");
    
    // Now delete the genus
    let result = delete_genus(&db, genus.id).await;
    assert!(result.is_ok(), "Failed to delete genus: {:?}", result.err());
    assert!(result.unwrap(), "Delete should return true for existing genus");
    
    // Verify the deletion
    let retrieved = get_genus_by_id(&db, genus.id).await
        .expect("Query should succeed after deletion");
    assert!(retrieved.is_none(), "Deleted genus should not be found");
}

#[tokio::test]
async fn test_delete_genus_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = delete_genus(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Delete query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Delete should return false for nonexistent genus");
}

#[tokio::test]
async fn test_genus_foreign_key_constraint() {
    let db = setup_test_database().await;
    let fake_family_id = Uuid::new_v4();
    
    let invalid_genus = Genus::new(
        fake_family_id,
        "InvalidGenus".to_string(),
        "Test".to_string()
    );
    
    let result = insert_genus(&db, &invalid_genus).await;
    assert!(result.is_err(), "Insert should fail due to foreign key constraint");
}

#[tokio::test]
async fn test_genus_data_integrity() {
    let db = setup_test_database().await;
    let family = Family::new("TestFamily".to_string(), "Test".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    // Test with empty name
    let genus_empty_name = Genus::new(
        family.id,
        "".to_string(),
        "Test".to_string()
    );
    
    let result = insert_genus(&db, &genus_empty_name).await;
    assert!(result.is_ok(), "Insert should succeed with empty name");
    
    // Test with empty authority
    let genus_empty_authority = Genus::new(
        family.id,
        "TestGenus".to_string(),
        "".to_string()
    );
    
    let result = insert_genus(&db, &genus_empty_authority).await;
    assert!(result.is_ok(), "Insert should succeed with empty authority");
}

#[tokio::test]
async fn test_genus_creation_methods() {
    let family_id = Uuid::new_v4();
    
    // Test new() method
    let genus1 = Genus::new(
        family_id,
        "TestGenus".to_string(),
        "Test Authority".to_string()
    );
    
    assert_eq!(genus1.family_id, family_id);
    assert_eq!(genus1.name, "TestGenus");
    assert_eq!(genus1.authority, "Test Authority");
    assert_ne!(genus1.id, Uuid::nil());
    
    // Test with_id() method
    let specific_id = Uuid::new_v4();
    let genus2 = Genus::with_id(
        specific_id,
        family_id,
        "AnotherGenus".to_string(),
        "Another Authority".to_string()
    );
    
    assert_eq!(genus2.id, specific_id);
    assert_eq!(genus2.family_id, family_id);
    assert_eq!(genus2.name, "AnotherGenus");
    assert_eq!(genus2.authority, "Another Authority");
}

#[tokio::test]
async fn test_multiple_genera_same_name_different_families() {
    let db = setup_test_database().await;
    
    // Create two different families
    let family1 = Family::new("Family1".to_string(), "Author1".to_string());
    let family2 = Family::new("Family2".to_string(), "Author2".to_string());
    
    insert_family(&db, &family1).await.expect("Failed to insert family1");
    insert_family(&db, &family2).await.expect("Failed to insert family2");
    
    // Create genera with same name but in different families
    let genus1 = Genus::new(family1.id, "SameName".to_string(), "Auth1".to_string());
    let genus2 = Genus::new(family2.id, "SameName".to_string(), "Auth2".to_string());
    
    let result1 = insert_genus(&db, &genus1).await;
    assert!(result1.is_ok(), "Failed to insert genus1: {:?}", result1.err());
    
    let result2 = insert_genus(&db, &genus2).await;
    assert!(result2.is_ok(), "Failed to insert genus2: {:?}", result2.err());
    
    // Both should be retrievable
    let retrieved1 = get_genus_by_id(&db, genus1.id).await
        .expect("Failed to retrieve genus1")
        .expect("Genus1 should exist");
        
    let retrieved2 = get_genus_by_id(&db, genus2.id).await
        .expect("Failed to retrieve genus2")
        .expect("Genus2 should exist");
    
    assert_eq!(retrieved1.family_id, family1.id);
    assert_eq!(retrieved2.family_id, family2.id);
    assert_eq!(retrieved1.name, retrieved2.name);
}
