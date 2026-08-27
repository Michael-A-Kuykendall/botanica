use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Health of a managed individual plant (L3 inventory)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Stressed,
    Declining,
    Dead,
    Dormant,
    Unknown,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Stressed => "stressed",
            HealthStatus::Declining => "declining",
            HealthStatus::Dead => "dead",
            HealthStatus::Dormant => "dormant",
            HealthStatus::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "healthy" => HealthStatus::Healthy,
            "stressed" => HealthStatus::Stressed,
            "declining" => HealthStatus::Declining,
            "dead" => HealthStatus::Dead,
            "dormant" => HealthStatus::Dormant,
            _ => HealthStatus::Unknown,
        }
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

/// One physical / managed plant instance (L3)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plant {
    pub id: Uuid,
    pub species_id: Option<Uuid>,
    pub cultivar_id: Option<Uuid>,
    pub user_given_name: String,
    pub health_status: HealthStatus,
    pub acquired_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    /// Sync hook — owner identity (optional until multi-device)
    pub user_id: Option<String>,
    /// Sync hook — device that last wrote
    pub device_id: Option<String>,
}

impl Plant {
    pub fn new(user_given_name: String, species_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            species_id,
            cultivar_id: None,
            user_given_name,
            health_status: HealthStatus::Unknown,
            acquired_date: None,
            location: None,
            notes: None,
            user_id: None,
            device_id: None,
        }
    }

    pub fn with_health(mut self, status: HealthStatus) -> Self {
        self.health_status = status;
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_cultivar(mut self, cultivar_id: Uuid) -> Self {
        self.cultivar_id = Some(cultivar_id);
        self
    }

    pub fn with_sync_ids(mut self, user_id: Option<String>, device_id: Option<String>) -> Self {
        self.user_id = user_id;
        self.device_id = device_id;
        self
    }
}
