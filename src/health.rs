// Health check system
#![allow(dead_code)]

use crate::error::Result;
use crate::storage::Storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub details: HealthDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetails {
    pub storages: HashMap<String, StorageHealth>,
    pub uptime_seconds: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_check: DateTime<Utc>,
}

pub struct HealthChecker {
    storages: HashMap<String, Arc<dyn Storage>>,
    start_time: DateTime<Utc>,
}

impl HealthChecker {
    pub fn new(storages: HashMap<String, Arc<dyn Storage>>) -> Self {
        Self {
            storages,
            start_time: Utc::now(),
        }
    }

    /// Perform health check on all storages
    pub async fn check_all(&self) -> HealthCheck {
        let mut storage_health = HashMap::new();

        for (name, storage) in &self.storages {
            storage_health.insert(name.clone(), self.check_storage(storage).await);
        }

        // Overall status is unhealthy if any storage is unhealthy
        let overall_status = if storage_health
            .values()
            .any(|h| h.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else if storage_health
            .values()
            .any(|h| h.status == HealthStatus::Degraded)
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let uptime = Utc::now().signed_duration_since(self.start_time);

        HealthCheck {
            status: overall_status,
            timestamp: Utc::now(),
            details: HealthDetails {
                storages: storage_health,
                uptime_seconds: uptime.num_seconds() as u64,
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    /// Check health of a specific storage
    pub async fn check_storage(&self, storage: &Arc<dyn Storage>) -> StorageHealth {
        let start = std::time::Instant::now();
        let last_check = Utc::now();

        // Try to list files with empty prefix (should be fast)
        match tokio::time::timeout(Duration::from_secs(5), storage.list(None)).await {
            Ok(Ok(_)) => StorageHealth {
                status: HealthStatus::Healthy,
                message: Some("Storage is operational".to_string()),
                response_time_ms: start.elapsed().as_millis() as u64,
                last_check,
            },
            Ok(Err(e)) => StorageHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Storage error: {}", e)),
                response_time_ms: start.elapsed().as_millis() as u64,
                last_check,
            },
            Err(_) => StorageHealth {
                status: HealthStatus::Unhealthy,
                message: Some("Storage timeout".to_string()),
                response_time_ms: 5000, // Timeout duration
                last_check,
            },
        }
    }

    /// Check health of a specific storage by name
    pub async fn check_storage_by_name(&self, name: &str) -> Result<StorageHealth> {
        let storage = self
            .storages
            .get(name)
            .ok_or_else(|| crate::error::ApiError::StorageNotFound(name.to_string()))?;

        Ok(self.check_storage(storage).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");
    }
}
