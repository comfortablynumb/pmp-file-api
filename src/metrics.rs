// Prometheus metrics
#![allow(dead_code)]

use prometheus::{
    Counter, CounterVec, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts,
    Registry,
};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,

    // Request counters
    pub requests_total: IntCounterVec,
    pub requests_duration: HistogramVec,

    // File operation counters
    pub files_uploaded: IntCounterVec,
    pub files_downloaded: IntCounterVec,
    pub files_deleted: IntCounterVec,
    pub files_listed: IntCounter,

    // File sizes
    pub upload_bytes: CounterVec,
    pub download_bytes: CounterVec,

    // Storage metrics
    pub storage_operations: IntCounterVec,
    pub storage_errors: IntCounterVec,

    // Versioning metrics
    pub versions_created: IntCounterVec,
    pub versions_restored: IntCounterVec,

    // Deduplication metrics
    pub deduplicated_files: IntCounterVec,
    pub deduplicated_bytes_saved: Counter,

    // Share link metrics
    pub share_links_created: IntCounterVec,
    pub share_links_accessed: IntCounterVec,

    // Webhook metrics
    pub webhooks_sent: IntCounterVec,
    pub webhooks_failed: IntCounterVec,

    // Cache metrics
    pub cache_hits: IntCounterVec,
    pub cache_misses: IntCounterVec,

    // System metrics
    pub active_connections: IntGauge,
    pub total_files: IntGaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // Request counters
        let requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "path", "status"],
        )?;
        registry.register(Box::new(requests_total.clone()))?;

        let requests_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            ),
            &["method", "path"],
        )?;
        registry.register(Box::new(requests_duration.clone()))?;

        // File operation counters
        let files_uploaded = IntCounterVec::new(
            Opts::new("files_uploaded_total", "Total number of files uploaded"),
            &["storage"],
        )?;
        registry.register(Box::new(files_uploaded.clone()))?;

        let files_downloaded = IntCounterVec::new(
            Opts::new("files_downloaded_total", "Total number of files downloaded"),
            &["storage"],
        )?;
        registry.register(Box::new(files_downloaded.clone()))?;

        let files_deleted = IntCounterVec::new(
            Opts::new("files_deleted_total", "Total number of files deleted"),
            &["storage"],
        )?;
        registry.register(Box::new(files_deleted.clone()))?;

        let files_listed =
            IntCounter::new("files_listed_total", "Total number of file list operations")?;
        registry.register(Box::new(files_listed.clone()))?;

        // File sizes
        let upload_bytes = CounterVec::new(
            Opts::new("upload_bytes_total", "Total bytes uploaded"),
            &["storage"],
        )?;
        registry.register(Box::new(upload_bytes.clone()))?;

        let download_bytes = CounterVec::new(
            Opts::new("download_bytes_total", "Total bytes downloaded"),
            &["storage"],
        )?;
        registry.register(Box::new(download_bytes.clone()))?;

        // Storage metrics
        let storage_operations = IntCounterVec::new(
            Opts::new("storage_operations_total", "Total storage operations"),
            &["storage", "operation"],
        )?;
        registry.register(Box::new(storage_operations.clone()))?;

        let storage_errors = IntCounterVec::new(
            Opts::new("storage_errors_total", "Total storage errors"),
            &["storage", "operation"],
        )?;
        registry.register(Box::new(storage_errors.clone()))?;

        // Versioning metrics
        let versions_created = IntCounterVec::new(
            Opts::new("file_versions_created_total", "Total file versions created"),
            &["storage"],
        )?;
        registry.register(Box::new(versions_created.clone()))?;

        let versions_restored = IntCounterVec::new(
            Opts::new(
                "file_versions_restored_total",
                "Total file versions restored",
            ),
            &["storage"],
        )?;
        registry.register(Box::new(versions_restored.clone()))?;

        // Deduplication metrics
        let deduplicated_files = IntCounterVec::new(
            Opts::new("deduplicated_files_total", "Total deduplicated files"),
            &["storage"],
        )?;
        registry.register(Box::new(deduplicated_files.clone()))?;

        let deduplicated_bytes_saved = Counter::new(
            "deduplicated_bytes_saved_total",
            "Total bytes saved through deduplication",
        )?;
        registry.register(Box::new(deduplicated_bytes_saved.clone()))?;

        // Share link metrics
        let share_links_created = IntCounterVec::new(
            Opts::new("share_links_created_total", "Total share links created"),
            &["storage", "type"],
        )?;
        registry.register(Box::new(share_links_created.clone()))?;

        let share_links_accessed = IntCounterVec::new(
            Opts::new("share_links_accessed_total", "Total share link accesses"),
            &["storage"],
        )?;
        registry.register(Box::new(share_links_accessed.clone()))?;

        // Webhook metrics
        let webhooks_sent = IntCounterVec::new(
            Opts::new("webhooks_sent_total", "Total webhooks sent"),
            &["event"],
        )?;
        registry.register(Box::new(webhooks_sent.clone()))?;

        let webhooks_failed = IntCounterVec::new(
            Opts::new("webhooks_failed_total", "Total webhook failures"),
            &["event"],
        )?;
        registry.register(Box::new(webhooks_failed.clone()))?;

        // Cache metrics
        let cache_hits = IntCounterVec::new(
            Opts::new("cache_hits_total", "Total cache hits"),
            &["storage"],
        )?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = IntCounterVec::new(
            Opts::new("cache_misses_total", "Total cache misses"),
            &["storage"],
        )?;
        registry.register(Box::new(cache_misses.clone()))?;

        // System metrics
        let active_connections =
            IntGauge::new("active_connections", "Number of active connections")?;
        registry.register(Box::new(active_connections.clone()))?;

        let total_files = IntGaugeVec::new(
            Opts::new("total_files", "Total number of files per storage"),
            &["storage"],
        )?;
        registry.register(Box::new(total_files.clone()))?;

        Ok(Self {
            registry,
            requests_total,
            requests_duration,
            files_uploaded,
            files_downloaded,
            files_deleted,
            files_listed,
            upload_bytes,
            download_bytes,
            storage_operations,
            storage_errors,
            versions_created,
            versions_restored,
            deduplicated_files,
            deduplicated_bytes_saved,
            share_links_created,
            share_links_accessed,
            webhooks_sent,
            webhooks_failed,
            cache_hits,
            cache_misses,
            active_connections,
            total_files,
        })
    }

    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

/// Shared metrics instance
pub type SharedMetrics = Arc<Metrics>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new().unwrap();

        // Increment some metrics
        metrics
            .files_uploaded
            .with_label_values(&["test-storage"])
            .inc();
        metrics
            .files_downloaded
            .with_label_values(&["test-storage"])
            .inc();

        // Gather metrics
        let families = metrics.gather();
        assert!(!families.is_empty());
    }

    #[test]
    fn test_metrics_gathering() {
        let metrics = Metrics::new().unwrap();

        metrics
            .requests_total
            .with_label_values(&["GET", "/api/v1/file", "200"])
            .inc();

        let families = metrics.gather();
        let requests = families
            .iter()
            .find(|f| f.get_name() == "http_requests_total");

        assert!(requests.is_some());
    }
}
