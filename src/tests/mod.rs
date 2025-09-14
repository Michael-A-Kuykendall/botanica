//! Test modules for BotanyDB
//!
//! Comprehensive test suite covering database operations, CRUD functionality,
//! and integration testing with in-memory SQLite databases.

use crate::{create_test_database, BotanicalDatabase};
use crate::types::{Family, Genus, Species};
use uuid::Uuid;

// Test modules
pub mod database_tests;
pub mod species_tests; 
pub mod genus_tests;
pub mod family_tests;
pub mod integration_tests;
pub mod comprehensive_tests;

/// Helper function to create a test database with sample data
pub async fn setup_test_database() -> BotanicalDatabase {
    let db = create_test_database().await.expect("Failed to create test database");
    db
}

/// Helper function to create a test family
pub fn create_test_family() -> Family {
    Family::new(
        "Rosaceae".to_string(),
        "Jussieu".to_string()
    )
}

/// Helper function to create a test genus with a specific family ID
pub fn create_test_genus(family_id: Uuid) -> Genus {
    Genus::new(
        family_id,
        "Rosa".to_string(),
        "Linnaeus".to_string()
    )
}

/// Helper function to create a test species with a specific genus ID
pub fn create_test_species(genus_id: Uuid) -> Species {
    Species::new(
        genus_id,
        "rubiginosa".to_string(),
        "Linnaeus".to_string(),
        Some(1753),
        Some("LC".to_string())
    )
}

/// Helper function to create sample taxonomic hierarchy
pub async fn setup_sample_taxonomy(db: &BotanicalDatabase) -> Result<(Family, Genus, Species), crate::DatabaseError> {
    use crate::queries::family::insert_family;
    use crate::queries::genus::insert_genus;
    use crate::queries::species::insert_species;

    let family = create_test_family();
    let genus = create_test_genus(family.id);
    let species = create_test_species(genus.id);

    insert_family(db.pool(), &family).await?;
    insert_genus(db.pool(), &genus).await?;
    insert_species(db.pool(), &species).await?;

    Ok((family, genus, species))
}

/// Helper function to assert species equality with better error messages
pub fn assert_species_eq(expected: &Species, actual: &Species) {
    assert_eq!(expected.id, actual.id, "Species ID mismatch");
    assert_eq!(expected.genus_id, actual.genus_id, "Genus ID mismatch");
    assert_eq!(expected.specific_epithet, actual.specific_epithet, "Specific epithet mismatch");
    assert_eq!(expected.authority, actual.authority, "Authority mismatch");
    assert_eq!(expected.publication_year, actual.publication_year, "Publication year mismatch");
    assert_eq!(expected.conservation_status, actual.conservation_status, "Conservation status mismatch");
}

/// Helper function to assert genus equality with better error messages
pub fn assert_genus_eq(expected: &Genus, actual: &Genus) {
    assert_eq!(expected.id, actual.id, "Genus ID mismatch");
    assert_eq!(expected.family_id, actual.family_id, "Family ID mismatch");
    assert_eq!(expected.name, actual.name, "Genus name mismatch");
    assert_eq!(expected.authority, actual.authority, "Authority mismatch");
}

/// Helper function to assert family equality with better error messages
pub fn assert_family_eq(expected: &Family, actual: &Family) {
    assert_eq!(expected.id, actual.id, "Family ID mismatch");
    assert_eq!(expected.name, actual.name, "Family name mismatch");
    assert_eq!(expected.authority, actual.authority, "Authority mismatch");
}