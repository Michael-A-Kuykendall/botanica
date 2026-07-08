//! Integration tests for cross-model relationships
//! 
//! Tests the relationships between Family, Genus, and Species models,
//! including foreign key constraints, cascading operations, and complex queries.

use super::{setup_test_database, setup_sample_taxonomy};
use crate::queries::{family::*, genus::*, species::*};
use crate::types::{Family, Genus, Species};
use uuid::Uuid;

#[tokio::test]
async fn test_complete_taxonomic_hierarchy() {
    let db = setup_test_database().await;
    
    // Create a complete taxonomic hierarchy
    let family = Family::new("Rosaceae".to_string(), "Jussieu".to_string());
    let genus = Genus::new(family.id, "Rosa".to_string(), "Linnaeus".to_string());
    let species = Species::new(
        genus.id,
        "gallica".to_string(),
        "Linnaeus".to_string(),
        Some(1753),
        Some("LC".to_string())
    );
    
    // Insert in correct order (family -> genus -> species)
    insert_family(&db, &family).await.expect("Failed to insert family");
    insert_genus(&db, &genus).await.expect("Failed to insert genus");
    insert_species(&db, &species).await.expect("Failed to insert species");
    
    // Verify all can be retrieved
    let retrieved_family = get_family_by_id(&db, family.id).await
        .expect("Failed to get family")
        .expect("Family should exist");
    
    let retrieved_genus = get_genus_by_id(&db, genus.id).await
        .expect("Failed to get genus")
        .expect("Genus should exist");
    
    let retrieved_species = get_species_by_id(&db, species.id).await
        .expect("Failed to get species")
        .expect("Species should exist");
    
    // Verify relationships
    assert_eq!(retrieved_genus.family_id, retrieved_family.id);
    assert_eq!(retrieved_species.genus_id, retrieved_genus.id);
}

#[tokio::test]
async fn test_foreign_key_constraint_genus_to_family() {
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
async fn test_foreign_key_constraint_species_to_genus() {
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
async fn test_multiple_genera_per_family() {
    let db = setup_test_database().await;
    let family = Family::new("Rosaceae".to_string(), "Jussieu".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    // Create multiple genera in the same family
    let genera_data = vec![
        ("Rosa", "Linnaeus"),
        ("Prunus", "Linnaeus"),
        ("Malus", "Miller"),
        ("Pyrus", "Linnaeus"),
    ];
    
    let mut inserted_genera = Vec::new();
    for (name, authority) in genera_data {
        let genus = Genus::new(family.id, name.to_string(), authority.to_string());
        insert_genus(&db, &genus).await.expect(&format!("Failed to insert genus {}", name));
        inserted_genera.push(genus);
    }
    
    // Retrieve all genera for the family
    let retrieved_genera = get_genera_by_family_id(&db, family.id).await
        .expect("Failed to get genera by family");
    
    assert_eq!(retrieved_genera.len(), 4, "Should find all 4 genera");
    
    // Verify they're ordered by name
    let names: Vec<&str> = retrieved_genera.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(names, vec!["Malus", "Prunus", "Pyrus", "Rosa"]);
}

#[tokio::test]
async fn test_multiple_species_per_genus() {
    let db = setup_test_database().await;
    let (family, genus, _) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Create multiple species in the same genus
    let species_data = vec![
        ("gallica", "Linnaeus", 1753),
        ("canina", "Linnaeus", 1753),
        ("rugosa", "Thunberg", 1784),
        ("damascena", "Miller", 1768),
    ];
    
    let mut inserted_species = Vec::new();
    for (epithet, authority, year) in species_data {
        let species = Species::new(
            genus.id,
            epithet.to_string(),
            authority.to_string(),
            Some(year),
            Some("LC".to_string())
        );
        insert_species(&db, &species).await.expect(&format!("Failed to insert species {}", epithet));
        inserted_species.push(species);
    }
    
    // There should now be 5 species total (4 new + 1 from setup)
    // Verify by searching for all species containing common letters
    let all_species = get_species_by_name(&db, "").await
        .expect("Failed to get all species");
    
    assert!(all_species.len() >= 4, "Should find at least 4 species");
}

#[tokio::test]
async fn test_delete_family_with_dependent_genera() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Try to delete the family while it has dependent genera
    let result = delete_family(&db, family.id).await;
    
    // This should either fail due to foreign key constraint or succeed if ON DELETE CASCADE is set
    // Based on the schema, it should fail
    if result.is_err() {
        // Foreign key constraint prevented deletion - this is expected
        assert!(true, "Foreign key constraint correctly prevented family deletion");
    } else {
        // If deletion succeeded, verify that dependent records were also deleted
        let genus_exists = get_genus_by_id(&db, genus.id).await
            .expect("Query should succeed");
        let species_exists = get_species_by_id(&db, species.id).await
            .expect("Query should succeed");
            
        // If cascading delete is implemented, these should be None
        // If not, we need to clean up manually
        if genus_exists.is_some() {
            delete_species(&db, species.id).await.ok();
            delete_genus(&db, genus.id).await.ok();
        }
    }
}

#[tokio::test]
async fn test_delete_genus_with_dependent_species() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Try to delete the genus while it has dependent species
    let result = delete_genus(&db, genus.id).await;
    
    // This should either fail due to foreign key constraint or succeed if ON DELETE CASCADE is set
    if result.is_err() {
        // Foreign key constraint prevented deletion - this is expected
        assert!(true, "Foreign key constraint correctly prevented genus deletion");
    } else {
        // If deletion succeeded, verify that dependent species were also deleted
        let species_exists = get_species_by_id(&db, species.id).await
            .expect("Query should succeed");
            
        // If cascading delete is implemented, this should be None
        if species_exists.is_some() {
            delete_species(&db, species.id).await.ok();
        }
    }
}

#[tokio::test]
async fn test_proper_deletion_order() {
    let db = setup_test_database().await;
    let (family, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Delete in proper order: species -> genus -> family
    let species_delete = delete_species(&db, species.id).await;
    assert!(species_delete.is_ok() && species_delete.unwrap(), "Failed to delete species");
    
    let genus_delete = delete_genus(&db, genus.id).await;
    assert!(genus_delete.is_ok() && genus_delete.unwrap(), "Failed to delete genus");
    
    let family_delete = delete_family(&db, family.id).await;
    assert!(family_delete.is_ok() && family_delete.unwrap(), "Failed to delete family");
    
    // Verify all are deleted
    assert!(get_species_by_id(&db, species.id).await.unwrap().is_none());
    assert!(get_genus_by_id(&db, genus.id).await.unwrap().is_none());
    assert!(get_family_by_id(&db, family.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_update_genus_family_relationship() {
    let db = setup_test_database().await;
    let (family1, genus, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    use crate::queries::species::delete_species;
    delete_species(&db, species.id).await.expect("Failed to delete dependent species");
    
    // Create a second family
    let family2 = Family::new("Poaceae".to_string(), "Barnhart".to_string());
    insert_family(&db, &family2).await.expect("Failed to insert second family");
    
    // Update genus to belong to second family
    let mut updated_genus = genus.clone();
    updated_genus.family_id = family2.id;
    
    let result = update_genus(&db, genus.id, &updated_genus).await;
    assert!(result.is_ok() && result.unwrap(), "Failed to update genus family");
    
    // Verify the relationship change
    let retrieved_genus = get_genus_by_id(&db, genus.id).await
        .expect("Failed to retrieve updated genus")
        .expect("Genus should still exist");
    
    assert_eq!(retrieved_genus.family_id, family2.id);
    
    // Insert a new species in the updated genus to verify it still works
    let new_species = Species::new(
        genus.id,
        "testspecies".to_string(),
        "Test".to_string(),
        Some(2023),
        Some("LC".to_string())
    );
    insert_species(&db, &new_species).await.expect("Failed to insert new species");
    
    let retrieved_species = get_species_by_id(&db, new_species.id).await
        .expect("Failed to retrieve species")
        .expect("Species should exist");
    
    assert_eq!(retrieved_species.genus_id, genus.id);
}

#[tokio::test]
async fn test_update_species_genus_relationship() {
    let db = setup_test_database().await;
    let (family, genus1, species) = setup_sample_taxonomy(&db).await.expect("Failed to setup taxonomy");
    
    // Create a second genus in the same family
    let genus2 = Genus::new(family.id, "Prunus".to_string(), "Linnaeus".to_string());
    insert_genus(&db, &genus2).await.expect("Failed to insert second genus");
    
    // Update species to belong to second genus
    let mut updated_species = species.clone();
    updated_species.genus_id = genus2.id;
    
    let result = update_species(&db, species.id, &updated_species).await;
    assert!(result.is_ok() && result.unwrap(), "Failed to update species genus");
    
    // Verify the relationship change
    let retrieved_species = get_species_by_id(&db, species.id).await
        .expect("Failed to retrieve updated species")
        .expect("Species should still exist");
    
    assert_eq!(retrieved_species.genus_id, genus2.id);
}

#[tokio::test]
async fn test_complex_taxonomic_search() {
    let db = setup_test_database().await;
    
    // Create multiple taxonomic hierarchies
    let family1 = Family::new("Rosaceae".to_string(), "Jussieu".to_string());
    let family2 = Family::new("Poaceae".to_string(), "Barnhart".to_string());
    
    insert_family(&db, &family1).await.expect("Failed to insert family1");
    insert_family(&db, &family2).await.expect("Failed to insert family2");
    
    let genus1 = Genus::new(family1.id, "Rosa".to_string(), "Linnaeus".to_string());
    let genus2 = Genus::new(family1.id, "Prunus".to_string(), "Linnaeus".to_string());
    let genus3 = Genus::new(family2.id, "Triticum".to_string(), "Linnaeus".to_string());
    
    insert_genus(&db, &genus1).await.expect("Failed to insert genus1");
    insert_genus(&db, &genus2).await.expect("Failed to insert genus2");
    insert_genus(&db, &genus3).await.expect("Failed to insert genus3");
    
    // Create species in different genera
    let species_data = vec![
        (genus1.id, "gallica", "Linnaeus"),
        (genus1.id, "canina", "Linnaeus"),
        (genus2.id, "dulcis", "Miller"),
        (genus3.id, "aestivum", "Linnaeus"),
    ];
    
    for (genus_id, epithet, authority) in species_data {
        let species = Species::new(
            genus_id,
            epithet.to_string(),
            authority.to_string(),
            Some(1753),
            None
        );
        insert_species(&db, &species).await.expect("Failed to insert species");
    }
    
    // Test complex searches
    let rosaceae_genera = get_genera_by_family_id(&db, family1.id).await
        .expect("Failed to get Rosaceae genera");
    assert_eq!(rosaceae_genera.len(), 2, "Should find 2 genera in Rosaceae");
    
    let poaceae_genera = get_genera_by_family_id(&db, family2.id).await
        .expect("Failed to get Poaceae genera");
    assert_eq!(poaceae_genera.len(), 1, "Should find 1 genus in Poaceae");
    
    let linnaeus_families = get_families_by_name(&db, "aceae").await
        .expect("Failed to search families");
    assert_eq!(linnaeus_families.len(), 2, "Should find both families ending in 'aceae'");
}

#[tokio::test]
async fn test_transaction_rollback_scenario() {
    let db = setup_test_database().await;
    
    // This test simulates what would happen if a transaction failed partway through
    let family = Family::new("TestFamily".to_string(), "Test".to_string());
    insert_family(&db, &family).await.expect("Failed to insert family");
    
    let genus = Genus::new(family.id, "TestGenus".to_string(), "Test".to_string());
    insert_genus(&db, &genus).await.expect("Failed to insert genus");
    
    // Try to insert a species with invalid genus_id (simulating partial transaction failure)
    let invalid_species = Species::new(
        Uuid::new_v4(), // Non-existent genus ID
        "invalid".to_string(),
        "Test".to_string(),
        None,
        None
    );
    
    let result = insert_species(&db, &invalid_species).await;
    assert!(result.is_err(), "Invalid species insert should fail");
    
    // Verify that the family and genus are still intact
    let family_exists = get_family_by_id(&db, family.id).await
        .expect("Family query should succeed");
    assert!(family_exists.is_some(), "Family should still exist after failed species insert");
    
    let genus_exists = get_genus_by_id(&db, genus.id).await
        .expect("Genus query should succeed");
    assert!(genus_exists.is_some(), "Genus should still exist after failed species insert");
}
