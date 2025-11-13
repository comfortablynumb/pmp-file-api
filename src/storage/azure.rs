use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use bytes::Bytes;
use futures::StreamExt;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct AzureStorage {
    container_client: ContainerClient,
    prefix: String,
}

impl AzureStorage {
    pub async fn new(
        account: String,
        access_key: String,
        container: String,
        prefix: String,
    ) -> Result<Self> {
        let storage_credentials =
            StorageCredentials::access_key(account.clone(), access_key.clone());

        let blob_service_client = BlobServiceClient::new(account, storage_credentials);
        let container_client = blob_service_client.container_client(container);

        // Try to create container if it doesn't exist
        let _ = container_client.create().await;

        Ok(Self {
            container_client,
            prefix,
        })
    }

    fn get_blob_name(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }

    fn get_metadata_blob_name(&self, key: &str) -> String {
        format!("{}.metadata.json", self.get_blob_name(key))
    }
}

#[async_trait]
impl Storage for AzureStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let blob_name = self.get_blob_name(key);
        let metadata_blob_name = self.get_metadata_blob_name(key);

        let blob_client = self.container_client.blob_client(blob_name);
        let metadata_blob_client = self.container_client.blob_client(metadata_blob_name);

        // Upload file data
        blob_client
            .put_block_blob(data)
            .content_type(metadata.content_type.clone().unwrap_or_default())
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to upload to Azure Blob: {}", e)))?;

        // Upload metadata
        let metadata_json = serde_json::to_vec(&metadata)?;
        metadata_blob_client
            .put_block_blob(Bytes::from(metadata_json))
            .content_type("application/json")
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to upload metadata: {}", e)))?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let blob_name = self.get_blob_name(key);
        let metadata_blob_name = self.get_metadata_blob_name(key);

        let blob_client = self.container_client.blob_client(blob_name);
        let metadata_blob_client = self.container_client.blob_client(metadata_blob_name);

        // Download file data
        let mut stream = blob_client.get().await.map_err(|e| {
            if e.to_string().contains("404") || e.to_string().contains("BlobNotFound") {
                ApiError::FileNotFound(key.to_string())
            } else {
                ApiError::Storage(format!("Failed to download from Azure Blob: {}", e))
            }
        })?;

        let mut data = Vec::new();
        while let Some(chunk) = stream.data.next().await {
            let chunk = chunk
                .map_err(|e| ApiError::Storage(format!("Failed to read chunk: {}", e)))?;
            data.extend_from_slice(&chunk);
        }

        // Download metadata
        let mut metadata_stream = metadata_blob_client
            .get()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to download metadata: {}", e)))?;

        let mut metadata_data = Vec::new();
        while let Some(chunk) = metadata_stream.data.next().await {
            let chunk = chunk
                .map_err(|e| ApiError::Storage(format!("Failed to read metadata chunk: {}", e)))?;
            metadata_data.extend_from_slice(&chunk);
        }

        let metadata: FileMetadata = serde_json::from_slice(&metadata_data)?;

        Ok((Bytes::from(data), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let blob_name = self.get_blob_name(key);
        let metadata_blob_name = self.get_metadata_blob_name(key);

        let blob_client = self.container_client.blob_client(&blob_name);
        let metadata_blob_client = self.container_client.blob_client(&metadata_blob_name);

        // Check if blob exists
        let exists = blob_client.get_properties().await.is_ok();
        if !exists {
            return Err(ApiError::FileNotFound(key.to_string()));
        }

        // Delete file blob
        blob_client
            .delete()
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete blob: {}", e)))?;

        // Delete metadata blob
        let _ = metadata_blob_client.delete().await;

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>> {
        let search_prefix = if let Some(p) = prefix {
            if self.prefix.is_empty() {
                p.to_string()
            } else {
                format!("{}/{}", self.prefix.trim_end_matches('/'), p)
            }
        } else {
            self.prefix.clone()
        };

        let mut stream = self
            .container_client
            .list_blobs()
            .prefix(search_prefix)
            .into_stream();

        let mut metadata_list = Vec::new();

        while let Some(result) = stream.next().await {
            let response = result
                .map_err(|e| ApiError::Storage(format!("Failed to list blobs: {}", e)))?;

            for blob in response.blobs.blobs() {
                // Skip metadata blobs
                if blob.name.ends_with(".metadata.json") {
                    continue;
                }

                // Extract relative key
                let key = if !self.prefix.is_empty() {
                    blob.name
                        .strip_prefix(&format!("{}/", self.prefix.trim_end_matches('/')))
                        .unwrap_or(&blob.name)
                } else {
                    &blob.name
                };

                // Try to get metadata for this blob
                if let Ok(metadata) = self.get_metadata(key).await {
                    metadata_list.push(metadata);
                }
            }
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let blob_name = self.get_blob_name(key);
        let blob_client = self.container_client.blob_client(blob_name);

        Ok(blob_client.get_properties().await.is_ok())
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        let metadata_blob_name = self.get_metadata_blob_name(key);
        let metadata_blob_client = self.container_client.blob_client(metadata_blob_name);

        let mut stream = metadata_blob_client.get().await.map_err(|e| {
            if e.to_string().contains("404") || e.to_string().contains("BlobNotFound") {
                ApiError::FileNotFound(key.to_string())
            } else {
                ApiError::Storage(format!("Failed to download metadata: {}", e))
            }
        })?;

        let mut data = Vec::new();
        while let Some(chunk) = stream.data.next().await {
            let chunk = chunk
                .map_err(|e| ApiError::Storage(format!("Failed to read chunk: {}", e)))?;
            data.extend_from_slice(&chunk);
        }

        let metadata: FileMetadata = serde_json::from_slice(&data)?;

        Ok(metadata)
    }
}
