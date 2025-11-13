mod api;
mod config;
mod error;
mod metadata;
mod storage;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, StorageConfig};
use crate::storage::{local::LocalStorage, s3::S3Storage, Storage};

pub struct AppState {
    pub storages: HashMap<String, Arc<dyn Storage>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pmp_file_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    tracing::info!("Loading configuration from: {}", config_path);

    let config = Config::from_file(&config_path)?;

    // Initialize storages
    let mut storages: HashMap<String, Arc<dyn Storage>> = HashMap::new();

    for (name, storage_config) in config.storages.iter() {
        tracing::info!("Initializing storage: {}", name);

        let storage: Arc<dyn Storage> = match storage_config {
            StorageConfig::S3 {
                bucket,
                region,
                prefix,
                endpoint,
            } => {
                let s3_storage = S3Storage::new(
                    bucket.clone(),
                    region.clone(),
                    prefix.clone(),
                    endpoint.clone(),
                )
                .await?;
                Arc::new(s3_storage)
            }
            StorageConfig::Local { path } => {
                let local_storage = LocalStorage::new(path).await?;
                Arc::new(local_storage)
            }
        };

        storages.insert(name.clone(), storage);
    }

    tracing::info!("Initialized {} storage(s)", storages.len());

    // Create application state
    let state = Arc::new(AppState { storages });

    // Create router
    let app = api::create_router(state).layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
