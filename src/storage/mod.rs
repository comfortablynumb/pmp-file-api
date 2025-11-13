pub mod local;
pub mod s3;

use async_trait::async_trait;
use bytes::Bytes;

use crate::error::Result;
use crate::metadata::FileMetadata;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()>;
    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn get_metadata(&self, key: &str) -> Result<FileMetadata>;
}
