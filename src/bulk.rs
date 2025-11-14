// Bulk file operations
#![allow(dead_code)]

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUploadRequest {
    pub files: Vec<BulkFileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkFileItem {
    pub name: String,
    pub content: String, // Base64 encoded
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResponse {
    pub successful: Vec<String>,
    pub failed: Vec<BulkOperationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationError {
    pub name: String,
    pub error: String,
}

pub struct BulkOperations {
    storage: Arc<dyn Storage>,
}

impl BulkOperations {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Upload multiple files
    pub async fn bulk_upload(&self, request: BulkUploadRequest) -> Result<BulkOperationResponse> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for file in request.files {
            match self.upload_single_file(file.clone()).await {
                Ok(_) => successful.push(file.name),
                Err(e) => failed.push(BulkOperationError {
                    name: file.name,
                    error: e.to_string(),
                }),
            }
        }

        Ok(BulkOperationResponse { successful, failed })
    }

    async fn upload_single_file(&self, file: BulkFileItem) -> Result<()> {
        use base64::{engine::general_purpose, Engine as _};

        // Decode base64 content
        let data = general_purpose::STANDARD
            .decode(&file.content)
            .map_err(|e| ApiError::Storage(format!("Failed to decode base64: {}", e)))?;

        let mut metadata = FileMetadata::new(file.name.clone(), data.len() as u64);
        if let Some(custom) = file.metadata {
            metadata.custom = custom;
        }

        self.storage
            .put(&file.name, Bytes::from(data), metadata)
            .await
    }

    /// Delete multiple files
    pub async fn bulk_delete(&self, file_names: Vec<String>) -> Result<BulkOperationResponse> {
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for name in file_names {
            match self.storage.delete(&name).await {
                Ok(_) => successful.push(name),
                Err(e) => failed.push(BulkOperationError {
                    name: name.clone(),
                    error: e.to_string(),
                }),
            }
        }

        Ok(BulkOperationResponse { successful, failed })
    }

    /// Download multiple files
    pub async fn bulk_download(&self, file_names: Vec<String>) -> Result<BulkDownloadResponse> {
        use base64::{engine::general_purpose, Engine as _};

        let mut files = Vec::new();
        let mut failed = Vec::new();

        for name in file_names {
            match self.storage.get(&name).await {
                Ok((data, metadata)) => {
                    let file_name = metadata.file_name.clone();
                    files.push(BulkDownloadFile {
                        name: file_name,
                        content: general_purpose::STANDARD.encode(&data),
                        metadata: Some(metadata),
                    });
                }
                Err(e) => failed.push(BulkOperationError {
                    name: name.clone(),
                    error: e.to_string(),
                }),
            }
        }

        Ok(BulkDownloadResponse { files, failed })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDownloadResponse {
    pub files: Vec<BulkDownloadFile>,
    pub failed: Vec<BulkOperationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDownloadFile {
    pub name: String,
    pub content: String, // Base64 encoded
    pub metadata: Option<FileMetadata>,
}

// Note: base64 is not in dependencies yet, add to Cargo.toml:
// base64 = "0.22"
