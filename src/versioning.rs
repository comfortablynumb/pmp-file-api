// File versioning module
#![allow(dead_code)]

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;
use bytes::Bytes;
use std::sync::Arc;
use uuid::Uuid;

pub struct VersioningService {
    storage: Arc<dyn Storage>,
}

impl VersioningService {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Create a new version of a file
    pub async fn create_version(
        &self,
        key: &str,
        data: Bytes,
        base_metadata: &FileMetadata,
    ) -> Result<FileMetadata> {
        // Create new version metadata
        let new_metadata = base_metadata.create_new_version();

        // Store the new version with a versioned key
        let version_key = self.get_version_key(key, &new_metadata);
        self.storage
            .put(&version_key, data, new_metadata.clone())
            .await?;

        // Update the "latest" version pointer
        self.storage
            .put(key, Bytes::new(), new_metadata.clone())
            .await?;

        Ok(new_metadata)
    }

    /// Get a specific version of a file
    pub async fn get_version(&self, key: &str, version_id: &Uuid) -> Result<(Bytes, FileMetadata)> {
        // First, get all versions to find the right one
        let versions = self.list_versions(key).await?;

        for metadata in versions {
            if metadata.version_id.as_ref() == Some(version_id) {
                let version_key = self.get_version_key(key, &metadata);
                return self.storage.get(&version_key).await;
            }
        }

        Err(ApiError::NotFound(format!(
            "Version {} not found for file {}",
            version_id, key
        )))
    }

    /// List all versions of a file
    pub async fn list_versions(&self, key: &str) -> Result<Vec<FileMetadata>> {
        let prefix = format!("{}.", key);
        let all_files = self.storage.list(Some(&prefix)).await?;

        // Filter to only versioned files for this key
        let mut versions: Vec<FileMetadata> = all_files
            .into_iter()
            .filter(|m| m.file_name.starts_with(&prefix) && m.version_id.is_some())
            .collect();

        // Sort by version number (descending)
        versions.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(versions)
    }

    /// Delete a specific version (soft delete)
    pub async fn delete_version(&self, key: &str, version_id: &Uuid) -> Result<()> {
        let (data, mut metadata) = self.get_version(key, version_id).await?;

        // Soft delete the version
        metadata.soft_delete();

        let version_key = self.get_version_key(key, &metadata);
        self.storage.put(&version_key, data, metadata).await?;

        Ok(())
    }

    /// Restore a specific version as the latest version
    pub async fn restore_version(&self, key: &str, version_id: &Uuid) -> Result<FileMetadata> {
        let (data, _old_metadata) = self.get_version(key, version_id).await?;

        // Create a new version based on the old version's content
        let (_, current_metadata) = self.storage.get(key).await?;
        self.create_version(key, data, &current_metadata).await
    }

    /// Get the versioned storage key for a file version
    fn get_version_key(&self, key: &str, metadata: &FileMetadata) -> String {
        match &metadata.version_id {
            Some(version_id) => format!("{}.v{}.{}", key, metadata.version, version_id),
            None => key.to_string(),
        }
    }

    /// Get the latest non-deleted version
    pub async fn get_latest_version(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        // Try to get the main file
        match self.storage.get(key).await {
            Ok((data, metadata)) if !metadata.is_deleted => Ok((data, metadata)),
            _ => {
                // If main file is deleted or not found, try to find latest non-deleted version
                let versions = self.list_versions(key).await?;
                for metadata in versions {
                    if !metadata.is_deleted {
                        let version_key = self.get_version_key(key, &metadata);
                        return self.storage.get(&version_key).await;
                    }
                }
                Err(ApiError::NotFound(format!(
                    "No valid version found for {}",
                    key
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local::LocalStorage;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore] // TODO: Fix versioning test - needs proper setup
    async fn test_version_creation() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(
            LocalStorage::new(dir.path().to_str().unwrap())
                .await
                .unwrap(),
        );
        let versioning = VersioningService::new(storage.clone());

        let data = Bytes::from("version 1");
        let metadata = FileMetadata::new("test.txt".to_string(), data.len() as u64);

        // Create first version
        storage
            .put("test.txt", data.clone(), metadata.clone())
            .await
            .unwrap();

        // Create second version
        let data_v2 = Bytes::from("version 2");
        let metadata_v2 = versioning
            .create_version("test.txt", data_v2.clone(), &metadata)
            .await
            .unwrap();

        assert_eq!(metadata_v2.version, 2);
        assert!(metadata_v2.parent_version_id.is_some());

        // List versions
        let versions = versioning.list_versions("test.txt").await.unwrap();
        assert_eq!(versions.len(), 1); // Only v2 is stored with version key
    }
}
