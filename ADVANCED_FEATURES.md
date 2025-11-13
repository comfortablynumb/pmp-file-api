# Advanced Features Implementation Guide

This document describes the advanced features that have been implemented in the PMP File API.

## Implemented Features

### 1. File Versioning (`src/versioning.rs`)

**Status**: ✅ Complete - Module implemented, requires API integration

**Features**:
- Create new versions of files
- List all versions of a file
- Retrieve specific versions
- Restore previous versions
- Soft delete specific versions
- Automatic version numbering and UUID tracking

**Metadata Fields Added**:
- `version`: u32 - Version number (starts at 1)
- `version_id`: Option<Uuid> - Unique identifier for this version
- `parent_version_id`: Option<Uuid> - Link to previous version

**Usage**:
```rust
let versioning = VersioningService::new(storage);

// Create a new version
let new_metadata = versioning.create_version("file.txt", new_data, &base_metadata).await?;

// List all versions
let versions = versioning.list_versions("file.txt").await?;

// Restore a version
let restored = versioning.restore_version("file.txt", &version_id).await?;
```

### 2. Soft Delete / Trash (`src/metadata.rs`)

**Status**: ✅ Complete - Integrated into metadata

**Features**:
- Soft delete files without permanent removal
- Restore deleted files
- Automatic timestamp tracking
- Filter support to include/exclude deleted files

**Metadata Fields Added**:
- `is_deleted`: bool - Soft delete flag
- `deleted_at`: Option<DateTime<Utc>> - Deletion timestamp

**Usage**:
```rust
// Soft delete
metadata.soft_delete();

// Restore
metadata.restore();

// Filter excludes deleted by default
let filters = FilterParams {
    include_deleted: false, // Default
    ..Default::default()
};
```

### 3. File Deduplication (`src/deduplication.rs`)

**Status**: ✅ Complete - Module implemented, requires API integration

**Features**:
- SHA-256 hash-based deduplication
- Automatic duplicate detection
- Storage space savings tracking
- Find all files with same content

**Metadata Fields Added**:
- `content_hash`: Option<String> - SHA-256 hash of file content

**Usage**:
```rust
let dedup = DeduplicationManager::new(storage);

// Store with deduplication
let (is_duplicate, metadata) = dedup.put_deduplicated("file.txt", data, metadata).await?;

// Retrieve (automatically follows references)
let (data, metadata) = dedup.get_deduplicated("file.txt").await?;

// Find duplicates
let duplicates = dedup.find_duplicates(&hash).await?;

// Get stats
let stats = dedup.get_stats().await;
```

### 4. File Sharing with Expirable Links (`src/sharing.rs`)

**Status**: ✅ Complete - Module implemented, requires API integration

**Features**:
- Generate shareable links with expiration
- Password-protected links
- Download limit enforcement
- Separate upload and download links
- Link revocation
- Automatic cleanup of expired links

**ShareLink Structure**:
- `id`: Unique link identifier
- `expires_at`: Expiration timestamp
- `max_downloads`: Optional download limit
- `password`: Optional password protection
- `is_upload_link`: Upload vs download link

**Usage**:
```rust
let share_manager = ShareLinkManager::new();

// Create a download link
let link = ShareLink::new(
    "my-storage".to_string(),
    "file.txt".to_string(),
    3600, // expires in 1 hour
    Some(5), // max 5 downloads
    Some("secret".to_string()), // password protected
    false, // download link
);

let created_link = share_manager.create_link(link).await?;

// Access link
let link = share_manager.get_link(&link_id).await?;
if link.is_valid() && link.verify_password("secret") {
    // Allow download
    share_manager.increment_download(&link_id).await?;
}

// Revoke link
share_manager.revoke_link(&link_id).await?;

// Cleanup expired
let removed = share_manager.cleanup_expired().await?;
```

### 5. File Tags/Categories (`src/metadata.rs`)

**Status**: ✅ Complete - Integrated into metadata

**Features**:
- Multiple tags per file
- Tag-based filtering
- Tag search support

**Metadata Fields Added**:
- `tags`: Vec<String> - List of tags

**Usage**:
```rust
// Add tags during creation
let metadata = FileMetadata::new("file.txt".to_string(), size)
    .with_tags(vec!["important".to_string(), "project-x".to_string()]);

// Filter by tags
let filters = FilterParams {
    tags: Some(vec!["important".to_string()]),
    ..Default::default()
};
```

### 6. Webhooks (`src/webhooks.rs`)

**Status**: ✅ Complete - Module implemented, requires API integration

**Features**:
- Event-based notifications
- Multiple webhook endpoints
- Custom headers support
- Asynchronous delivery
- Event filtering
- Enable/disable webhooks

**Supported Events**:
- `FileUploaded`
- `FileDownloaded`
- `FileDeleted`
- `FileRestored`
- `FileVersionCreated`

**Usage**:
```rust
let webhook_manager = WebhookManager::new();

// Register webhook
let config = WebhookConfig {
    url: "https://example.com/webhook".to_string(),
    events: vec![WebhookEvent::FileUploaded, WebhookEvent::FileDeleted],
    headers: HashMap::from([("Authorization".to_string(), "Bearer token".to_string())]),
    enabled: true,
};

webhook_manager.register_webhook("my-webhook".to_string(), config).await?;

// Trigger event
let payload = WebhookPayload {
    event: WebhookEvent::FileUploaded,
    timestamp: Utc::now(),
    storage_name: "my-storage".to_string(),
    file_key: "file.txt".to_string(),
    metadata: Some(metadata),
    user_id: Some("user123".to_string()),
};

webhook_manager.trigger_event(payload).await?;
```

### 7. Prometheus Metrics (`src/metrics.rs`)

**Status**: ✅ Complete - Module implemented, requires integration

**Metrics Tracked**:
- HTTP requests (total, duration, by method/path/status)
- File operations (uploads, downloads, deletes, lists)
- File sizes (upload/download bytes)
- Storage operations and errors
- Versioning (versions created/restored)
- Deduplication (files deduplicated, bytes saved)
- Share links (created, accessed)
- Webhooks (sent, failed)
- Cache (hits, misses)
- System metrics (active connections, total files)

**Usage**:
```rust
let metrics = Arc::new(Metrics::new()?);

// Track upload
metrics.files_uploaded.with_label_values(&["my-storage"]).inc();
metrics.upload_bytes.with_label_values(&["my-storage"]).inc_by(file_size as f64);

// Track request
metrics.requests_total
    .with_label_values(&["GET", "/api/v1/file", "200"])
    .inc();

// Gather metrics for Prometheus
let metric_families = metrics.gather();
```

**Metrics Endpoint**: Requires adding `/metrics` endpoint to API

### 8. Caching Layer (`src/caching.rs`)

**Status**: ✅ Complete - Module implemented, requires integration

**Features**:
- In-memory caching with Moka
- Configurable capacity and TTL
- Size-based filtering (don't cache large files)
- Cache statistics
- Cache invalidation
- Enable/disable caching

**Configuration**:
```rust
let config = CacheConfig {
    max_capacity: 1000, // max entries
    ttl_seconds: 3600, // 1 hour TTL
    max_file_size: 10 * 1024 * 1024, // 10MB max
    enabled: true,
};

let cache = FileCache::new(config);
```

**Usage**:
```rust
// Try cache first
if let Some(cached) = cache.get("file.txt").await {
    return Ok((cached.data.clone(), cached.metadata.clone()));
}

// Cache miss - fetch from storage
let (data, metadata) = storage.get("file.txt").await?;

// Put in cache
cache.put("file.txt".to_string(), data.clone(), metadata.clone()).await?;

// Invalidate
cache.invalidate("file.txt").await?;

// Get stats
let stats = cache.stats().await;
```

### 9. Database Migrations

**Status**: ✅ Complete - SQL files created

**Location**: `migrations/`

**Files**:
- `20250113_add_advanced_features_postgres.sql`
- `20250113_add_advanced_features_mysql.sql`
- `20250113_add_advanced_features_sqlite.sql`

**To Apply**:
```bash
# PostgreSQL
psql -U fileapi -d filedb < migrations/20250113_add_advanced_features_postgres.sql

# MySQL
mysql -u fileapi -ppassword filedb < migrations/20250113_add_advanced_features_mysql.sql

# SQLite
sqlite3 /path/to/database.db < migrations/20250113_add_advanced_features_sqlite.sql
```

### 10. Docker Compose Development Environment

**Status**: ✅ Complete

**Location**: `docker-compose.yml`

**Services Included**:
- PostgreSQL (port 5432)
- MySQL (port 3306)
- Redis (port 6379)
- MinIO (S3-compatible, ports 9000/9001)
- ClamAV (virus scanner, port 3310)
- Prometheus (metrics, port 9090)
- Grafana (visualization, port 3001)

**Usage**:
```bash
# Start all services
docker-compose up -d

# Start specific services
docker-compose up -d postgres redis minio

# Stop all services
docker-compose down

# View logs
docker-compose logs -f

# Remove volumes (clean slate)
docker-compose down -v
```

**Access**:
- PostgreSQL: `postgresql://fileapi:password@localhost:5432/filedb`
- MySQL: `mysql://fileapi:password@localhost:3306/filedb`
- Redis: `redis://localhost:6379`
- MinIO Console: http://localhost:9001 (minioadmin/minioadmin)
- MinIO API: http://localhost:9000
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3001 (admin/admin)
- ClamAV: `localhost:3310`

## Pending Implementation Tasks

### 1. Storage Backend Updates

All storage backends (MySQL, PostgreSQL, SQLite) need to be updated to handle new metadata fields:

**Required Changes**:
- Update SELECT queries to include new fields
- Update INSERT/UPDATE queries
- Handle NULL values for backward compatibility
- Use `FileMetadata::new()` and builder pattern

**Example Fix**:
```rust
// Instead of struct initialization:
let metadata = FileMetadata {
    file_name,
    content_type,
    size,
    created_at,
    updated_at,
    custom,
    // ... missing new fields
};

// Use new() and builder:
let mut metadata = FileMetadata::new(file_name, size);
metadata.content_type = content_type;
metadata.created_at = created_at;
metadata.updated_at = updated_at;
metadata.custom = custom;
// New fields have defaults from new()
```

### 2. API Endpoints

Add new endpoints to `src/api/handlers.rs` and `src/api/mod.rs`:

**Versioning Endpoints**:
- `POST /api/v1/file/:storage/:key/versions` - Create new version
- `GET /api/v1/file/:storage/:key/versions` - List versions
- `GET /api/v1/file/:storage/:key/versions/:version_id` - Get specific version
- `POST /api/v1/file/:storage/:key/versions/:version_id/restore` - Restore version
- `DELETE /api/v1/file/:storage/:key/versions/:version_id` - Delete version

**Share Links Endpoints**:
- `POST /api/v1/share/create` - Create share link
- `GET /api/v1/share/:link_id` - Get link info
- `GET /api/v1/share/:link_id/download` - Download via share link
- `DELETE /api/v1/share/:link_id` - Revoke link
- `GET /api/v1/share/storage/:storage` - List links for storage

**Webhook Endpoints**:
- `POST /api/v1/webhooks` - Register webhook
- `GET /api/v1/webhooks` - List webhooks
- `DELETE /api/v1/webhooks/:name` - Unregister webhook

**Tag Endpoints**:
- `PUT /api/v1/file/:storage/:key/tags` - Update tags
- `GET /api/v1/tags/:storage` - List all tags in use

**Trash Endpoints**:
- `GET /api/v1/trash/:storage` - List deleted files
- `POST /api/v1/file/:storage/:key/restore` - Restore deleted file
- `DELETE /api/v1/trash/:storage/empty` - Permanently delete all trash

**Metrics Endpoint**:
- `GET /metrics` - Prometheus metrics endpoint

**Health Check Endpoints**:
- `GET /health/:storage` - Per-storage health check
- `GET /health/all` - All storages health

**Bulk Operations**:
- `POST /api/v1/bulk/upload` - Bulk upload
- `POST /api/v1/bulk/download` - Bulk download
- `POST /api/v1/bulk/delete` - Bulk delete

**Search Endpoint**:
- `GET /api/v1/search/:storage?q=...&tags=...` - Full-text metadata search

### 3. Rate Limiting

Add rate limiting middleware using `tower_governor`:

```rust
use tower_governor::{GovernorLayer, GovernorConfigBuilder};

// In api/mod.rs
let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap(),
);

let app = Router::new()
    // ... routes ...
    .layer(GovernorLayer {
        config: Box::leak(governor_conf),
    });
```

### 4. OpenAPI/Swagger Documentation

Add OpenAPI documentation using `utoipa`:

```rust
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        upload_file,
        download_file,
        delete_file,
        // ... more handlers
    ),
    components(schemas(FileMetadata, ShareLink, WebhookConfig))
)]
struct ApiDoc;

// In router
let app = Router::new()
    // ... routes ...
    .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));
```

### 5. Configuration Updates

Add to `src/config.rs`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storages: HashMap<String, StorageConfig>,
    pub cache: Option<CacheConfig>,
    pub webhooks: Option<HashMap<String, WebhookConfig>>,
    pub rate_limit: Option<RateLimitConfig>,
}
```

## Testing Guide

### Unit Tests

Each module includes unit tests:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --package pmp-file-api --lib versioning::tests
cargo test --package pmp-file-api --lib sharing::tests
cargo test --package pmp-file-api --lib deduplication::tests
```

### Integration Tests

Create integration tests for:
- End-to-end file versioning workflow
- Share link lifecycle
- Webhook delivery
- Cache hit/miss scenarios
- Deduplication across multiple uploads

### Manual Testing

1. Start Docker Compose environment
2. Run migrations
3. Start API server
4. Test endpoints with curl or Postman
5. Monitor Prometheus metrics
6. Check Grafana dashboards

## Performance Considerations

1. **Versioning**: Each version stores complete file - consider cleanup policy
2. **Deduplication**: Hash computation adds CPU overhead - async recommended
3. **Caching**: Monitor memory usage - adjust max_file_size and capacity
4. **Webhooks**: Asynchronous delivery prevents blocking - consider retry logic
5. **Metrics**: Minimal overhead - safe for production

## Security Considerations

1. **Share Links**: Use cryptographically random UUIDs
2. **Passwords**: Hash passwords before storing (not currently implemented)
3. **Webhooks**: Validate webhook URLs, use HMAC signatures
4. **Rate Limiting**: Protect against abuse
5. **File Uploads**: Enforce size limits, scan for viruses

## Next Steps

1. Fix storage backend compilation errors
2. Implement all API endpoints
3. Add rate limiting middleware
4. Generate OpenAPI documentation
5. Write integration tests
6. Update README with all features
7. Create example configurations
8. Write deployment guide

## Migration Path for Existing Deployments

1. **Backup all data**
2. Run database migrations
3. Update application code
4. Deploy new version
5. Test all features
6. Monitor metrics
7. Set up Grafana dashboards

## Support

For questions or issues:
- Check logs in `tracing` output
- Review Prometheus metrics at `/metrics`
- Check Grafana dashboards
- Review module documentation in source files
