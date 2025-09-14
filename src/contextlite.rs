//! ContextLite integration for plant knowledge retrieval
//!
//! Provides AI-powered context assembly for plant cultivation data,
//! enabling intelligent plant care recommendations and troubleshooting.

use crate::error::DatabaseError;
use crate::types::{Species, CultivationRecord};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "contextlite")]
use contextlite_client::ContextLiteClient;

/// ContextLite integration for botanical knowledge
#[derive(Debug, Clone)]
pub struct BotanicalContext {
    #[cfg(feature = "contextlite")]
    client: ContextLiteClient,
    workspace_id: String,
}

/// Plant context query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantContextQuery {
    pub plant_id: Uuid,
    pub query: String,
    pub include_cultivation_history: bool,
    pub include_species_data: bool,
    pub max_documents: usize,
    pub max_tokens: usize,
}

/// Plant context response with AI insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantContextResponse {
    pub plant_id: Uuid,
    pub query: String,
    pub context: String,
    pub recommendations: Vec<String>,
    pub relevant_documents: Vec<ContextDocument>,
    pub confidence_score: f32,
}

/// Context document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDocument {
    pub id: String,
    pub title: String,
    pub source: String,
    pub relevance_score: f32,
    pub content_snippet: String,
}

impl BotanicalContext {
    /// Create new botanical context client
    #[cfg(feature = "contextlite")]
    pub fn new(base_url: &str, _auth_token: &str, workspace_id: &str) -> Result<Self, DatabaseError> {
        let client = ContextLiteClient::new(base_url)
            .map_err(|e| DatabaseError::ContextLiteError(e.to_string()))?;
        
        Ok(Self {
            client,
            workspace_id: workspace_id.to_string(),
        })
    }

    /// Create new botanical context client (no-op without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub fn new(_base_url: &str, _auth_token: &str, workspace_id: &str) -> Result<Self, DatabaseError> {
        Ok(Self {
            workspace_id: workspace_id.to_string(),
        })
    }

    /// Get AI-powered plant care recommendations
    #[cfg(feature = "contextlite")]
    pub async fn get_plant_recommendations(
        &self,
        species: &Species,
        cultivation_records: &[CultivationRecord],
        query: &str,
    ) -> Result<PlantContextResponse, DatabaseError> {
        // Build context from plant data
        let mut context_parts = vec![
            format!("Species: {}", species.specific_epithet),
            format!("Authority: {}", species.authority),
            format!("Genus: {}", species.genus_id),
        ];

        if !cultivation_records.is_empty() {
            let latest_record = &cultivation_records[cultivation_records.len() - 1];
            context_parts.push(format!("Current stage: {:?}", latest_record.growth_stage));
            if let Some(notes) = &latest_record.notes {
                context_parts.push(format!("Notes: {}", notes));
            }
        }

        // TODO: Implement actual ContextLite API call once we discover correct method
        // For now, provide mock response
        Ok(PlantContextResponse {
            plant_id: species.id, // Using species ID as plant ID for now
            query: query.to_string(),
            context: format!("Context for {} ({})", species.specific_epithet, query),
            recommendations: vec!["Mock recommendation".to_string()],
            relevant_documents: vec![],
            confidence_score: 0.8,
        })
    }

    /// Get AI-powered plant care recommendations (mock without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn get_plant_recommendations(
        &self,
        species: &Species,
        _cultivation_records: &[CultivationRecord],
        query: &str,
    ) -> Result<PlantContextResponse, DatabaseError> {
        // Mock response when ContextLite is not available
        Ok(PlantContextResponse {
            plant_id: species.id, // Using species ID as plant ID for now
            query: query.to_string(),
            context: "ContextLite feature not enabled".to_string(),
            recommendations: vec!["Enable ContextLite feature for AI recommendations".to_string()],
            relevant_documents: vec![],
            confidence_score: 0.0,
        })
    }

    /// Query general botanical knowledge
    #[cfg(feature = "contextlite")]
    pub async fn query_botanical_knowledge(&self, query: &str) -> Result<String, DatabaseError> {
        // TODO: Implement actual ContextLite API call
        Ok(format!("Mock botanical knowledge for: {}", query))
    }

    /// Query general botanical knowledge (mock without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn query_botanical_knowledge(&self, query: &str) -> Result<String, DatabaseError> {
        Ok(format!("ContextLite feature not enabled for query: {}", query))
    }

    /// Add plant data to ContextLite knowledge base
    #[cfg(feature = "contextlite")]
    pub async fn index_plant_data(
        &self,
        species: &Species,
        records: &[CultivationRecord],
    ) -> Result<(), DatabaseError> {
        // Assemble plant data into ContextLite document
        let _plant_data = format!(
            "SPECIES: {} ({})\nRECORDS: {}",
            species.specific_epithet,
            species.authority,
            records.len()
        );

        // TODO: Implement actual ContextLite document indexing
        // For now, just log the data being indexed
        log::info!("Would index {} records for species {}", records.len(), species.specific_epithet);

        Ok(())
    }

    /// Add plant data to ContextLite knowledge base (no-op without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn index_plant_data(
        &self,
        _species: &Species,
        _records: &[CultivationRecord],
    ) -> Result<(), DatabaseError> {
        Ok(()) // No-op when ContextLite is not available
    }
}

/// Extract recommendations from context text
pub fn extract_recommendations(context: &str) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if context.is_empty() {
        return recommendations;
    }
    
    let context_lower = context.to_lowercase();
    
    // Look for recommendation patterns in text
    if context_lower.contains("nutrient") && context_lower.contains("deficiency") {
        recommendations.push("Consider adjusting nutrient levels".to_string());
    }
    
    if context_lower.contains("water") && (context_lower.contains("over") || context_lower.contains("under")) {
        recommendations.push("Review watering schedule".to_string());
    }
    
    if context_lower.contains("light") && context_lower.contains("stress") {
        recommendations.push("Adjust lighting conditions".to_string());
    }
    
    if context_lower.contains("ph") {
        recommendations.push("Check and adjust soil/water pH levels".to_string());
    }
    
    if context_lower.contains("harvest") && context_lower.contains("ready") {
        recommendations.push("Consider harvest timing evaluation".to_string());
    }
    
    if context_lower.contains("temperature") {
        recommendations.push("Monitor temperature conditions".to_string());
    }
    
    if context_lower.contains("pest") || context_lower.contains("insect") {
        recommendations.push("Check for pest management needs".to_string());
    }
    
    if context_lower.contains("disease") || context_lower.contains("fungus") {
        recommendations.push("Evaluate plant health and disease prevention".to_string());
    }
    
    if context_lower.contains("pruning") || context_lower.contains("trim") {
        recommendations.push("Consider pruning and plant training techniques".to_string());
    }
    
    // If no specific patterns found, provide general recommendation
    if recommendations.is_empty() {
        recommendations.push("Review cultivation data and environmental conditions".to_string());
    }
    
    recommendations
}

/// Build context query from plant data
pub fn build_plant_context_query(
    species: &Species, 
    records: &[CultivationRecord],
    user_query: &str
) -> PlantContextQuery {
    PlantContextQuery {
        plant_id: species.id,
        query: format!("Species: {} {} - {}", 
            species.genus_id, 
            species.specific_epithet, 
            user_query
        ),
        include_cultivation_history: !records.is_empty(),
        include_species_data: true,
        max_documents: 10,
        max_tokens: 4000,
    }
}

/// Validate context response quality
pub fn validate_context_response(response: &PlantContextResponse) -> Result<(), DatabaseError> {
    if response.query.is_empty() {
        return Err(DatabaseError::ValidationError("Query cannot be empty".to_string()));
    }
    
    if response.context.is_empty() {
        return Err(DatabaseError::ValidationError("Context cannot be empty".to_string()));
    }
    
    if response.confidence_score < 0.0 || response.confidence_score > 1.0 {
        return Err(DatabaseError::ValidationError("Confidence score must be between 0.0 and 1.0".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GrowthStage;

    #[tokio::test]
    async fn test_botanical_context_creation() {
        let context = BotanicalContext::new(
            "http://localhost:8090",
            "test-token",
            "budsy-cultivation"
        ).expect("Failed to create context");

        assert_eq!(context.workspace_id, "budsy-cultivation");
    }

    #[tokio::test]
    async fn test_mock_recommendations() {
        let context = BotanicalContext::new(
            "http://localhost:8090", 
            "test-token",
            "test-workspace"
        ).expect("Failed to create context");

        let species = Species::new(
            Uuid::new_v4(),
            "test_species".to_string(),
            "L.".to_string(),
            Some(1753),
            None
        );

        let records = vec![
            CultivationRecord::new(
                species.id,
                GrowthStage::Vegetative,
                "test_cultivator".to_string()
            )
        ];

        let response = context.get_plant_recommendations(
            &species,
            &records,
            "How is my plant doing?"
        ).await.expect("Failed to get recommendations");

        assert_eq!(response.plant_id, species.id);
        assert!(!response.recommendations.is_empty());
    }

    #[test]
    fn test_recommendation_extraction() {
        // Test nutrient deficiency detection
        let nutrient_context = "The plant shows signs of nutrient deficiency";
        let recommendations = extract_recommendations(nutrient_context);
        assert!(recommendations.contains(&"Consider adjusting nutrient levels".to_string()));
        
        // Test watering issues
        let water_context = "Plant appears to be overwatered";
        let recommendations = extract_recommendations(water_context);
        assert!(recommendations.contains(&"Review watering schedule".to_string()));
        
        // Test pH issues
        let ph_context = "Check the pH levels in soil";
        let recommendations = extract_recommendations(ph_context);
        assert!(recommendations.contains(&"Check and adjust soil/water pH levels".to_string()));
        
        // Test empty context
        let empty_recommendations = extract_recommendations("");
        assert!(empty_recommendations.is_empty());
        
        // Test general case
        let general_context = "General plant concern";
        let recommendations = extract_recommendations(general_context);
        assert!(!recommendations.is_empty());
        assert!(recommendations.contains(&"Review cultivation data and environmental conditions".to_string()));
    }
    
    #[test]
    fn test_build_plant_context_query() {
        let species = Species::new(
            Uuid::new_v4(),
            "testspecies".to_string(),
            "L.".to_string(),
            Some(1753),
            None
        );
        
        let records = vec![
            CultivationRecord::new(
                species.id,
                GrowthStage::Vegetative,
                "test_cultivator".to_string()
            )
        ];
        
        let query = build_plant_context_query(&species, &records, "How is my plant?");
        
        assert_eq!(query.plant_id, species.id);
        assert!(query.query.contains("testspecies"));
        assert!(query.query.contains("How is my plant?"));
        assert!(query.include_cultivation_history);
        assert!(query.include_species_data);
        assert_eq!(query.max_documents, 10);
        assert_eq!(query.max_tokens, 4000);
    }
    
    #[test]
    fn test_validate_context_response() {
        let valid_response = PlantContextResponse {
            plant_id: Uuid::new_v4(),
            query: "Test query".to_string(),
            context: "Test context".to_string(),
            recommendations: vec!["Test recommendation".to_string()],
            relevant_documents: vec![],
            confidence_score: 0.8,
        };
        
        assert!(validate_context_response(&valid_response).is_ok());
        
        // Test invalid confidence score
        let invalid_confidence = PlantContextResponse {
            plant_id: Uuid::new_v4(),
            query: "Test query".to_string(),
            context: "Test context".to_string(),
            recommendations: vec![],
            relevant_documents: vec![],
            confidence_score: 1.5, // Invalid score > 1.0
        };
        
        assert!(validate_context_response(&invalid_confidence).is_err());
        
        // Test empty query
        let empty_query = PlantContextResponse {
            plant_id: Uuid::new_v4(),
            query: "".to_string(),
            context: "Test context".to_string(),
            recommendations: vec![],
            relevant_documents: vec![],
            confidence_score: 0.5,
        };
        
        assert!(validate_context_response(&empty_query).is_err());
    }
}