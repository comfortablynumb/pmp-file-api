# PMP File API

PMP File API: A flexible Rust API to upload, download, and manage files using various storage backends. Part of the Poor Man's Platform ecosystem.

## Features

- **Multiple Storage Backends**: Support for AWS S3, S3-compatible services (MinIO, LocalStack), and local filesystem
- **YAML Configuration**: Easy-to-configure storage backends via YAML
- **Custom Metadata**: Attach custom JSON metadata to files
- **File Filtering**: List and filter files by name, content type, and custom metadata
- **RESTful API**: Clean REST endpoints for all file operations
- **Async/Await**: Built with Tokio for high-performance async I/O
- **Type-Safe**: Leverages Rust's type system for safety and reliability

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- For S3 storage: AWS credentials configured or S3-compatible service

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

The API will start on `http://0.0.0.0:3000` by default.

## Configuration

Create a `config.yaml` file to define your storage backends:

```yaml
server:
  host: "0.0.0.0"
  port: 3000

storages:
  my-s3:
    type: s3
    bucket: my-bucket
    region: us-east-1
    prefix: files/
    # Optional: for S3-compatible services
    # endpoint: http://localhost:9000

  local-storage:
    type: local
    path: /tmp/files
```

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

## Project Structure

```
src/
├── main.rs          # Application entry point
├── config.rs        # YAML configuration
├── error.rs         # Error types
├── metadata.rs      # File metadata handling
├── storage/         # Storage abstraction
│   ├── mod.rs       # Storage trait
│   ├── s3.rs        # S3 implementation
│   └── local.rs     # Local filesystem implementation
└── api/             # API layer
    ├── mod.rs       # Router setup
    ├── handlers.rs  # Request handlers
    └── models.rs    # API models
```

## License

See [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
