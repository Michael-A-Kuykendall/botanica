//! Darwin Core compliant data structures for professional botanical applications
//! 
//! This module implements the Darwin Core standard for biodiversity data exchange,
//! used by GBIF, iDigBio, and other major biodiversity informatics platforms.
//! 
//! These types are **additive only** - they don't replace the existing Botanica API,
//! but provide professional-grade botanical data management for advanced use cases.

#[cfg(feature = "darwin-core")]
use uuid::Uuid;
#[cfg(feature = "darwin-core")]
use chrono::{DateTime, Utc, NaiveDate};
#[cfg(feature = "darwin-core")]
use serde::{Deserialize, Serialize};

/// Darwin Core Occurrence record - core standard for biodiversity data
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarwinCoreOccurrence {
    // Core Darwin Core terms
    pub occurrence_id: Uuid,
    pub scientific_name: String,
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
    pub specific_epithet: Option<String>,
    
    // Event information
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<DateTime<Utc>>,
    pub recorded_by: Option<String>,
    
    // Location
    pub decimal_latitude: Option<f64>,
    pub decimal_longitude: Option<f64>,
    pub coordinate_uncertainty_in_meters: Option<f64>,
    pub country: Option<String>,
    pub state_province: Option<String>,
    pub locality: Option<String>,
    
    // Record metadata
    pub basis_of_record: BasisOfRecord,
    pub occurrence_status: OccurrenceStatus,
    pub catalog_number: Option<String>,
    pub collection_code: Option<String>,
    pub institution_code: Option<String>,
    
    // Additional fields
    pub individual_count: Option<i32>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub establishment_means: Option<EstablishmentMeans>,
    pub preparations: Option<String>,
}

/// Darwin Core Taxon record for nomenclatural data
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DarwinCoreTaxon {
    pub taxon_id: Uuid,
    pub scientific_name: String,
    pub scientific_name_authorship: Option<String>,
    pub taxonomic_status: TaxonomicStatus,
    pub taxon_rank: TaxonRank,
    pub nomenclatural_code: Option<NomenclaturalCode>,
    pub nomenclatural_status: Option<String>,
    pub parent_name_usage_id: Option<Uuid>,
    pub accepted_name_usage_id: Option<Uuid>,
    pub original_name_usage_id: Option<Uuid>,
}

/// Basis of Record - how the record was created
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasisOfRecord {
    PreservedSpecimen,
    FossilSpecimen,
    LivingSpecimen,
    HumanObservation,
    MachineObservation,
    MaterialSample,
    Event,
    Taxon,
    Occurrence,
}

/// Occurrence Status - whether the organism was present or absent
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OccurrenceStatus {
    Present,
    Absent,
}

/// How the organism came to be at the location
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EstablishmentMeans {
    Native,
    Introduced,
    Naturalised,
    Invasive,
    Managed,
    Cultivated,
}

/// Taxonomic Status following Darwin Core
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaxonomicStatus {
    Accepted,
    Synonym,
    DoubtfulSynonym,
    Misapplied,
    Homonym,
    ProvisionallyAccepted,
}

/// Taxonomic Rank following Darwin Core
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaxonRank {
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Genus,
    Species,
    Subspecies,
    Variety,
    Form,
    Cultivar,
}

/// Nomenclatural Code governing the scientific name
#[cfg(feature = "darwin-core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NomenclaturalCode {
    ICN,  // International Code of Nomenclature for algae, fungi, and plants
    ICZN, // International Code of Zoological Nomenclature
    ICNP, // International Code of Nomenclature of Prokaryotes
    ICVCN, // International Code of Virus Classification and Nomenclature
}

/// Compatibility functions to convert between existing Botanica types and Darwin Core
#[cfg(feature = "darwin-core")]
pub mod conversion {
    use super::*;
    use crate::types::Species;
    
    /// Convert existing Botanica Species to Darwin Core Taxon
    /// This maintains backward compatibility while adding professional features
    pub fn species_to_darwin_core_taxon(species: &Species, genus_name: &str) -> DarwinCoreTaxon {
        DarwinCoreTaxon {
            taxon_id: Uuid::new_v4(),
            scientific_name: format!("{} {}", 
                genus_name, 
                species.get_specific_epithet()
            ),
            scientific_name_authorship: Some(species.get_authority().to_string()),
            taxonomic_status: TaxonomicStatus::Accepted,
            taxon_rank: TaxonRank::Species,
            nomenclatural_code: Some(NomenclaturalCode::ICN),
            nomenclatural_status: None,
            parent_name_usage_id: None,
            accepted_name_usage_id: None,
            original_name_usage_id: None,
        }
    }
    
    /// Create Darwin Core Occurrence from existing Botanica data
    pub fn create_occurrence_record(
        species: &Species,
        genus_name: &str,
        family_name: &str,
        location: Option<(f64, f64)>,
        collection_info: Option<&str>
    ) -> DarwinCoreOccurrence {
        let (lat, lon) = location.unwrap_or((0.0, 0.0));
        
        DarwinCoreOccurrence {
            occurrence_id: Uuid::new_v4(),
            scientific_name: format!("{} {}", 
                genus_name, 
                species.get_specific_epithet()
            ),
            kingdom: Some("Plantae".to_string()),
            phylum: None,
            class: None,
            order: None,
            family: Some(family_name.to_string()),
            genus: Some(genus_name.to_string()),
            specific_epithet: Some(species.get_specific_epithet().to_string()),
            event_date: None,
            event_time: None,
            recorded_by: collection_info.map(|s| s.to_string()),
            decimal_latitude: if lat != 0.0 { Some(lat) } else { None },
            decimal_longitude: if lon != 0.0 { Some(lon) } else { None },
            coordinate_uncertainty_in_meters: None,
            country: None,
            state_province: None,
            locality: None,
            basis_of_record: BasisOfRecord::PreservedSpecimen,
            occurrence_status: OccurrenceStatus::Present,
            catalog_number: None,
            collection_code: None,
            institution_code: None,
            individual_count: Some(1),
            life_stage: None,
            reproductive_condition: None,
            establishment_means: None,
            preparations: None,
        }
    }
}

/// Professional Darwin Core functions - only available with feature flag
#[cfg(feature = "darwin-core")]
pub mod queries {
    use super::*;
    use crate::database::BotanicalDatabase;
    use crate::error::DatabaseError;
    
    /// Search occurrences by scientific name (professional feature)
    pub async fn search_darwin_core_occurrences(
        _db: &BotanicalDatabase,
        _scientific_name: &str
    ) -> Result<Vec<DarwinCoreOccurrence>, DatabaseError> {
        // This would integrate with existing database queries
        // For now, return empty vector as placeholder
        Ok(vec![])
    }
    
    /// Validate Darwin Core record completeness
    pub fn validate_darwin_core_record(record: &DarwinCoreOccurrence) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if record.decimal_latitude.is_none() || record.decimal_longitude.is_none() {
            warnings.push("Missing geographic coordinates".to_string());
        }
        
        if record.event_date.is_none() {
            warnings.push("Missing collection date".to_string());
        }
        
        if record.recorded_by.is_none() {
            warnings.push("Missing collector information".to_string());
        }
        
        warnings
    }
}

#[cfg(all(test, feature = "darwin-core"))]
mod tests {
    use super::*;
    use crate::types::Species;
    
    #[test]
    fn test_darwin_core_conversion() {
        let species = Species::new(
            Uuid::new_v4(), // genus_id 
            "rubiginosa".to_string(),
            "L.".to_string(), 
            None,
            None
        );
        
        let darwin_taxon = conversion::species_to_darwin_core_taxon(&species, "Rosa");
        assert_eq!(darwin_taxon.scientific_name, "Rosa rubiginosa");
        assert_eq!(darwin_taxon.scientific_name_authorship, Some("L.".to_string()));
        assert_eq!(darwin_taxon.taxon_rank, TaxonRank::Species);
    }
    
    #[test]
    fn test_occurrence_creation() {
        let species = Species::new(
            Uuid::new_v4(), // genus_id
            "rubiginosa".to_string(), 
            "L.".to_string(),
            None,
            None
        );
        
        let occurrence = conversion::create_occurrence_record(
            &species,
            "Rosa", // genus_name
            "Rosaceae", // family_name
            Some((40.7128, -74.0060)), // NYC coordinates
            Some("J. Smith")
        );
        
        assert_eq!(occurrence.scientific_name, "Rosa rubiginosa");
        assert_eq!(occurrence.decimal_latitude, Some(40.7128));
        assert_eq!(occurrence.recorded_by, Some("J. Smith".to_string()));
    }
    
    #[test]
    fn test_record_validation() {
        let mut occurrence = DarwinCoreOccurrence {
            occurrence_id: Uuid::new_v4(),
            scientific_name: "Rosa rubiginosa".to_string(),
            kingdom: Some("Plantae".to_string()),
            phylum: None,
            class: None,
            order: None,
            family: Some("Rosaceae".to_string()),
            genus: Some("Rosa".to_string()),
            specific_epithet: Some("rubiginosa".to_string()),
            event_date: None,
            event_time: None,
            recorded_by: None,
            decimal_latitude: None,
            decimal_longitude: None,
            coordinate_uncertainty_in_meters: None,
            country: None,
            state_province: None,
            locality: None,
            basis_of_record: BasisOfRecord::PreservedSpecimen,
            occurrence_status: OccurrenceStatus::Present,
            catalog_number: None,
            collection_code: None,
            institution_code: None,
            individual_count: Some(1),
            life_stage: None,
            reproductive_condition: None,
            establishment_means: None,
            preparations: None,
        };
        
        let warnings = queries::validate_darwin_core_record(&occurrence);
        assert_eq!(warnings.len(), 3); // Missing coords, date, collector
    }
}