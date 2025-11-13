// Extended API handlers for advanced enterprise features
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bulk::{BulkDownloadResponse, BulkOperationResponse, BulkUploadRequest};
use crate::error::Result;
use crate::health::{HealthCheck, StorageHealth};
use crate::metadata::FileMetadata;
use crate::search::{SearchQuery, SearchResults};
use crate::sharing::ShareLink;
use crate::versioning::VersioningService;
use crate::webhooks::WebhookConfig;
use crate::AppState;

// ============================================================================
// Versioning Endpoints
// ============================================================================

pub async fn create_version(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
    body: Bytes,
) -> Result<Json<FileMetadata>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let versioning = VersioningService::new(storage.clone());
    let (_, base_metadata) = storage.get(&file_name).await?;

    let new_metadata = versioning.create_version(&file_name, body, &base_metadata).await?;

    // Track metrics
    state.metrics.versions_created.with_label_values(&[&storage_name]).inc();

    Ok(Json(new_metadata))
}

pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Json<Vec<FileMetadata>>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let versioning = VersioningService::new(storage.clone());
    let versions = versioning.list_versions(&file_name).await?;

    Ok(Json(versions))
}

pub async fn get_version(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name, version_id)): Path<(String, String, String)>,
) -> Result<axum::response::Response> {
    use axum::http::{header, HeaderMap};

    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let version_uuid = Uuid::parse_str(&version_id)
        .map_err(|_| crate::error::ApiError::Storage("Invalid version ID".to_string()))?;

    let versioning = VersioningService::new(storage.clone());
    let (data, metadata) = versioning.get_version(&file_name, &version_uuid).await?;

    let mut headers = HeaderMap::new();
    if let Some(ct) = metadata.content_type {
        headers.insert(header::CONTENT_TYPE, ct.parse().unwrap());
    }
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", metadata.file_name)
            .parse()
            .unwrap(),
    );

    Ok((headers, data).into_response())
}

pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name, version_id)): Path<(String, String, String)>,
) -> Result<Json<FileMetadata>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let version_uuid = Uuid::parse_str(&version_id)
        .map_err(|_| crate::error::ApiError::Storage("Invalid version ID".to_string()))?;

    let versioning = VersioningService::new(storage.clone());
    let metadata = versioning.restore_version(&file_name, &version_uuid).await?;

    // Track metrics
    state.metrics.versions_restored.with_label_values(&[&storage_name]).inc();

    Ok(Json(metadata))
}

// ============================================================================
// Share Link Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateShareLinkRequest {
    pub file_key: String,
    #[serde(default = "default_share_expires_in")]
    pub expires_in_seconds: i64,
    pub max_downloads: Option<u32>,
    pub password: Option<String>,
    #[serde(default)]
    pub is_upload_link: bool,
}

fn default_share_expires_in() -> i64 {
    86400 // 24 hours
}

pub async fn create_share_link(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Json(req): Json<CreateShareLinkRequest>,
) -> Result<Json<ShareLink>> {
    let link = ShareLink::new(
        storage_name.clone(),
        req.file_key,
        req.expires_in_seconds,
        req.max_downloads,
        req.password,
        req.is_upload_link,
    );

    let created_link = state.share_links.create_link(link).await?;

    // Track metrics
    let link_type = if req.is_upload_link { "upload" } else { "download" };
    state.metrics.share_links_created
        .with_label_values(&[&storage_name, link_type])
        .inc();

    Ok(Json(created_link))
}

pub async fn get_share_link(
    State(state): State<Arc<AppState>>,
    Path(link_id): Path<String>,
) -> Result<Json<ShareLink>> {
    let link = state.share_links.get_link(&link_id).await?;
    Ok(Json(link))
}

pub async fn access_share_link(
    State(state): State<Arc<AppState>>,
    Path(link_id): Path<String>,
    Query(password): Query<Option<String>>,
) -> Result<Bytes> {
    let link = state.share_links.get_link(&link_id).await?;

    if !link.is_valid() {
        return Err(crate::error::ApiError::Storage("Share link expired or invalid".to_string()));
    }

    if let Some(pwd) = password {
        if !link.verify_password(&pwd) {
            return Err(crate::error::ApiError::Storage("Invalid password".to_string()));
        }
    }

    let storage = state.storages.get(&link.storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(link.storage_name.clone()))?;

    let (data, _) = storage.get(&link.file_key).await?;

    state.share_links.increment_download(&link_id).await?;
    state.metrics.share_links_accessed
        .with_label_values(&[&link.storage_name])
        .inc();

    Ok(data)
}

pub async fn revoke_share_link(
    State(state): State<Arc<AppState>>,
    Path(link_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    state.share_links.revoke_link(&link_id).await?;
    Ok(Json(serde_json::json!({"message": "Link revoked"})))
}

// ============================================================================
// Bulk Operations Endpoints
// ============================================================================

pub async fn bulk_upload(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Json(request): Json<BulkUploadRequest>,
) -> Result<Json<BulkOperationResponse>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let bulk_ops = crate::bulk::BulkOperations::new(storage.clone());
    let response = bulk_ops.bulk_upload(request).await?;

    // Track metrics
    state.metrics.files_uploaded
        .with_label_values(&[&storage_name])
        .inc_by(response.successful.len() as u64);

    Ok(Json(response))
}

pub async fn bulk_download(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Json(file_names): Json<Vec<String>>,
) -> Result<Json<BulkDownloadResponse>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let bulk_ops = crate::bulk::BulkOperations::new(storage.clone());
    let response = bulk_ops.bulk_download(file_names).await?;

    // Track metrics
    state.metrics.files_downloaded
        .with_label_values(&[&storage_name])
        .inc_by(response.files.len() as u64);

    Ok(Json(response))
}

pub async fn bulk_delete(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Json(file_names): Json<Vec<String>>,
) -> Result<Json<BulkOperationResponse>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let bulk_ops = crate::bulk::BulkOperations::new(storage.clone());
    let response = bulk_ops.bulk_delete(file_names).await?;

    // Track metrics
    state.metrics.files_deleted
        .with_label_values(&[&storage_name])
        .inc_by(response.successful.len() as u64);

    Ok(Json(response))
}

// ============================================================================
// Search Endpoint
// ============================================================================

pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResults>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let search_engine = crate::search::SearchEngine::new(storage.clone());
    let results = search_engine.search(query).await?;

    Ok(Json(results))
}

// ============================================================================
// Health Check Endpoints
// ============================================================================

pub async fn health_check_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HealthCheck>> {
    let health_checker = crate::health::HealthChecker::new(state.storages.clone());
    let health = health_checker.check_all().await;

    Ok(Json(health))
}

pub async fn health_check_storage(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
) -> Result<Json<StorageHealth>> {
    let health_checker = crate::health::HealthChecker::new(state.storages.clone());
    let health = health_checker.check_storage_by_name(&storage_name).await?;

    Ok(Json(health))
}

// ============================================================================
// Webhook Endpoints
// ============================================================================

pub async fn register_webhook(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(config): Json<WebhookConfig>,
) -> Result<Json<serde_json::Value>> {
    state.webhooks.register_webhook(name.clone(), config).await?;
    Ok(Json(serde_json::json!({"message": format!("Webhook '{}' registered", name)})))
}

pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<(String, WebhookConfig)>>> {
    let webhooks = state.webhooks.list_webhooks().await?;
    Ok(Json(webhooks))
}

pub async fn unregister_webhook(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    state.webhooks.unregister_webhook(&name).await?;
    Ok(Json(serde_json::json!({"message": format!("Webhook '{}' unregistered", name)})))
}

// ============================================================================
// Tag Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateTagsRequest {
    pub tags: Vec<String>,
}

pub async fn update_tags(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
    Json(req): Json<UpdateTagsRequest>,
) -> Result<Json<FileMetadata>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let (data, mut metadata) = storage.get(&file_name).await?;
    metadata.tags = req.tags;

    storage.put(&file_name, data, metadata.clone()).await?;

    Ok(Json(metadata))
}

pub async fn list_all_tags(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
) -> Result<Json<Vec<String>>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let files = storage.list(None).await?;

    let mut all_tags = std::collections::HashSet::new();
    for file in files {
        for tag in file.tags {
            all_tags.insert(tag);
        }
    }

    let mut tags: Vec<String> = all_tags.into_iter().collect();
    tags.sort();

    Ok(Json(tags))
}

// ============================================================================
// Trash/Soft Delete Endpoints
// ============================================================================

pub async fn list_trash(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
) -> Result<Json<Vec<FileMetadata>>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let files = storage.list(None).await?;
    let deleted_files: Vec<FileMetadata> = files
        .into_iter()
        .filter(|f| f.is_deleted)
        .collect();

    Ok(Json(deleted_files))
}

pub async fn restore_file(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Json<FileMetadata>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let (data, mut metadata) = storage.get(&file_name).await?;
    metadata.restore();

    storage.put(&file_name, data, metadata.clone()).await?;

    Ok(Json(metadata))
}

pub async fn empty_trash(
    State(state): State<Arc<AppState>>,
    Path(storage_name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let storage = state.storages.get(&storage_name)
        .ok_or_else(|| crate::error::ApiError::StorageNotFound(storage_name.clone()))?;

    let files = storage.list(None).await?;
    let mut deleted_count = 0;

    for file in files {
        if file.is_deleted {
            storage.delete(&file.file_name).await?;
            deleted_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "message": format!("Permanently deleted {} files", deleted_count),
        "count": deleted_count
    })))
}

// ============================================================================
// Metrics Endpoint
// ============================================================================

pub async fn metrics(
    State(state): State<Arc<AppState>>,
) -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

// ============================================================================
// Cache Management Endpoints
// ============================================================================

pub async fn cache_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::caching::CacheStats>> {
    let stats = state.cache.stats().await;
    Ok(Json(stats))
}

pub async fn cache_invalidate(
    State(state): State<Arc<AppState>>,
    Path((storage_name, file_name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let key = format!("{}:{}", storage_name, file_name);
    state.cache.invalidate(&key).await?;
    Ok(Json(serde_json::json!({"message": "Cache invalidated"})))
}

pub async fn cache_clear(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    state.cache.clear().await?;
    Ok(Json(serde_json::json!({"message": "Cache cleared"})))
}
