use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Storage '{0}' not found in configuration")]
    StorageNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::FileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::StorageNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidMetadata(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Serialization(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
