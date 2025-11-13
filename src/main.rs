mod api;
mod bulk;
mod caching;
mod config;
mod deduplication;
mod error;
mod health;
mod metadata;
mod metrics;
mod processing;
mod search;
mod sharing;
mod storage;
mod versioning;
mod webhooks;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::caching::FileCache;
use crate::config::{Config, StorageConfig};
use crate::metrics::SharedMetrics;
use crate::sharing::ShareLinkManager;
use crate::storage::{
    azure::AzureStorage, gcs::GcsStorage, local::LocalStorage, mysql::MySqlStorage,
    postgres::PostgresStorage, redis::RedisStorage, s3::S3Storage, sqlite::SqliteStorage, Storage,
};
use crate::webhooks::WebhookManager;

pub struct AppState {
    pub storages: HashMap<String, Arc<dyn Storage>>,
    pub metrics: SharedMetrics,
    pub cache: Arc<FileCache>,
    pub share_links: Arc<ShareLinkManager>,
    pub webhooks: Arc<WebhookManager>,
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
            StorageConfig::Postgres { connection_string } => {
                let postgres_storage = PostgresStorage::new(connection_string).await?;
                Arc::new(postgres_storage)
            }
            StorageConfig::MySql { connection_string } => {
                let mysql_storage = MySqlStorage::new(connection_string).await?;
                Arc::new(mysql_storage)
            }
            StorageConfig::Sqlite { database_url } => {
                let sqlite_storage = SqliteStorage::new(database_url).await?;
                Arc::new(sqlite_storage)
            }
            StorageConfig::Redis {
                connection_string,
                ttl_seconds,
                key_prefix,
            } => {
                let redis_storage =
                    RedisStorage::new(connection_string, *ttl_seconds, key_prefix.clone()).await?;
                Arc::new(redis_storage)
            }
            StorageConfig::Azure {
                account,
                access_key,
                container,
                prefix,
            } => {
                let azure_storage = AzureStorage::new(
                    account.clone(),
                    access_key.clone(),
                    container.clone(),
                    prefix.clone(),
                )
                .await?;
                Arc::new(azure_storage)
            }
            StorageConfig::Gcs {
                bucket,
                prefix,
                credentials_path,
            } => {
                let gcs_storage =
                    GcsStorage::new(bucket.clone(), prefix.clone(), credentials_path.clone())
                        .await?;
                Arc::new(gcs_storage)
            }
        };

        storages.insert(name.clone(), storage);
    }

    tracing::info!("Initialized {} storage(s)", storages.len());

    // Initialize metrics
    let metrics = Arc::new(metrics::Metrics::new()?);
    tracing::info!("Initialized Prometheus metrics");

    // Initialize cache
    let cache_config = caching::CacheConfig::default();
    let cache = Arc::new(FileCache::new(cache_config));
    tracing::info!("Initialized file cache");

    // Initialize share link manager
    let share_links = Arc::new(ShareLinkManager::new());
    tracing::info!("Initialized share link manager");

    // Initialize webhook manager
    let webhooks = Arc::new(WebhookManager::new());
    tracing::info!("Initialized webhook manager");

    // Create application state
    let state = Arc::new(AppState {
        storages,
        metrics: metrics.clone(),
        cache,
        share_links,
        webhooks,
    });

    // Create router
    let app = api::create_router(state).layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
