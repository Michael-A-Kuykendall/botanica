use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cultivar or trade designation under a species (L1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cultivar {
    pub id: Uuid,
    pub species_id: Uuid,
    pub cultivar_name: String,
    pub trade_name: Option<String>,
    pub source: Option<String>,
}

impl Cultivar {
    pub fn new(species_id: Uuid, cultivar_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            species_id,
            cultivar_name,
            trade_name: None,
            source: None,
        }
    }

    pub fn with_trade_name(mut self, trade_name: impl Into<String>) -> Self {
        self.trade_name = Some(trade_name.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}
