use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// External ID linking a species to USDA / POWO / GBIF / WFO / etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesIdentifier {
    pub id: Uuid,
    pub species_id: Uuid,
    pub source: String,
    pub external_id: String,
    pub is_primary: bool,
}

impl SpeciesIdentifier {
    pub fn new(species_id: Uuid, source: impl Into<String>, external_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            species_id,
            source: source.into(),
            external_id: external_id.into(),
            is_primary: false,
        }
    }

    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
}
