use async_trait::async_trait;
use bytes::Bytes;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::list::ListObjectsRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct GcsStorage {
    client: Client,
    bucket: String,
    prefix: String,
}

impl GcsStorage {
    pub async fn new(
        bucket: String,
        prefix: String,
        credentials_path: Option<String>,
    ) -> Result<Self> {
        let config = if let Some(cred_path) = credentials_path {
            let cred = CredentialsFile::new_from_file(cred_path)
                .await
                .map_err(|e| {
                    ApiError::Storage(format!("Failed to load GCS credentials: {}", e))
                })?;
            ClientConfig::default()
                .with_credentials(cred)
                .await
                .map_err(|e| ApiError::Storage(format!("Failed to create GCS config: {}", e)))?
        } else {
            ClientConfig::default()
                .with_auth()
                .await
                .map_err(|e| ApiError::Storage(format!("Failed to create GCS config: {}", e)))?
        };

        let client = Client::new(config);

        Ok(Self {
            client,
            bucket,
            prefix,
        })
    }

    fn get_object_name(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), key)
        }
    }

    fn get_metadata_object_name(&self, key: &str) -> String {
        format!("{}.metadata.json", self.get_object_name(key))
    }
}

#[async_trait]
impl Storage for GcsStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let object_name = self.get_object_name(key);
        let metadata_object_name = self.get_metadata_object_name(key);

        // Upload file data
        let upload_type = UploadType::Simple(Media::new(object_name.clone()));
        let mut upload_request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        if let Some(content_type) = &metadata.content_type {
            upload_request.content_type = Some(content_type.clone());
        }

        self.client
            .upload_object(&upload_request, data.to_vec(), &upload_type)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to upload to GCS: {}", e)))?;

        // Upload metadata
        let metadata_json = serde_json::to_vec(&metadata)?;
        let metadata_upload_type = UploadType::Simple(Media::new(metadata_object_name.clone()));
        let metadata_upload_request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            content_type: Some("application/json".to_string()),
            ..Default::default()
        };

        self.client
            .upload_object(
                &metadata_upload_request,
                metadata_json,
                &metadata_upload_type,
            )
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to upload metadata to GCS: {}", e)))?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let object_name = self.get_object_name(key);
        let metadata_object_name = self.get_metadata_object_name(key);

        // Download file data
        let data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: object_name.clone(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("404") || e.to_string().contains("Not Found") {
                    ApiError::FileNotFound(key.to_string())
                } else {
                    ApiError::Storage(format!("Failed to download from GCS: {}", e))
                }
            })?;

        // Download metadata
        let metadata_data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: metadata_object_name,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to download metadata from GCS: {}", e)))?;

        let metadata: FileMetadata = serde_json::from_slice(&metadata_data)?;

        Ok((Bytes::from(data), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let object_name = self.get_object_name(key);
        let metadata_object_name = self.get_metadata_object_name(key);

        // Check if object exists
        let exists = self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name.clone(),
                ..Default::default()
            })
            .await
            .is_ok();

        if !exists {
            return Err(ApiError::FileNotFound(key.to_string()));
        }

        // Delete file object
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name,
                ..Default::default()
            })
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to delete from GCS: {}", e)))?;

        // Delete metadata object (ignore errors)
        let _ = self
            .client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: metadata_object_name,
                ..Default::default()
            })
            .await;

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

        let mut list_request = ListObjectsRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        if !search_prefix.is_empty() {
            list_request.prefix = Some(search_prefix.clone());
        }

        let response = self
            .client
            .list_objects(&list_request)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to list GCS objects: {}", e)))?;

        let mut metadata_list = Vec::new();

        if let Some(items) = response.items {
            for item in items {
                // Skip metadata objects
                if item.name.ends_with(".metadata.json") {
                    continue;
                }

                // Extract relative key
                let key = if !self.prefix.is_empty() {
                    item.name
                        .strip_prefix(&format!("{}/", self.prefix.trim_end_matches('/')))
                        .unwrap_or(&item.name)
                } else {
                    &item.name
                };

                // Try to get metadata for this object
                if let Ok(metadata) = self.get_metadata(key).await {
                    metadata_list.push(metadata);
                }
            }
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let object_name = self.get_object_name(key);

        let exists = self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name,
                ..Default::default()
            })
            .await
            .is_ok();

        Ok(exists)
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        let metadata_object_name = self.get_metadata_object_name(key);

        let data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: metadata_object_name,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("404") || e.to_string().contains("Not Found") {
                    ApiError::FileNotFound(key.to_string())
                } else {
                    ApiError::Storage(format!("Failed to download metadata from GCS: {}", e))
                }
            })?;

        let metadata: FileMetadata = serde_json::from_slice(&data)?;

        Ok(metadata)
    }
}
