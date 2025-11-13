use axum::{
    body::Bytes,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use std::sync::Arc;

use crate::api::models::{
    FileDeleteResponse, FileListResponse, FileUploadResponse, HealthResponse, ListQueryParams,
};
use crate::error::{ApiError, Result};
use crate::metadata::{FileMetadata, FilterParams};
use crate::AppState;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let mut file_data: Option<Bytes> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut custom_metadata = serde_json::Value::Object(serde_json::Map::new());

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::Internal(format!("Failed to read file data: {}", e)))?,
            );
        } else if field_name == "metadata" {
            let metadata_bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::Internal(format!("Failed to read metadata: {}", e)))?;
            custom_metadata = serde_json::from_slice(&metadata_bytes)
                .map_err(|e| ApiError::InvalidMetadata(e.to_string()))?;
        }
    }

    let file_data = file_data.ok_or_else(|| ApiError::Internal("No file provided".to_string()))?;
    let file_name =
        file_name.ok_or_else(|| ApiError::Internal("No file name provided".to_string()))?;

    let size = file_data.len() as u64;
    let mut metadata = FileMetadata::new(file_name.clone(), size);

    if let Some(ct) = content_type {
        metadata = metadata.with_content_type(ct);
    }

    metadata = metadata.with_custom(custom_metadata);

    storage.put(&file_name, file_data, metadata).await?;

    Ok(Json(FileUploadResponse {
        message: "File uploaded successfully".to_string(),
        file_name,
        size,
    }))
}

pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Response> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let (data, metadata) = storage.get(&file_name).await?;

    let mut response = Response::builder().status(StatusCode::OK);

    if let Some(content_type) = metadata.content_type {
        response = response.header(header::CONTENT_TYPE, content_type);
    }

    response = response.header(header::CONTENT_LENGTH, data.len()).header(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", metadata.file_name),
    );

    Ok(response.body(axum::body::Body::from(data)).unwrap())
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<FileListResponse>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let files = storage.list(params.prefix.as_deref()).await?;

    // Apply filters
    let filter_params = FilterParams {
        name_pattern: params.name_pattern,
        content_type: params.content_type,
        custom: None,
        tags: None,
        include_deleted: false,
    };

    let filtered_files: Vec<FileMetadata> = files
        .into_iter()
        .filter(|f| f.matches_filter(&filter_params))
        .collect();

    let count = filtered_files.len();

    Ok(Json(FileListResponse {
        files: filtered_files,
        count,
    }))
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Json<FileDeleteResponse>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    storage.delete(&file_name).await?;

    Ok(Json(FileDeleteResponse {
        message: format!("File '{}' deleted successfully", file_name),
    }))
}

pub async fn get_file_metadata(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Json<FileMetadata>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let metadata = storage.get_metadata(&file_name).await?;

    Ok(Json(metadata))
}

#[derive(Debug, serde::Deserialize)]
pub struct PresignedUrlQuery {
    #[serde(default = "default_expires_in")]
    pub expires_in: u64,
}

fn default_expires_in() -> u64 {
    3600 // 1 hour default
}

pub async fn generate_download_url(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
    Query(params): Query<PresignedUrlQuery>,
) -> Result<Json<crate::storage::PresignedUrl>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let presigned_url = storage
        .generate_presigned_download_url(&file_name, params.expires_in)
        .await?;

    Ok(Json(presigned_url))
}

pub async fn generate_upload_url(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
    Query(params): Query<PresignedUrlQuery>,
) -> Result<Json<crate::storage::PresignedUrl>> {
    let storage = state
        .storages
        .get(&storage_name)
        .ok_or_else(|| ApiError::StorageNotFound(storage_name.clone()))?;

    let presigned_url = storage
        .generate_presigned_upload_url(&file_name, params.expires_in)
        .await?;

    Ok(Json(presigned_url))
}
