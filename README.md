# PMP File API

PMP File API: A flexible Rust API to upload, download, and manage files using various storage backends. Part of the Poor Man's Platform ecosystem.

## Features

- **Multiple Storage Backends**:
  - AWS S3 and S3-compatible services (MinIO, LocalStack)
  - Azure Blob Storage
  - Google Cloud Storage (GCS)
  - Local filesystem storage
  - PostgreSQL database storage
  - MySQL database storage
  - SQLite database storage (perfect for serverless/edge)
  - Redis storage with TTL support (ideal for caching)
- **Presigned URLs**: Generate time-limited upload/download URLs for S3-compatible storage
- **File Processing & Validation**:
  - File size limits (configurable per storage)
  - MIME type validation (whitelist/blacklist)
  - Image thumbnail generation
  - File compression (gzip, brotli)
  - Virus scanning with ClamAV integration (optional feature)
- **YAML Configuration**: Easy-to-configure storage backends via YAML
- **Custom Metadata**: Attach custom JSON metadata to files
- **File Filtering**: List and filter files by name, content type, and custom metadata
- **RESTful API**: Clean REST endpoints for all file operations
- **Async/Await**: Built with Tokio for high-performance async I/O
- **Type-Safe**: Leverages Rust's type system for safety and reliability

### Enterprise Features

- **File Versioning**: Track file history with parent-child relationships
- **Soft Delete/Trash**: Recoverable file deletion with restore capability
- **File Sharing**: Time-limited, password-protected share links with download limits
- **Deduplication**: SHA-256 hash-based storage optimization
- **Bulk Operations**: Upload, download, and delete multiple files efficiently
- **Full-Text Search**: Query files by metadata, tags, and content type
- **Tagging System**: Organize files with custom tags
- **Caching**: In-memory cache with Moka for high-performance reads
- **Webhooks**: Event-driven notifications for file operations
- **Metrics**: Prometheus integration for comprehensive monitoring
- **Health Checks**: Per-storage and system-wide health monitoring
- **Rate Limiting**: Built-in rate limiting (10 req/sec with burst capacity)

## Documentation

### 📚 Complete Documentation

For comprehensive documentation including API reference, examples, and use cases:
- **[Complete Documentation](DOCUMENTATION.md)** - Full API reference with 30+ endpoints
- **[Advanced Features Guide](ADVANCED_FEATURES.md)** - Detailed feature implementation guide
- **[Examples Directory](examples/)** - Practical code examples in Bash, Python, and TypeScript

### 🚀 Quick Examples

**Upload a file:**
```bash
curl -X PUT http://localhost:3000/api/v1/file/my-storage \
  -F "file=@document.pdf" \
  -F 'metadata={"project": "alpha", "status": "draft"}'
```

**Search files:**
```bash
curl -X POST http://localhost:3000/api/v1/search/my-storage \
  -H "Content-Type: application/json" \
  -d '{"query": "report", "tags": ["finance", "q4"]}'
```

**Create share link:**
```bash
curl -X POST http://localhost:3000/api/v1/share/my-storage \
  -H "Content-Type: application/json" \
  -d '{"file_name": "report.pdf", "expires_in_seconds": 86400, "password": "secret"}'
```

**Bulk operations:**
```bash
curl -X POST http://localhost:3000/api/v1/bulk/my-storage/upload \
  -H "Content-Type: application/json" \
  -d '{"files": [{"name": "file1.txt", "content": "BASE64_DATA"}]}'
```

### 🐍 Python Client

```python
from file_api_client import FileAPIClient

api = FileAPIClient('http://localhost:3000')

# Upload
api.upload('my-storage', 'report.pdf', metadata={'project': 'alpha'})

# Search
results = api.search('my-storage', query='report', tags=['finance'])

# Create share link
share = api.create_share_link('my-storage', 'report.pdf',
                              expires_in_seconds=86400, password='secret')
```

See [examples/python/](examples/python/) for complete examples.

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- For S3 storage: AWS credentials configured or S3-compatible service
- For Azure storage: Azure Storage account credentials
- For GCS storage: Google Cloud service account credentials
- For PostgreSQL storage: PostgreSQL 12+ database
- For MySQL storage: MySQL 8.0+ or MariaDB 10.5+ database
- For SQLite storage: No additional dependencies (embedded database)
- For Redis storage: Redis 6.0+ server
- For virus scanning: ClamAV server (optional, requires `virus-scan` feature)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/pmp-file-api.git
cd pmp-file-api
```

2. Copy the example configuration:
```bash
cp config.example.yaml config.yaml
```

3. Edit `config.yaml` to configure your storage backends

4. Build and run:
```bash
cargo build --release
cargo run --release
```

Or for development:
```bash
cargo run
```

To enable virus scanning with ClamAV:
```bash
cargo build --release --features virus-scan
cargo run --release --features virus-scan
```

The API will start on `http://0.0.0.0:3000` by default.

## Configuration

Create a `config.yaml` file to define your storage backends:

```yaml
server:
  host: "0.0.0.0"
  port: 3000

storages:
  # S3 Storage
  my-s3:
    type: s3
    bucket: my-bucket
    region: us-east-1
    prefix: files/
    # Optional: for S3-compatible services
    # endpoint: http://localhost:9000

  # Local Filesystem Storage
  local-storage:
    type: local
    path: /tmp/files

  # PostgreSQL Storage
  postgres-storage:
    type: postgres
    connection_string: postgresql://user:password@localhost:5432/filedb

  # MySQL Storage
  mysql-storage:
    type: mysql
    connection_string: mysql://user:password@localhost:3306/filedb

  # SQLite Storage (serverless/edge)
  sqlite-storage:
    type: sqlite
    database_url: sqlite:///tmp/files.db

  # Redis Storage (cache with TTL)
  redis-storage:
    type: redis
    connection_string: redis://localhost:6379
    ttl_seconds: 3600  # Optional: files expire after 1 hour
    key_prefix: files:  # Optional: prefix for all keys

  # Azure Blob Storage
  azure-storage:
    type: azure
    account: myaccount
    access_key: YOUR_ACCESS_KEY_HERE
    container: files
    prefix: uploads/  # Optional

  # Google Cloud Storage
  gcs-storage:
    type: gcs
    bucket: my-gcs-bucket
    prefix: files/  # Optional
    credentials_path: /path/to/service-account.json  # Optional: uses default credentials if not provided
```

### Storage Backend Details

#### S3 Storage
- Stores files in Amazon S3 or S3-compatible services (MinIO, LocalStack)
- Metadata is stored as a separate JSON file alongside each file
- Requires AWS credentials or equivalent for S3-compatible services

#### Local Filesystem Storage
- Stores files directly on the local filesystem
- Metadata is stored as JSON files with `.metadata.json` extension
- Best for development and single-server deployments

#### PostgreSQL Storage
- Stores files and metadata in a PostgreSQL database
- Files are stored as BYTEA (binary data)
- Metadata is stored as JSONB for efficient querying
- Automatically creates the required table on first connection
- Suitable for transactional workloads and complex queries

#### MySQL Storage
- Stores files and metadata in a MySQL/MariaDB database
- Files are stored as LONGBLOB (binary data)
- Metadata is stored as JSON
- Automatically creates the required table on first connection
- Good for traditional relational database deployments

#### SQLite Storage
- Stores files and metadata in an embedded SQLite database
- Files are stored as BLOB (binary data)
- Metadata is stored as TEXT (JSON)
- Perfect for serverless/edge deployments and single-user applications
- No external database server required
- Automatically creates the required table on first connection

#### Redis Storage
- Stores files and metadata in Redis with optional TTL (time-to-live)
- Files expire automatically after the configured TTL period
- Uses separate keys for data and metadata
- Ideal for temporary file storage and caching scenarios
- Supports key prefixing for namespace isolation
- Note: Redis is in-memory, so ensure adequate memory for your files

#### Azure Blob Storage
- Stores files in Azure Blob Storage
- Metadata is stored as separate JSON blobs
- Requires Azure Storage account credentials
- Supports container-level organization with optional prefix
- Good for Azure cloud deployments

#### Google Cloud Storage (GCS)
- Stores files in Google Cloud Storage buckets
- Metadata is stored as separate JSON objects
- Supports service account authentication or default credentials
- Bucket-level organization with optional prefix
- Ideal for Google Cloud Platform deployments

### Environment Variables

- `CONFIG_PATH`: Path to configuration file (default: `config.yaml`)
- `RUST_LOG`: Logging level (default: `pmp_file_api=debug,tower_http=debug`)

## API Endpoints

### Health Check

```
GET /health
```

### Upload File

```
PUT /api/v1/file/{storage-name}
Content-Type: multipart/form-data

Fields:
- file: The file to upload (required)
- metadata: Custom JSON metadata (optional)
```

Example with curl:
```bash
curl -X PUT http://localhost:3000/api/v1/file/my-s3 \
  -F "file=@document.pdf" \
  -F 'metadata={"author": "John Doe", "version": 1}'
```

### Get File

```
GET /api/v1/file/{storage-name}/{file-name}
```

Example:
```bash
curl http://localhost:3000/api/v1/file/my-s3/document.pdf -o downloaded.pdf
```

### List Files

```
GET /api/v1/file/{storage-name}?prefix=...&name_pattern=...&content_type=...
```

Query parameters:
- `prefix`: Filter by path prefix
- `name_pattern`: Filter by file name pattern
- `content_type`: Filter by content type

Example:
```bash
curl "http://localhost:3000/api/v1/file/my-s3?name_pattern=report&content_type=application/pdf"
```

### Delete File

```
DELETE /api/v1/file/{storage-name}/{file-name}
```

Example:
```bash
curl -X DELETE http://localhost:3000/api/v1/file/my-s3/document.pdf
```

### Get File Metadata

```
GET /api/v1/file/{storage-name}/{file-name}/metadata
```

Example:
```bash
curl http://localhost:3000/api/v1/file/my-s3/document.pdf/metadata
```

### Generate Presigned Download URL

```
GET /api/v1/file/{storage-name}/{file-name}/presigned-download?expires_in=3600
```

Query parameters:
- `expires_in`: Time in seconds until the URL expires (default: 3600 = 1 hour)

Example:
```bash
curl "http://localhost:3000/api/v1/file/my-s3/document.pdf/presigned-download?expires_in=7200"
```

Response:
```json
{
  "url": "https://my-bucket.s3.amazonaws.com/files/document.pdf?...",
  "expires_at": "2024-01-15T12:00:00Z"
}
```

**Note**: Presigned URLs are currently only supported for S3-compatible storage backends.

### Generate Presigned Upload URL

```
GET /api/v1/file/{storage-name}/{file-name}/presigned-upload?expires_in=3600
```

Query parameters:
- `expires_in`: Time in seconds until the URL expires (default: 3600 = 1 hour)

Example:
```bash
curl "http://localhost:3000/api/v1/file/my-s3/new-file.pdf/presigned-upload?expires_in=7200"
```

Response:
```json
{
  "url": "https://my-bucket.s3.amazonaws.com/files/new-file.pdf?...",
  "expires_at": "2024-01-15T12:00:00Z"
}
```

You can then use the presigned URL directly to upload files without going through the API:
```bash
curl -X PUT "PRESIGNED_URL" --data-binary @local-file.pdf
```

**Note**: Presigned URLs are currently only supported for S3-compatible storage backends.

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## File Processing Features

The API includes optional file processing capabilities that can be enabled through configuration:

### Size Validation
- Configure maximum file size limits per storage backend
- Prevents uploading files that exceed the limit

### MIME Type Validation
- Whitelist: Only allow specific MIME types (e.g., images, PDFs)
- Blocklist: Reject specific MIME types (e.g., executables)
- Automatic MIME type detection from file content and filename

### Image Thumbnails
- Automatically generate thumbnails for uploaded images
- Configurable thumbnail dimensions
- Supports JPEG, PNG, WebP formats
- Thumbnails stored separately with `.thumbnail` suffix

### File Compression
- Optional gzip or brotli compression
- Reduces storage space and bandwidth
- Transparent decompression on retrieval

### Virus Scanning
- Integration with ClamAV for virus scanning
- Requires `virus-scan` feature flag
- Configurable ClamAV server connection
- Rejects files containing detected malware

**Note**: File processing features are implemented in the `processing` module but require integration into the upload handlers to be active. See `src/processing.rs` for implementation details.

## Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # YAML configuration
├── error.rs         # Error types
├── metadata.rs      # File metadata handling
├── processing.rs    # File processing & validation (size, MIME, thumbnails, compression, virus scanning)
├── storage/         # Storage abstraction
│   ├── mod.rs       # Storage trait and presigned URL support
│   ├── s3.rs        # S3 implementation with presigned URLs
│   ├── local.rs     # Local filesystem implementation
│   ├── postgres.rs  # PostgreSQL implementation
│   ├── mysql.rs     # MySQL implementation
│   ├── sqlite.rs    # SQLite implementation
│   ├── redis.rs     # Redis implementation with TTL
│   ├── azure.rs     # Azure Blob Storage implementation
│   └── gcs.rs       # Google Cloud Storage implementation
└── api/             # API layer
    ├── mod.rs       # Router setup
    ├── handlers.rs  # Request handlers (includes presigned URL endpoints)
    └── models.rs    # API models
```

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
