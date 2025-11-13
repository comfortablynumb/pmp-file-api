use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct S3Storage {
    client: Client,
    bucket: String,
    prefix: String,
}

impl S3Storage {
    pub async fn new(
        bucket: String,
        region: String,
        prefix: String,
        endpoint: Option<String>,
    ) -> Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region));

        if let Some(endpoint_url) = endpoint {
            config_loader = config_loader.endpoint_url(endpoint_url);
        }

        let config = config_loader.load().await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket,
            prefix,
        })
    }

    fn get_full_key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }

    fn get_metadata_key(&self, key: &str) -> String {
        format!("{}.metadata.json", self.get_full_key(key))
    }

    async fn read_metadata(&self, key: &str) -> Result<FileMetadata> {
        let metadata_key = self.get_metadata_key(key);

        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to read metadata from S3: {}", e)))?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to read metadata body: {}", e)))?;

        let metadata: FileMetadata = serde_json::from_slice(&data.into_bytes())
            .map_err(|e| ApiError::InvalidMetadata(format!("Failed to parse metadata: {}", e)))?;

        Ok(metadata)
    }

    async fn write_metadata(&self, key: &str, metadata: &FileMetadata) -> Result<()> {
        let metadata_key = self.get_metadata_key(key);
        let content = serde_json::to_vec(metadata)?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .body(ByteStream::from(content))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to write metadata to S3: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl Storage for S3Storage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let full_key = self.get_full_key(key);

        let mut put_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(ByteStream::from(data));

        if let Some(content_type) = &metadata.content_type {
            put_request = put_request.content_type(content_type);
        }

        put_request
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to upload to S3: {}", e)))?;

        self.write_metadata(key, &metadata).await?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let full_key = self.get_full_key(key);

        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| {
                if e.to_string().contains("NoSuchKey") {
                    ApiError::FileNotFound(key.to_string())
                } else {
                    ApiError::Storage(format!("Failed to get from S3: {}", e))
                }
            })?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to read body: {}", e)))?;

        let metadata = self.read_metadata(key).await?;

        Ok((data.into_bytes(), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.get_full_key(key);
        let metadata_key = self.get_metadata_key(key);

        // Delete the file
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete from S3: {}", e)))?;

        // Delete the metadata
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&metadata_key)
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete metadata from S3: {}", e)))?;

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>> {
        let list_prefix = if let Some(p) = prefix {
            format!("{}/{}", self.get_full_key(""), p)
        } else {
            self.get_full_key("")
        };

        let result = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&list_prefix)
            .send()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to list S3 objects: {}", e)))?;

        let mut metadata_list = Vec::new();

        let contents = result.contents();
        for object in contents {
            if let Some(key) = object.key() {
                // Skip metadata files
                if key.ends_with(".metadata.json") {
                    continue;
                }

                // Remove the prefix to get the relative key
                let relative_key = if !self.prefix.is_empty() {
                    key.strip_prefix(&format!("{}/", self.prefix.trim_end_matches('/')))
                        .unwrap_or(key)
                } else {
                    key
                };

                if let Ok(metadata) = self.read_metadata(relative_key).await {
                    metadata_list.push(metadata);
                }
            }
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let full_key = self.get_full_key(key);

        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await;

        Ok(result.is_ok())
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        self.read_metadata(key).await
    }
}
