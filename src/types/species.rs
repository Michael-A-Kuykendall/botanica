use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a species in the botanical taxonomy system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
    pub id: Uuid,
    pub genus_id: Uuid,
    pub specific_epithet: String,
    pub authority: String,
    pub publication_year: Option<i32>,
    pub conservation_status: Option<String>,
    /// Full binomial (or trinomial) when known, e.g. "Rosa rubiginosa"
    pub scientific_name: Option<String>,
    /// accepted | synonym | unresolved | provisional
    pub taxonomic_status: String,
    /// species | subspecies | variety | form | ...
    pub rank: String,
}

impl Species {
    pub fn new(
        genus_id: Uuid,
        specific_epithet: String,
        authority: String,
        publication_year: Option<i32>,
        conservation_status: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            genus_id,
            specific_epithet,
            authority,
            publication_year,
            conservation_status,
            scientific_name: None,
            taxonomic_status: "accepted".to_string(),
            rank: "species".to_string(),
        }
    }

    pub fn with_id(
        id: Uuid,
        genus_id: Uuid,
        specific_epithet: String,
        authority: String,
        publication_year: Option<i32>,
        conservation_status: Option<String>,
    ) -> Self {
        Self {
            id,
            genus_id,
            specific_epithet,
            authority,
            publication_year,
            conservation_status,
            scientific_name: None,
            taxonomic_status: "accepted".to_string(),
            rank: "species".to_string(),
        }
    }

    pub fn with_taxonomy(
        mut self,
        scientific_name: Option<String>,
        taxonomic_status: impl Into<String>,
        rank: impl Into<String>,
    ) -> Self {
        self.scientific_name = scientific_name;
        self.taxonomic_status = taxonomic_status.into();
        self.rank = rank.into();
        self
    }

    pub fn get_specific_epithet(&self) -> &str {
        &self.specific_epithet
    }

    pub fn get_authority(&self) -> &str {
        &self.authority
    }

    pub fn get_publication_year(&self) -> Option<i32> {
        self.publication_year
    }

    pub fn get_conservation_status(&self) -> Option<&str> {
        self.conservation_status.as_deref()
    }

    pub fn set_conservation_status(&mut self, status: Option<String>) {
        self.conservation_status = status;
    }

    pub fn has_conservation_status(&self) -> bool {
        self.conservation_status.is_some()
    }
}
