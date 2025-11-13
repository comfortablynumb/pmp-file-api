// File sharing with expirable links
#![allow(dead_code)]

use crate::error::{ApiError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareLink {
    pub id: String,
    pub storage_name: String,
    pub file_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_downloads: Option<u32>,
    pub download_count: u32,
    pub password: Option<String>,
    pub is_upload_link: bool,
}

impl ShareLink {
    pub fn new(
        storage_name: String,
        file_key: String,
        expires_in_seconds: i64,
        max_downloads: Option<u32>,
        password: Option<String>,
        is_upload_link: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            storage_name,
            file_key,
            created_at: now,
            expires_at: now + Duration::seconds(expires_in_seconds),
            max_downloads,
            download_count: 0,
            password,
            is_upload_link,
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = Utc::now();

        // Check expiration
        if now > self.expires_at {
            return false;
        }

        // Check download limit
        if let Some(max) = self.max_downloads {
            if self.download_count >= max {
                return false;
            }
        }

        true
    }

    pub fn increment_downloads(&mut self) {
        self.download_count += 1;
    }

    pub fn verify_password(&self, password: &str) -> bool {
        match &self.password {
            Some(stored_password) => stored_password == password,
            None => true, // No password required
        }
    }
}

/// In-memory share link manager (can be replaced with Redis for production)
pub struct ShareLinkManager {
    links: Arc<RwLock<HashMap<String, ShareLink>>>,
}

impl ShareLinkManager {
    pub fn new() -> Self {
        Self {
            links: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_link(&self, link: ShareLink) -> Result<ShareLink> {
        let mut links = self.links.write().await;
        let id = link.id.clone();
        links.insert(id, link.clone());
        Ok(link)
    }

    pub async fn get_link(&self, id: &str) -> Result<ShareLink> {
        let links = self.links.read().await;
        links
            .get(id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Share link {} not found", id)))
    }

    pub async fn increment_download(&self, id: &str) -> Result<()> {
        let mut links = self.links.write().await;
        if let Some(link) = links.get_mut(id) {
            link.increment_downloads();
            Ok(())
        } else {
            Err(ApiError::NotFound(format!("Share link {} not found", id)))
        }
    }

    pub async fn revoke_link(&self, id: &str) -> Result<()> {
        let mut links = self.links.write().await;
        links
            .remove(id)
            .ok_or_else(|| ApiError::NotFound(format!("Share link {} not found", id)))?;
        Ok(())
    }

    pub async fn list_links(&self, storage_name: &str) -> Result<Vec<ShareLink>> {
        let links = self.links.read().await;
        Ok(links
            .values()
            .filter(|link| link.storage_name == storage_name)
            .cloned()
            .collect())
    }

    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut links = self.links.write().await;
        let before_count = links.len();

        links.retain(|_, link| link.is_valid());

        Ok(before_count - links.len())
    }
}

impl Default for ShareLinkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_share_link_creation() {
        let manager = ShareLinkManager::new();

        let link = ShareLink::new(
            "my-storage".to_string(),
            "test.txt".to_string(),
            3600,
            Some(5),
            None,
            false,
        );

        let created = manager.create_link(link.clone()).await.unwrap();
        assert_eq!(created.storage_name, "my-storage");
        assert_eq!(created.max_downloads, Some(5));

        let retrieved = manager.get_link(&created.id).await.unwrap();
        assert_eq!(retrieved.id, created.id);
    }

    #[tokio::test]
    async fn test_share_link_expiration() {
        let link = ShareLink::new(
            "my-storage".to_string(),
            "test.txt".to_string(),
            -1, // Already expired
            None,
            None,
            false,
        );

        assert!(!link.is_valid());
    }

    #[tokio::test]
    async fn test_download_limit() {
        let mut link = ShareLink::new(
            "my-storage".to_string(),
            "test.txt".to_string(),
            3600,
            Some(2),
            None,
            false,
        );

        assert!(link.is_valid());

        link.increment_downloads();
        assert!(link.is_valid());

        link.increment_downloads();
        assert!(!link.is_valid()); // Exceeded limit
    }

    #[tokio::test]
    async fn test_password_protection() {
        let link = ShareLink::new(
            "my-storage".to_string(),
            "test.txt".to_string(),
            3600,
            None,
            Some("secret123".to_string()),
            false,
        );

        assert!(link.verify_password("secret123"));
        assert!(!link.verify_password("wrong"));
    }
}
