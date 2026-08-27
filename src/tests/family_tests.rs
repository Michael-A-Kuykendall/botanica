//! Family CRUD operation tests
//! 
//! Tests all Family operations including create, read, update, delete, and search functionality.

use super::{setup_test_database, assert_family_eq, create_test_family};
use crate::queries::family::*;
use crate::types::Family;
use uuid::Uuid;

#[tokio::test]
async fn test_insert_family() {
    let db = setup_test_database().await;
    
    let family = Family::new(
        "Cannabaceae".to_string(),
        "Martius".to_string()
    );
    
    let result = insert_family(&db, &family).await;
    assert!(result.is_ok(), "Failed to insert family: {:?}", result.err());
}

#[tokio::test]
async fn test_get_family_by_id_existing() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let result = get_family_by_id(&db, family.id).await;
    assert!(result.is_ok(), "Failed to get family by id: {:?}", result.err());
    
    let found_family = result.unwrap();
    assert!(found_family.is_some(), "Family should be found");
    
    let found_family = found_family.unwrap();
    assert_family_eq(&family, &found_family);
}

#[tokio::test]
async fn test_get_family_by_id_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = get_family_by_id(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent id");
    
    let found_family = result.unwrap();
    assert!(found_family.is_none(), "No family should be found for nonexistent id");
}

#[tokio::test]
async fn test_get_families_by_name_exact_match() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let result = get_families_by_name(&db, "Rosaceae").await;
    assert!(result.is_ok(), "Failed to get families by name: {:?}", result.err());
    
    let found_families = result.unwrap();
    assert_eq!(found_families.len(), 1, "Should find exactly one family");
    assert_family_eq(&family, &found_families[0]);
}

#[tokio::test]
async fn test_get_families_by_name_partial_match() {
    let db = setup_test_database().await;
    
    // Insert multiple families with similar names
    let family1 = Family::new("Rosaceae".to_string(), "Jussieu".to_string());
    let family2 = Family::new("Brassicaceae".to_string(), "Burnett".to_string());
    let family3 = Family::new("Poaceae".to_string(), "Barnhart".to_string());
    
    insert_family(&db, &family1).await.expect("Failed to insert family1");
    insert_family(&db, &family2).await.expect("Failed to insert family2");
    insert_family(&db, &family3).await.expect("Failed to insert family3");
    
    let result = get_families_by_name(&db, "aceae").await;
    assert!(result.is_ok(), "Failed to get families by partial name: {:?}", result.err());
    
    let found_families = result.unwrap();
    assert_eq!(found_families.len(), 3, "Should find all three families");
    
    // Results should be ordered by name
    assert_eq!(found_families[0].name, "Brassicaceae");
    assert_eq!(found_families[1].name, "Poaceae");
    assert_eq!(found_families[2].name, "Rosaceae");
}

#[tokio::test]
async fn test_get_families_by_name_case_sensitivity() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    // Search with different case - DuckDB ILIKE is case-insensitive
    let result = get_families_by_name(&db, "rosaceae").await;
    assert!(result.is_ok(), "Failed to get families by lowercase name: {:?}", result.err());
    
    let found_families = result.unwrap();
    assert_eq!(found_families.len(), 1, "Should find family with case-insensitive search");
}

#[tokio::test]
async fn test_get_families_by_name_no_match() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let result = get_families_by_name(&db, "nonexistent").await;
    assert!(result.is_ok(), "Query should succeed even for nonexistent name");
    
    let found_families = result.unwrap();
    assert!(found_families.is_empty(), "No families should be found for nonexistent name");
}

#[tokio::test]
async fn test_update_family_existing() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let mut updated_family = family.clone();
    updated_family.name = "Updated_Rosaceae".to_string();
    updated_family.authority = "Updated Authority".to_string();
    
    let result = update_family(&db, family.id, &updated_family).await;
    assert!(result.is_ok(), "Failed to update family: {:?}", result.err());
    assert!(result.unwrap(), "Update should return true for existing family");
    
    // Verify the update
    let retrieved = get_family_by_id(&db, family.id).await
        .expect("Failed to retrieve updated family")
        .expect("Updated family should exist");
    
    assert_eq!(retrieved.name, "Updated_Rosaceae");
    assert_eq!(retrieved.authority, "Updated Authority");
    assert_eq!(retrieved.id, family.id); // ID should remain the same
}

#[tokio::test]
async fn test_update_family_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    let fake_family = create_test_family();
    
    let result = update_family(&db, nonexistent_id, &fake_family).await;
    assert!(result.is_ok(), "Update query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Update should return false for nonexistent family");
}

#[tokio::test]
async fn test_delete_family_existing() {
    let db = setup_test_database().await;
    let family = create_test_family();
    
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let result = delete_family(&db, family.id).await;
    assert!(result.is_ok(), "Failed to delete family: {:?}", result.err());
    assert!(result.unwrap(), "Delete should return true for existing family");
    
    // Verify the deletion
    let retrieved = get_family_by_id(&db, family.id).await
        .expect("Query should succeed after deletion");
    assert!(retrieved.is_none(), "Deleted family should not be found");
}

#[tokio::test]
async fn test_delete_family_nonexistent() {
    let db = setup_test_database().await;
    let nonexistent_id = Uuid::new_v4();
    
    let result = delete_family(&db, nonexistent_id).await;
    assert!(result.is_ok(), "Delete query should succeed even for nonexistent id");
    assert!(!result.unwrap(), "Delete should return false for nonexistent family");
}

#[tokio::test]
async fn test_family_data_integrity() {
    let db = setup_test_database().await;
    
    // Test with empty name
    let family_empty_name = Family::new("".to_string(), "Test".to_string());
    let result = insert_family(&db, &family_empty_name).await;
    assert!(result.is_ok(), "Insert should succeed with empty name");
    
    // Test with empty authority
    let family_empty_authority = Family::new("TestFamily".to_string(), "".to_string());
    let result = insert_family(&db, &family_empty_authority).await;
    assert!(result.is_ok(), "Insert should succeed with empty authority");
    
    // Test with both empty (edge case)
    let family_empty_both = Family::new("".to_string(), "".to_string());
    let result = insert_family(&db, &family_empty_both).await;
    assert!(result.is_ok(), "Insert should succeed with both fields empty");
}

#[tokio::test]
async fn test_family_creation_methods() {
    // Test new() method
    let family1 = Family::new(
        "TestFamily".to_string(),
        "Test Authority".to_string()
    );
    
    assert_eq!(family1.name, "TestFamily");
    assert_eq!(family1.authority, "Test Authority");
    assert_ne!(family1.id, Uuid::nil());
    
    // Test with_id() method
    let specific_id = Uuid::new_v4();
    let family2 = Family::with_id(
        specific_id,
        "AnotherFamily".to_string(),
        "Another Authority".to_string()
    );
    
    assert_eq!(family2.id, specific_id);
    assert_eq!(family2.name, "AnotherFamily");
    assert_eq!(family2.authority, "Another Authority");
}

#[tokio::test]
async fn test_multiple_families_same_name() {
    let db = setup_test_database().await;
    
    // Create two families with same name but different authorities
    let family1 = Family::new("SameName".to_string(), "Authority1".to_string());
    let family2 = Family::new("SameName".to_string(), "Authority2".to_string());
    
    let result1 = insert_family(&db, &family1).await;
    assert!(result1.is_ok(), "Failed to insert family1: {:?}", result1.err());
    
    let result2 = insert_family(&db, &family2).await;
    assert!(result2.is_ok(), "Failed to insert family2: {:?}", result2.err());
    
    // Both should be retrievable by name search
    let found_families = get_families_by_name(&db, "SameName").await
        .expect("Failed to search for families");
    assert_eq!(found_families.len(), 2, "Should find both families with same name");
    
    // Both should be retrievable by ID
    let retrieved1 = get_family_by_id(&db, family1.id).await
        .expect("Failed to retrieve family1")
        .expect("Family1 should exist");
        
    let retrieved2 = get_family_by_id(&db, family2.id).await
        .expect("Failed to retrieve family2")
        .expect("Family2 should exist");
    
    assert_eq!(retrieved1.authority, "Authority1");
    assert_eq!(retrieved2.authority, "Authority2");
}

#[tokio::test]
async fn test_family_unicode_names() {
    let db = setup_test_database().await;
    
    // Test with Unicode characters
    let unicode_family = Family::new(
        "试验科".to_string(), // Chinese characters
        "Tëst Authör".to_string() // Accented characters
    );
    
    let result = insert_family(&db, &unicode_family).await;
    assert!(result.is_ok(), "Failed to insert Unicode family: {:?}", result.err());
    
    let retrieved = get_family_by_id(&db, unicode_family.id).await
        .expect("Failed to retrieve Unicode family")
        .expect("Unicode family should exist");
    
    assert_eq!(retrieved.name, "试验科");
    assert_eq!(retrieved.authority, "Tëst Authör");
}

#[tokio::test]
async fn test_family_long_names() {
    let db = setup_test_database().await;
    
    // Test with very long names
    let long_name = "A".repeat(1000);
    let long_authority = "B".repeat(1000);
    
    let long_family = Family::new(long_name.clone(), long_authority.clone());
    
    let result = insert_family(&db, &long_family).await;
    assert!(result.is_ok(), "Failed to insert family with long names: {:?}", result.err());
    
    let retrieved = get_family_by_id(&db, long_family.id).await
        .expect("Failed to retrieve long name family")
        .expect("Long name family should exist");
    
    assert_eq!(retrieved.name.len(), 1000);
    assert_eq!(retrieved.authority.len(), 1000);
}
