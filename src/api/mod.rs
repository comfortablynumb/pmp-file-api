pub mod handlers;
pub mod models;

use axum::{routing::get, Router};
use std::sync::Arc;

use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
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
        .with_state(state)
}
