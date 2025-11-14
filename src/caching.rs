// Caching layer for frequently accessed files
#![allow(dead_code)]

use crate::error::Result;
use crate::metadata::FileMetadata;
use bytes::Bytes;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct CachedFile {
    pub data: Bytes,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "default_max_capacity")]
    pub max_capacity: u64,
    #[serde(default = "default_ttl_seconds")]
    pub ttl_seconds: u64,
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    #[serde(default)]
    pub enabled: bool,
}

fn default_max_capacity() -> u64 {
    1000
}

fn default_ttl_seconds() -> u64 {
    3600 // 1 hour
}

fn default_max_file_size() -> u64 {
    10 * 1024 * 1024 // 10 MB
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: default_max_capacity(),
            ttl_seconds: default_ttl_seconds(),
            max_file_size: default_max_file_size(),
            enabled: false,
        }
    }
}

pub struct FileCache {
    cache: Cache<String, Arc<CachedFile>>,
    config: CacheConfig,
}

impl FileCache {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .build();

        Self { cache, config }
    }

    /// Get file from cache
    pub async fn get(&self, key: &str) -> Option<Arc<CachedFile>> {
        if !self.config.enabled {
            return None;
        }

        self.cache.get(key).await
    }

    /// Put file in cache (only if size is within limit)
    pub async fn put(&self, key: String, data: Bytes, metadata: FileMetadata) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Only cache files within size limit
        if data.len() as u64 > self.config.max_file_size {
            return Ok(());
        }

        let cached_file = Arc::new(CachedFile { data, metadata });
        self.cache.insert(key, cached_file).await;

        Ok(())
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await;
        Ok(())
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
            hit_rate: 0.0, // Moka doesn't expose hit_count/miss_count in this version
        }
    }

    /// Run cache maintenance (evict expired entries)
    pub async fn run_pending_tasks(&self) {
        self.cache.run_pending_tasks().await;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            max_capacity: 100,
            ttl_seconds: 60,
            max_file_size: 1024,
            enabled: true,
        };

        let cache = FileCache::new(config);

        let data = Bytes::from("test data");
        let metadata = FileMetadata::new("test.txt".to_string(), data.len() as u64);

        // Put in cache
        cache
            .put("test.txt".to_string(), data.clone(), metadata.clone())
            .await
            .unwrap();

        // Get from cache
        let cached = cache.get("test.txt").await;
        assert!(cached.is_some());

        let cached_file = cached.unwrap();
        assert_eq!(cached_file.data, data);
        assert_eq!(cached_file.metadata.file_name, "test.txt");

        // Invalidate
        cache.invalidate("test.txt").await.unwrap();

        let cached_after_invalidate = cache.get("test.txt").await;
        assert!(cached_after_invalidate.is_none());
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let config = CacheConfig {
            max_capacity: 100,
            ttl_seconds: 60,
            max_file_size: 10, // Only 10 bytes max
            enabled: true,
        };

        let cache = FileCache::new(config);

        let small_data = Bytes::from("small");
        let large_data = Bytes::from("this is a large file");

        let metadata = FileMetadata::new("file.txt".to_string(), 0);

        // Small file should be cached
        cache
            .put("small.txt".to_string(), small_data, metadata.clone())
            .await
            .unwrap();

        assert!(cache.get("small.txt").await.is_some());

        // Large file should not be cached
        cache
            .put("large.txt".to_string(), large_data, metadata)
            .await
            .unwrap();

        assert!(cache.get("large.txt").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        let cache = FileCache::new(config);

        let data = Bytes::from("test");
        let metadata = FileMetadata::new("test.txt".to_string(), data.len() as u64);

        cache
            .put("test.txt".to_string(), data, metadata)
            .await
            .unwrap();

        // Should not be cached when disabled
        assert!(cache.get("test.txt").await.is_none());
    }
}
