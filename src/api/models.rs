use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::metadata::FileMetadata;

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileMetadata>,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct FileUploadRequest {
    #[serde(default)]
    pub metadata: JsonValue,
}

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub message: String,
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct FileDeleteResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    pub prefix: Option<String>,
    pub name_pattern: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
