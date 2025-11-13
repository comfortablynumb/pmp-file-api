// File deduplication using content hashing
#![allow(dead_code)]

use crate::error::Result;
use crate::metadata::FileMetadata;
use crate::storage::Storage;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Computes SHA-256 hash of file content
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// In-memory hash-to-key mapping (can be replaced with Redis for production)
pub struct DeduplicationManager {
    hash_map: Arc<RwLock<HashMap<String, String>>>,
    storage: Arc<dyn Storage>,
}

impl DeduplicationManager {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self {
            hash_map: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// Store file with deduplication
    pub async fn put_deduplicated(
        &self,
        key: &str,
        data: Bytes,
        mut metadata: FileMetadata,
    ) -> Result<(bool, FileMetadata)> {
        // Compute hash
        let hash = compute_hash(&data);
        metadata.content_hash = Some(hash.clone());

        // Check if file with same hash already exists
        let hash_map = self.hash_map.read().await;
        if let Some(existing_key) = hash_map.get(&hash) {
            // File already exists, create a reference
            tracing::info!(
                key = %key,
                existing_key = %existing_key,
                hash = %hash,
                "Deduplicating file - using existing content"
            );

            // Store only metadata, not the actual file content
            self.storage.put(key, Bytes::new(), metadata.clone()).await?;

            return Ok((true, metadata)); // true = deduplicated
        }
        drop(hash_map);

        // File is unique, store it normally
        self.storage.put(key, data, metadata.clone()).await?;

        // Update hash map
        let mut hash_map = self.hash_map.write().await;
        hash_map.insert(hash, key.to_string());

        Ok((false, metadata)) // false = not deduplicated
    }

    /// Get file, following deduplication references if needed
    pub async fn get_deduplicated(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let (data, metadata) = self.storage.get(key).await?;

        // If data is empty but metadata has a hash, follow the reference
        if data.is_empty() && metadata.content_hash.is_some() {
            let hash = metadata.content_hash.as_ref().unwrap();
            let hash_map = self.hash_map.read().await;

            if let Some(original_key) = hash_map.get(hash) {
                if original_key != key {
                    tracing::debug!(
                        key = %key,
                        original_key = %original_key,
                        "Following deduplication reference"
                    );

                    // Get the actual data from the original file
                    let (original_data, _) = self.storage.get(original_key).await?;
                    return Ok((original_data, metadata));
                }
            }
        }

        Ok((data, metadata))
    }

    /// Find all files with the same content hash
    pub async fn find_duplicates(&self, hash: &str) -> Result<Vec<String>> {
        let hash_map = self.hash_map.read().await;
        let mut duplicates = Vec::new();

        // In a real implementation, this would query all files in storage
        // For now, we just check if the hash exists
        if let Some(key) = hash_map.get(hash) {
            duplicates.push(key.clone());
        }

        Ok(duplicates)
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> DeduplicationStats {
        let hash_map = self.hash_map.read().await;
        DeduplicationStats {
            unique_files: hash_map.len(),
            total_hashes: hash_map.len(),
        }
    }

    /// Rebuild hash map from storage (useful after restart)
    pub async fn rebuild_index(&self) -> Result<usize> {
        let files = self.storage.list(None).await?;
        let mut hash_map = self.hash_map.write().await;
        hash_map.clear();

        let mut count = 0;
        for metadata in files {
            if let Some(hash) = &metadata.content_hash {
                // Only index files that actually contain data
                if metadata.size > 0 {
                    hash_map.insert(hash.clone(), metadata.file_name.clone());
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationStats {
    pub unique_files: usize,
    pub total_hashes: usize,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local::LocalStorage;
    use tempfile::tempdir;

    #[test]
    fn test_hash_computation() {
        let data = b"Hello, World!";
        let hash = compute_hash(data);

        // Verify it's a valid hex string of correct length (SHA-256 = 64 hex chars)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same data should produce same hash
        let hash2 = compute_hash(data);
        assert_eq!(hash, hash2);

        // Different data should produce different hash
        let hash3 = compute_hash(b"Different data");
        assert_ne!(hash, hash3);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(LocalStorage::new(dir.path().to_str().unwrap()).await.unwrap());
        let dedup = DeduplicationManager::new(storage.clone());

        let data = Bytes::from("duplicate content");
        let metadata1 = FileMetadata::new("file1.txt".to_string(), data.len() as u64);
        let metadata2 = FileMetadata::new("file2.txt".to_string(), data.len() as u64);

        // Store first file
        let (is_dup1, _) = dedup
            .put_deduplicated("file1.txt", data.clone(), metadata1)
            .await
            .unwrap();
        assert!(!is_dup1); // First file is not a duplicate

        // Store second file with same content
        let (is_dup2, _) = dedup
            .put_deduplicated("file2.txt", data.clone(), metadata2)
            .await
            .unwrap();
        assert!(is_dup2); // Second file is a duplicate

        // Retrieve both files - should get same content
        let (data1, _) = dedup.get_deduplicated("file1.txt").await.unwrap();
        let (data2, _) = dedup.get_deduplicated("file2.txt").await.unwrap();

        assert_eq!(data1, data);
        assert_eq!(data2, data);

        // Check stats
        let stats = dedup.get_stats().await;
        assert_eq!(stats.unique_files, 1);
    }
}
