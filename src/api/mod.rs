pub mod handlers;
pub mod handlers_extended;
pub mod models;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    // Configure rate limiting (10 requests per second per IP)
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .finish()
            .unwrap(),
    );

    Router::new()
        // Basic health check
        .route("/health", get(handlers::health))
        // Advanced health checks
        .route("/health/all", get(handlers_extended::health_check_all))
        .route(
            "/health/:storage_name",
            get(handlers_extended::health_check_storage),
        )
        // Metrics endpoint (Prometheus)
        .route("/metrics", get(handlers_extended::metrics))
        // Basic file operations
        .route(
            "/api/v1/file/:storage_name",
            get(handlers::list_files).put(handlers::upload_file),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name",
            get(handlers::get_file).delete(handlers::delete_file),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name/metadata",
            get(handlers::get_file_metadata),
        )
        // Presigned URLs
        .route(
            "/api/v1/file/:storage_name/:file_name/presigned-download",
            get(handlers::generate_download_url),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name/presigned-upload",
            get(handlers::generate_upload_url),
        )
        // Versioning endpoints
        .route(
            "/api/v1/file/:storage_name/:file_name/versions",
            get(handlers_extended::list_versions).post(handlers_extended::create_version),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name/versions/:version_id",
            get(handlers_extended::get_version),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name/versions/:version_id/restore",
            post(handlers_extended::restore_version),
        )
        // Share links
        .route(
            "/api/v1/share/:storage_name",
            post(handlers_extended::create_share_link),
        )
        .route(
            "/api/v1/share/:link_id",
            get(handlers_extended::get_share_link).delete(handlers_extended::revoke_share_link),
        )
        .route(
            "/api/v1/share/:link_id/download",
            get(handlers_extended::access_share_link),
        )
        // Bulk operations
        .route(
            "/api/v1/bulk/:storage_name/upload",
            post(handlers_extended::bulk_upload),
        )
        .route(
            "/api/v1/bulk/:storage_name/download",
            post(handlers_extended::bulk_download),
        )
        .route(
            "/api/v1/bulk/:storage_name/delete",
            post(handlers_extended::bulk_delete),
        )
        // Search
        .route(
            "/api/v1/search/:storage_name",
            get(handlers_extended::search_files),
        )
        // Tags
        .route(
            "/api/v1/file/:storage_name/:file_name/tags",
            put(handlers_extended::update_tags),
        )
        .route(
            "/api/v1/tags/:storage_name",
            get(handlers_extended::list_all_tags),
        )
        // Trash/Soft Delete
        .route(
            "/api/v1/trash/:storage_name",
            get(handlers_extended::list_trash).delete(handlers_extended::empty_trash),
        )
        .route(
            "/api/v1/file/:storage_name/:file_name/restore",
            post(handlers_extended::restore_file),
        )
        // Webhooks
        .route("/api/v1/webhooks", get(handlers_extended::list_webhooks))
        .route(
            "/api/v1/webhooks/:name",
            post(handlers_extended::register_webhook).delete(handlers_extended::unregister_webhook),
        )
        // Cache management
        .route("/api/v1/cache/stats", get(handlers_extended::cache_stats))
        .route(
            "/api/v1/cache/:storage_name/:file_name",
            delete(handlers_extended::cache_invalidate),
        )
        .route("/api/v1/cache/clear", post(handlers_extended::cache_clear))
        // Apply rate limiting to all routes
        .layer(GovernorLayer {
            config: governor_conf,
        })
        .with_state(state)
}
