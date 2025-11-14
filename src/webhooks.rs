// Webhook support for file events
#![allow(dead_code)]

use crate::error::Result;
use crate::metadata::FileMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    Uploaded,
    Downloaded,
    Deleted,
    Restored,
    VersionCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub events: Vec<WebhookEvent>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub event: WebhookEvent,
    pub timestamp: DateTime<Utc>,
    pub storage_name: String,
    pub file_key: String,
    pub metadata: Option<FileMetadata>,
    pub user_id: Option<String>,
}

pub struct WebhookManager {
    webhooks: Arc<RwLock<HashMap<String, WebhookConfig>>>,
    client: reqwest::Client,
}

impl WebhookManager {
    pub fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub async fn register_webhook(&self, name: String, config: WebhookConfig) -> Result<()> {
        let mut webhooks = self.webhooks.write().await;
        webhooks.insert(name, config);
        Ok(())
    }

    pub async fn unregister_webhook(&self, name: &str) -> Result<()> {
        let mut webhooks = self.webhooks.write().await;
        webhooks.remove(name);
        Ok(())
    }

    pub async fn trigger_event(&self, payload: WebhookPayload) -> Result<()> {
        let webhooks = self.webhooks.read().await;

        // Trigger all webhooks that are subscribed to this event
        for (name, config) in webhooks.iter() {
            if !config.enabled {
                continue;
            }

            if !config.events.contains(&payload.event) {
                continue;
            }

            // Spawn async task to send webhook (don't block on response)
            let client = self.client.clone();
            let config = config.clone();
            let payload = payload.clone();
            let webhook_name = name.clone();

            tokio::spawn(async move {
                if let Err(e) = send_webhook(&client, &config, &payload).await {
                    tracing::error!(
                        webhook = %webhook_name,
                        error = %e,
                        "Failed to send webhook"
                    );
                }
            });
        }

        Ok(())
    }

    pub async fn list_webhooks(&self) -> Result<Vec<(String, WebhookConfig)>> {
        let webhooks = self.webhooks.read().await;
        Ok(webhooks
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

async fn send_webhook(
    client: &reqwest::Client,
    config: &WebhookConfig,
    payload: &WebhookPayload,
) -> Result<()> {
    let mut request = client.post(&config.url).json(payload);

    // Add custom headers
    for (key, value) in &config.headers {
        request = request.header(key, value);
    }

    let response = request
        .send()
        .await
        .map_err(|e| crate::error::ApiError::Storage(format!("Failed to send webhook: {}", e)))?;

    if !response.status().is_success() {
        return Err(crate::error::ApiError::Storage(format!(
            "Webhook returned status: {}",
            response.status()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_registration() {
        let manager = WebhookManager::new();

        let config = WebhookConfig {
            url: "https://example.com/webhook".to_string(),
            events: vec![WebhookEvent::Uploaded],
            headers: HashMap::new(),
            enabled: true,
        };

        manager
            .register_webhook("test".to_string(), config.clone())
            .await
            .unwrap();

        let webhooks = manager.list_webhooks().await.unwrap();
        assert_eq!(webhooks.len(), 1);
        assert_eq!(webhooks[0].0, "test");
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event: WebhookEvent::Uploaded,
            timestamp: Utc::now(),
            storage_name: "my-storage".to_string(),
            file_key: "test.txt".to_string(),
            metadata: None,
            user_id: Some("user123".to_string()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("uploaded"));
    }
}
