pub mod azure;
pub mod gcs;
pub mod local;
pub mod mysql;
pub mod postgres;
pub mod redis;
pub mod s3;
pub mod sqlite;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::metadata::FileMetadata;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()>;
    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn get_metadata(&self, key: &str) -> Result<FileMetadata>;

    // Optional: Generate presigned URL for download (default implementation returns error)
    async fn generate_presigned_download_url(
        &self,
        _key: &str,
        _expires_in_seconds: u64,
    ) -> Result<PresignedUrl> {
        Err(crate::error::ApiError::Storage(
            "Presigned URLs not supported for this storage backend".to_string(),
        ))
    }

    // Optional: Generate presigned URL for upload (default implementation returns error)
    async fn generate_presigned_upload_url(
        &self,
        _key: &str,
        _expires_in_seconds: u64,
    ) -> Result<PresignedUrl> {
        Err(crate::error::ApiError::Storage(
            "Presigned URLs not supported for this storage backend".to_string(),
        ))
    }
}
