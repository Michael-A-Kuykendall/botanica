//! IUCN Red List conservation status integration
//! 
//! This module provides integration with the IUCN Red List API for automated
//! conservation status updates and endangered species tracking.

#[cfg(feature = "conservation")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "conservation")]
use std::fmt;

/// IUCN Red List conservation categories
#[cfg(feature = "conservation")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IUCNCategory {
    /// Not Evaluated - species has not been evaluated against the criteria
    NotEvaluated,
    /// Data Deficient - inadequate information to make assessment
    DataDeficient,
    /// Least Concern - species is widespread and abundant
    LeastConcern,
    /// Near Threatened - species is close to qualifying for threatened category
    NearThreatened,
    /// Vulnerable - species faces high risk of extinction in the wild
    Vulnerable,
    /// Endangered - species faces very high risk of extinction in the wild
    Endangered,
    /// Critically Endangered - species faces extremely high risk of extinction
    CriticallyEndangered,
    /// Extinct in the Wild - species survives only in cultivation/captivity
    ExtinctInWild,
    /// Extinct - no reasonable doubt that the last individual has died
    Extinct,
}

#[cfg(feature = "conservation")]
impl fmt::Display for IUCNCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            IUCNCategory::NotEvaluated => "NE",
            IUCNCategory::DataDeficient => "DD", 
            IUCNCategory::LeastConcern => "LC",
            IUCNCategory::NearThreatened => "NT",
            IUCNCategory::Vulnerable => "VU",
            IUCNCategory::Endangered => "EN",
            IUCNCategory::CriticallyEndangered => "CR",
            IUCNCategory::ExtinctInWild => "EW",
            IUCNCategory::Extinct => "EX",
        };
        write!(f, "{}", code)
    }
}

/// Population trend for conservation assessment
#[cfg(feature = "conservation")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PopulationTrend {
    Increasing,
    Stable,
    Decreasing,
    Unknown,
}

/// IUCN Red List assessment data
#[cfg(feature = "conservation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationAssessment {
    /// IUCN Red List category
    pub category: IUCNCategory,
    /// Assessment criteria (A, B, C, D, or E with subcriteria)
    pub criteria: Option<String>,
    /// Date of assessment
    pub assessment_date: chrono::NaiveDate,
    /// Current population trend
    pub population_trend: PopulationTrend,
    /// Major threats to the species
    pub threats: Vec<String>,
    /// Conservation actions in place
    pub conservation_actions: Vec<String>,
    /// Conservation actions needed
    pub actions_needed: Vec<String>,
    /// Assessor information
    pub assessor: Option<String>,
    /// Reviewer information  
    pub reviewer: Option<String>,
}

/// IUCN Red List API client for fetching conservation data
#[cfg(feature = "conservation")]
pub struct IUCNClient {
    api_token: Option<String>,
    client: reqwest::Client,
}

#[cfg(feature = "conservation")]
impl IUCNClient {
    /// Create new IUCN API client
    pub fn new(api_token: Option<String>) -> Self {
        Self {
            api_token,
            client: reqwest::Client::new(),
        }
    }
    
    /// Fetch conservation status for a species by scientific name
    pub async fn get_conservation_status(
        &self,
        scientific_name: &str
    ) -> Result<Option<ConservationAssessment>, Box<dyn std::error::Error + Send + Sync>> {
        // For demo purposes, return mock data
        // In production, this would call the real IUCN Red List API
        self.fetch_mock_conservation_data(scientific_name).await
    }
    
    /// Mock implementation for development/testing
    async fn fetch_mock_conservation_data(
        &self,
        scientific_name: &str
    ) -> Result<Option<ConservationAssessment>, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate some known conservation statuses
        let assessment = match scientific_name {
            "Cannabis sativa" => Some(ConservationAssessment {
                category: IUCNCategory::NotEvaluated,
                criteria: None,
                assessment_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                population_trend: PopulationTrend::Unknown,
                threats: vec!["Legal restrictions".to_string(), "Habitat loss".to_string()],
                conservation_actions: vec!["Cultivation programs".to_string()],
                actions_needed: vec!["Legal status review".to_string()],
                assessor: Some("Mock Assessment".to_string()),
                reviewer: None,
            }),
            "Welwitschia mirabilis" => Some(ConservationAssessment {
                category: IUCNCategory::NearThreatened,
                criteria: Some("A2acd".to_string()),
                assessment_date: chrono::NaiveDate::from_ymd_opt(2019, 7, 18).unwrap(),
                population_trend: PopulationTrend::Decreasing,
                threats: vec!["Climate change".to_string(), "Collection".to_string()],
                conservation_actions: vec!["Protected areas".to_string()],
                actions_needed: vec!["Population monitoring".to_string()],
                assessor: Some("IUCN Species Specialist Group".to_string()),
                reviewer: Some("IUCN Red List Unit".to_string()),
            }),
            _ => None,
        };
        
        // Simulate API delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(assessment)
    }
}

/// Integration functions for existing Botanica types
#[cfg(feature = "conservation")]
pub mod integration {
    use super::*;
    use crate::types::Species;
    use crate::database::BotanicalDatabase;
    use crate::error::DatabaseError;
    
    /// Update species conservation status from IUCN Red List
    pub async fn update_species_conservation_status(
        _db: &BotanicalDatabase,
        _species_id: uuid::Uuid,
        scientific_name: &str
    ) -> Result<Option<ConservationAssessment>, DatabaseError> {
        let client = IUCNClient::new(None);
        
        match client.get_conservation_status(scientific_name).await {
            Ok(Some(assessment)) => {
                // In a real implementation, we'd update the species record in the database
                // For now, just return the assessment
                Ok(Some(assessment))
            },
            Ok(None) => Ok(None),
            Err(e) => {
                log::warn!("Failed to fetch conservation status for {}: {}", scientific_name, e);
                Ok(None) // Graceful degradation if API fails
            },
        }
    }
    
    /// Check if a species is threatened (VU, EN, CR, EW, EX)
    pub fn is_species_threatened(assessment: &ConservationAssessment) -> bool {
        matches!(assessment.category, 
            IUCNCategory::Vulnerable | 
            IUCNCategory::Endangered | 
            IUCNCategory::CriticallyEndangered |
            IUCNCategory::ExtinctInWild |
            IUCNCategory::Extinct
        )
    }
    
    /// Get conservation priority score (0-10, higher = more urgent)
    pub fn get_conservation_priority(assessment: &ConservationAssessment) -> u8 {
        match assessment.category {
            IUCNCategory::Extinct => 10,
            IUCNCategory::ExtinctInWild => 9,
            IUCNCategory::CriticallyEndangered => 8,
            IUCNCategory::Endangered => 7,
            IUCNCategory::Vulnerable => 6,
            IUCNCategory::NearThreatened => 4,
            IUCNCategory::DataDeficient => 3,
            IUCNCategory::LeastConcern => 1,
            IUCNCategory::NotEvaluated => 0,
        }
    }
}

#[cfg(all(test, feature = "conservation"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_iucn_category_display() {
        assert_eq!(IUCNCategory::CriticallyEndangered.to_string(), "CR");
        assert_eq!(IUCNCategory::LeastConcern.to_string(), "LC");
        assert_eq!(IUCNCategory::NotEvaluated.to_string(), "NE");
    }
    
    #[test]
    fn test_conservation_priority() {
        let extinct_assessment = ConservationAssessment {
            category: IUCNCategory::Extinct,
            criteria: None,
            assessment_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            population_trend: PopulationTrend::Decreasing,
            threats: vec![],
            conservation_actions: vec![],
            actions_needed: vec![],
            assessor: None,
            reviewer: None,
        };
        
        assert_eq!(integration::get_conservation_priority(&extinct_assessment), 10);
        assert!(integration::is_species_threatened(&extinct_assessment));
    }
    
    #[test]
    fn test_threatened_species_detection() {
        let vulnerable_assessment = ConservationAssessment {
            category: IUCNCategory::Vulnerable,
            criteria: Some("A2acd".to_string()),
            assessment_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            population_trend: PopulationTrend::Decreasing,
            threats: vec!["Habitat loss".to_string()],
            conservation_actions: vec![],
            actions_needed: vec!["Habitat protection".to_string()],
            assessor: None,
            reviewer: None,
        };
        
        assert!(integration::is_species_threatened(&vulnerable_assessment));
        assert_eq!(integration::get_conservation_priority(&vulnerable_assessment), 6);
    }
    
    #[tokio::test]
    async fn test_iucn_client() {
        let client = IUCNClient::new(None);
        
        // Test known species
        let cannabis_status = client.get_conservation_status("Cannabis sativa").await.unwrap();
        assert!(cannabis_status.is_some());
        assert_eq!(cannabis_status.unwrap().category, IUCNCategory::NotEvaluated);
        
        // Test unknown species
        let unknown_status = client.get_conservation_status("Nonexistent species").await.unwrap();
        assert!(unknown_status.is_none());
    }
}