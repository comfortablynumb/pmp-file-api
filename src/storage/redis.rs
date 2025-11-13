use async_trait::async_trait;
use bytes::Bytes;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct RedisStorage {
    client: ConnectionManager,
    ttl_seconds: Option<u64>,
    key_prefix: String,
}

impl RedisStorage {
    pub async fn new(
        connection_string: &str,
        ttl_seconds: Option<u64>,
        key_prefix: Option<String>,
    ) -> Result<Self> {
        let client = redis::Client::open(connection_string)
            .map_err(|e| ApiError::Storage(format!("Failed to create Redis client: {}", e)))?;

        let connection_manager = ConnectionManager::new(client)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client: connection_manager,
            ttl_seconds,
            key_prefix: key_prefix.unwrap_or_else(|| "file:".to_string()),
        })
    }

    fn get_data_key(&self, key: &str) -> String {
        format!("{}data:{}", self.key_prefix, key)
    }

    fn get_metadata_key(&self, key: &str) -> String {
        format!("{}meta:{}", self.key_prefix, key)
    }

    fn get_list_key(&self) -> String {
        format!("{}list", self.key_prefix)
    }

    async fn set_with_ttl<T: redis::ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: T,
    ) -> Result<()> {
        let mut conn = self.client.clone();

        if let Some(ttl) = self.ttl_seconds {
            conn.set_ex::<_, _, ()>(key, value, ttl)
                .await
                .map_err(|e| ApiError::Storage(format!("Failed to set value in Redis: {}", e)))?;
        } else {
            conn.set::<_, _, ()>(key, value)
                .await
                .map_err(|e| ApiError::Storage(format!("Failed to set value in Redis: {}", e)))?;
        }

        Ok(())
    }
}

#[async_trait]
impl Storage for RedisStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let data_key = self.get_data_key(key);
        let metadata_key = self.get_metadata_key(key);
        let list_key = self.get_list_key();

        let metadata_json = serde_json::to_string(&metadata)?;

        // Store file data
        self.set_with_ttl(&data_key, data.as_ref()).await?;

        // Store metadata
        self.set_with_ttl(&metadata_key, metadata_json).await?;

        // Add to list of files
        let mut conn = self.client.clone();
        conn.sadd::<_, _, ()>(&list_key, key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to add to file list: {}", e)))?;

        if let Some(ttl) = self.ttl_seconds {
            conn.expire::<_, ()>(&list_key, ttl as i64)
                .await
                .map_err(|e| ApiError::Storage(format!("Failed to set TTL: {}", e)))?;
        }

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let data_key = self.get_data_key(key);
        let metadata_key = self.get_metadata_key(key);

        let mut conn = self.client.clone();

        // Get file data
        let data: Vec<u8> = conn.get(&data_key).await.map_err(|e| {
            if e.to_string().contains("nil") {
                ApiError::FileNotFound(key.to_string())
            } else {
                ApiError::Storage(format!("Failed to get file data: {}", e))
            }
        })?;

        // Get metadata
        let metadata_str: String = conn
            .get(&metadata_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to get metadata: {}", e)))?;

        let metadata: FileMetadata = serde_json::from_str(&metadata_str)?;

        Ok((Bytes::from(data), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let data_key = self.get_data_key(key);
        let metadata_key = self.get_metadata_key(key);
        let list_key = self.get_list_key();

        let mut conn = self.client.clone();

        // Check if file exists
        let exists: bool = conn
            .exists(&data_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to check existence: {}", e)))?;

        if !exists {
            return Err(ApiError::FileNotFound(key.to_string()));
        }

        // Delete data and metadata
        conn.del::<_, ()>(&data_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete file data: {}", e)))?;

        conn.del::<_, ()>(&metadata_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete metadata: {}", e)))?;

        // Remove from list
        conn.srem::<_, _, ()>(&list_key, key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to remove from file list: {}", e)))?;

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>> {
        let list_key = self.get_list_key();
        let mut conn = self.client.clone();

        // Get all file keys
        let keys: Vec<String> = conn
            .smembers(&list_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to list files: {}", e)))?;

        let mut metadata_list = Vec::new();

        for key in keys {
            // Filter by prefix if provided
            if let Some(prefix) = prefix {
                if !key.starts_with(prefix) {
                    continue;
                }
            }

            // Get metadata for each file
            if let Ok(metadata) = self.get_metadata(&key).await {
                metadata_list.push(metadata);
            }
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let data_key = self.get_data_key(key);
        let mut conn = self.client.clone();

        let exists: bool = conn
            .exists(&data_key)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to check existence: {}", e)))?;

        Ok(exists)
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        let metadata_key = self.get_metadata_key(key);
        let mut conn = self.client.clone();

        let metadata_str: String = conn.get(&metadata_key).await.map_err(|e| {
            if e.to_string().contains("nil") {
                ApiError::FileNotFound(key.to_string())
            } else {
                ApiError::Storage(format!("Failed to get metadata: {}", e))
            }
        })?;

        let metadata: FileMetadata = serde_json::from_str(&metadata_str)?;

        Ok(metadata)
    }
}
