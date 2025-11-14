# PMP File API - Comprehensive Documentation

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [API Reference](#api-reference)
6. [Feature Examples & Use Cases](#feature-examples--use-cases)
7. [Integration Examples](#integration-examples)
8. [Best Practices](#best-practices)
9. [Performance & Scalability](#performance--scalability)
10. [Troubleshooting](#troubleshooting)

---

## Overview

PMP File API is a production-ready, enterprise-grade file storage and management system built in Rust. It provides a unified API for managing files across multiple storage backends with advanced features like versioning, sharing, search, caching, and comprehensive monitoring.

### Key Highlights

- **Multi-Backend Support**: 8 storage backends (S3, Azure, GCS, Local, PostgreSQL, MySQL, SQLite, Redis)
- **Enterprise Features**: Versioning, soft delete/trash, file sharing, deduplication, bulk operations
- **Production Ready**: Rate limiting, health checks, Prometheus metrics, webhook notifications
- **High Performance**: Async I/O, in-memory caching, efficient storage abstraction
- **Type-Safe**: Built with Rust's type system for safety and reliability

---

## Features

### Core Features
- ✅ Multiple storage backends (8 supported)
- ✅ RESTful API with 30+ endpoints
- ✅ Custom metadata (JSON)
- ✅ File filtering and search
- ✅ Presigned URLs (S3-compatible)
- ✅ File processing (validation, thumbnails, compression)
- ✅ Virus scanning (ClamAV integration)

### Enterprise Features
- ✅ **File Versioning**: Track file history with parent-child relationships
- ✅ **Soft Delete/Trash**: Recoverable file deletion
- ✅ **File Sharing**: Time-limited, password-protected share links
- ✅ **Deduplication**: SHA-256 hash-based storage optimization
- ✅ **Bulk Operations**: Upload/download/delete multiple files
- ✅ **Full-Text Search**: Query files by metadata, tags, content type
- ✅ **Tagging System**: Organize files with custom tags
- ✅ **Caching**: In-memory cache with Moka for performance
- ✅ **Webhooks**: Event-driven notifications
- ✅ **Metrics**: Prometheus integration for monitoring
- ✅ **Health Checks**: Per-storage and system-wide health monitoring
- ✅ **Rate Limiting**: 10 req/sec with burst capacity

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/yourusername/pmp-file-api.git
cd pmp-file-api

# Copy configuration
cp config.example.yaml config.yaml

# Build and run
cargo build --release
cargo run --release
```

### Basic Usage

```bash
# Upload a file
curl -X PUT http://localhost:3000/api/v1/file/local-storage \
  -F "file=@document.pdf"

# Download a file
curl http://localhost:3000/api/v1/file/local-storage/document.pdf \
  -o downloaded.pdf

# List files
curl http://localhost:3000/api/v1/file/local-storage

# Delete a file
curl -X DELETE http://localhost:3000/api/v1/file/local-storage/document.pdf
```

---

## Configuration

### Complete Configuration Example

```yaml
server:
  host: "0.0.0.0"
  port: 3000

storages:
  # Production S3 Storage
  s3-prod:
    type: s3
    bucket: production-files
    region: us-east-1
    prefix: uploads/
    # For S3-compatible services (MinIO, LocalStack)
    # endpoint: http://localhost:9000

  # Local Development Storage
  local-dev:
    type: local
    path: /tmp/dev-files

  # PostgreSQL for Transactional Storage
  postgres-main:
    type: postgres
    connection_string: postgresql://user:pass@localhost:5432/filedb

  # MySQL for Traditional SQL Storage
  mysql-main:
    type: mysql
    connection_string: mysql://user:pass@localhost:3306/filedb

  # SQLite for Serverless/Edge
  sqlite-edge:
    type: sqlite
    database_url: sqlite:///var/lib/files.db

  # Redis for Cache/Temporary Storage
  redis-cache:
    type: redis
    connection_string: redis://localhost:6379
    ttl_seconds: 3600  # Files expire after 1 hour
    key_prefix: cache:

  # Azure Blob Storage
  azure-prod:
    type: azure
    account: myaccount
    access_key: YOUR_ACCESS_KEY
    container: files
    prefix: uploads/

  # Google Cloud Storage
  gcs-prod:
    type: gcs
    bucket: my-gcs-bucket
    prefix: files/
    credentials_path: /path/to/service-account.json
```

### Environment Variables

```bash
# Configuration file location
export CONFIG_PATH=/etc/pmp-file-api/config.yaml

# Logging level
export RUST_LOG=pmp_file_api=info,tower_http=info

# AWS Credentials (for S3)
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret

# Azure Credentials
export AZURE_STORAGE_ACCOUNT=myaccount
export AZURE_STORAGE_ACCESS_KEY=mykey

# Google Cloud Credentials
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

---

## API Reference

### Base URL
```
http://localhost:3000
```

### Rate Limiting
- **Rate**: 10 requests per second per IP
- **Burst**: 20 requests
- **Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

---

### 1. Health & Monitoring

#### Basic Health Check
```http
GET /health
```

**Response**:
```json
{
  "status": "healthy"
}
```

#### System-Wide Health Check
```http
GET /health/all
```

**Response**:
```json
{
  "status": "Healthy",
  "uptime_seconds": 3600,
  "storages": {
    "local-storage": {
      "status": "Healthy",
      "response_time_ms": 5,
      "message": "OK"
    },
    "s3-prod": {
      "status": "Healthy",
      "response_time_ms": 120,
      "message": "OK"
    }
  }
}
```

#### Per-Storage Health Check
```http
GET /health/:storage_name
```

**Example**:
```bash
curl http://localhost:3000/health/s3-prod
```

#### Prometheus Metrics
```http
GET /metrics
```

**Response**: Prometheus format metrics
```
# HELP file_upload_total Total number of file uploads
# TYPE file_upload_total counter
file_upload_total{storage="local-storage"} 42

# HELP file_size_bytes File size in bytes
# TYPE file_size_bytes histogram
file_size_bytes_bucket{storage="local-storage",le="1000"} 10
```

---

### 2. Basic File Operations

#### Upload File
```http
PUT /api/v1/file/:storage_name
Content-Type: multipart/form-data
```

**Parameters**:
- `file` (required): File to upload
- `metadata` (optional): JSON metadata

**Example**:
```bash
curl -X PUT http://localhost:3000/api/v1/file/local-storage \
  -F "file=@report.pdf" \
  -F 'metadata={"project": "Q4-2024", "department": "sales"}'
```

**Response**:
```json
{
  "file_name": "report.pdf",
  "content_type": "application/pdf",
  "size": 245678,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z",
  "version": 1,
  "version_id": "550e8400-e29b-41d4-a716-446655440000",
  "parent_version_id": null,
  "tags": [],
  "is_deleted": false,
  "deleted_at": null,
  "content_hash": "sha256:abc123...",
  "custom": {
    "project": "Q4-2024",
    "department": "sales"
  }
}
```

#### Download File
```http
GET /api/v1/file/:storage_name/:file_name
```

**Example**:
```bash
curl http://localhost:3000/api/v1/file/local-storage/report.pdf \
  -o downloaded-report.pdf
```

**Response**: File binary data

#### List Files
```http
GET /api/v1/file/:storage_name?prefix=&name_pattern=&content_type=&tags=
```

**Query Parameters**:
- `prefix`: Path prefix filter
- `name_pattern`: File name pattern
- `content_type`: MIME type filter
- `tags`: Comma-separated tag list

**Example**:
```bash
curl "http://localhost:3000/api/v1/file/local-storage?name_pattern=report&content_type=application/pdf"
```

**Response**:
```json
[
  {
    "file_name": "report.pdf",
    "content_type": "application/pdf",
    "size": 245678,
    ...
  }
]
```

#### Delete File
```http
DELETE /api/v1/file/:storage_name/:file_name
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/file/local-storage/report.pdf
```

**Response**:
```json
{
  "message": "File deleted successfully"
}
```

#### Get File Metadata
```http
GET /api/v1/file/:storage_name/:file_name/metadata
```

**Response**: File metadata object (same as upload response)

---

### 3. File Versioning

#### Create New Version
```http
POST /api/v1/file/:storage_name/:file_name/versions
Content-Type: multipart/form-data
```

**Parameters**:
- `file` (required): New version file data

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/file/local-storage/contract.pdf/versions \
  -F "file=@contract-v2.pdf"
```

**Response**:
```json
{
  "file_name": "contract.pdf",
  "version": 2,
  "version_id": "660e8400-e29b-41d4-a716-446655440001",
  "parent_version_id": "550e8400-e29b-41d4-a716-446655440000",
  ...
}
```

#### List All Versions
```http
GET /api/v1/file/:storage_name/:file_name/versions
```

**Example**:
```bash
curl http://localhost:3000/api/v1/file/local-storage/contract.pdf/versions
```

**Response**:
```json
[
  {
    "file_name": "contract.pdf",
    "version": 1,
    "version_id": "550e8400-e29b-41d4-a716-446655440000",
    "created_at": "2024-01-10T10:00:00Z",
    ...
  },
  {
    "file_name": "contract.pdf",
    "version": 2,
    "version_id": "660e8400-e29b-41d4-a716-446655440001",
    "parent_version_id": "550e8400-e29b-41d4-a716-446655440000",
    "created_at": "2024-01-15T10:00:00Z",
    ...
  }
]
```

#### Get Specific Version
```http
GET /api/v1/file/:storage_name/:file_name/versions/:version_id
```

**Example**:
```bash
curl http://localhost:3000/api/v1/file/local-storage/contract.pdf/versions/550e8400-e29b-41d4-a716-446655440000 \
  -o contract-v1.pdf
```

**Response**: Version file data with metadata headers

#### Restore Version
```http
POST /api/v1/file/:storage_name/:file_name/versions/:version_id/restore
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/file/local-storage/contract.pdf/versions/550e8400-e29b-41d4-a716-446655440000/restore
```

**Response**:
```json
{
  "file_name": "contract.pdf",
  "version": 3,
  "version_id": "770e8400-e29b-41d4-a716-446655440002",
  "parent_version_id": "660e8400-e29b-41d4-a716-446655440001",
  "message": "Restored from version 1",
  ...
}
```

---

### 4. File Sharing

#### Create Share Link
```http
POST /api/v1/share/:storage_name
Content-Type: application/json
```

**Request Body**:
```json
{
  "file_name": "report.pdf",
  "expires_in_seconds": 86400,
  "max_downloads": 10,
  "password": "secret123"
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/share/local-storage \
  -H "Content-Type: application/json" \
  -d '{
    "file_name": "report.pdf",
    "expires_in_seconds": 86400,
    "max_downloads": 10,
    "password": "secret123"
  }'
```

**Response**:
```json
{
  "link_id": "abc123def456",
  "storage_name": "local-storage",
  "file_name": "report.pdf",
  "expires_at": "2024-01-16T10:30:00Z",
  "max_downloads": 10,
  "download_count": 0,
  "password_protected": true,
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### Get Share Link Info
```http
GET /api/v1/share/:link_id
```

**Example**:
```bash
curl http://localhost:3000/api/v1/share/abc123def456
```

**Response**: Share link object

#### Download via Share Link
```http
GET /api/v1/share/:link_id/download?password=secret123
```

**Example**:
```bash
curl "http://localhost:3000/api/v1/share/abc123def456/download?password=secret123" \
  -o shared-file.pdf
```

**Response**: File binary data

#### Revoke Share Link
```http
DELETE /api/v1/share/:link_id
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/share/abc123def456
```

**Response**:
```json
{
  "message": "Share link revoked"
}
```

---

### 5. Bulk Operations

#### Bulk Upload
```http
POST /api/v1/bulk/:storage_name/upload
Content-Type: application/json
```

**Request Body**:
```json
{
  "files": [
    {
      "name": "file1.pdf",
      "content": "base64_encoded_content_here",
      "metadata": {"type": "document"}
    },
    {
      "name": "file2.jpg",
      "content": "base64_encoded_content_here",
      "metadata": {"type": "image"}
    }
  ]
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/bulk/local-storage/upload \
  -H "Content-Type: application/json" \
  -d '{
    "files": [
      {
        "name": "doc1.txt",
        "content": "SGVsbG8gV29ybGQh",
        "metadata": {}
      }
    ]
  }'
```

**Response**:
```json
{
  "total": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    {
      "file_name": "file1.pdf",
      "success": true,
      "error": null
    },
    {
      "file_name": "file2.jpg",
      "success": true,
      "error": null
    }
  ]
}
```

#### Bulk Download
```http
POST /api/v1/bulk/:storage_name/download
Content-Type: application/json
```

**Request Body**:
```json
{
  "file_names": ["file1.pdf", "file2.jpg"]
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/bulk/local-storage/download \
  -H "Content-Type: application/json" \
  -d '{"file_names": ["report.pdf", "image.jpg"]}'
```

**Response**:
```json
{
  "files": [
    {
      "name": "report.pdf",
      "content": "base64_encoded_content",
      "metadata": {...}
    },
    {
      "name": "image.jpg",
      "content": "base64_encoded_content",
      "metadata": {...}
    }
  ]
}
```

#### Bulk Delete
```http
POST /api/v1/bulk/:storage_name/delete
Content-Type: application/json
```

**Request Body**:
```json
{
  "file_names": ["file1.pdf", "file2.jpg"]
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/bulk/local-storage/delete \
  -H "Content-Type: application/json" \
  -d '{"file_names": ["old-file1.pdf", "old-file2.jpg"]}'
```

**Response**:
```json
{
  "total": 2,
  "successful": 2,
  "failed": 0,
  "results": [
    {
      "file_name": "old-file1.pdf",
      "success": true,
      "error": null
    },
    {
      "file_name": "old-file2.jpg",
      "success": true,
      "error": null
    }
  ]
}
```

---

### 6. Search

#### Search Files
```http
POST /api/v1/search/:storage_name
Content-Type: application/json
```

**Request Body**:
```json
{
  "query": "quarterly report",
  "tags": ["finance", "q4"],
  "content_type": "application/pdf",
  "name_pattern": "report",
  "include_deleted": false
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/search/local-storage \
  -H "Content-Type: application/json" \
  -d '{
    "query": "quarterly report",
    "tags": ["finance"],
    "content_type": "application/pdf"
  }'
```

**Response**:
```json
{
  "results": [
    {
      "file_name": "Q4-report.pdf",
      "content_type": "application/pdf",
      "size": 245678,
      "tags": ["finance", "q4"],
      "score": 0.95,
      ...
    }
  ],
  "total": 1,
  "query": {
    "query": "quarterly report",
    "tags": ["finance"],
    "content_type": "application/pdf"
  }
}
```

---

### 7. Tags

#### Update File Tags
```http
PUT /api/v1/file/:storage_name/:file_name/tags
Content-Type: application/json
```

**Request Body**:
```json
{
  "tags": ["important", "legal", "2024"]
}
```

**Example**:
```bash
curl -X PUT http://localhost:3000/api/v1/file/local-storage/contract.pdf/tags \
  -H "Content-Type: application/json" \
  -d '{"tags": ["important", "legal", "2024"]}'
```

**Response**: Updated file metadata

#### List All Tags
```http
GET /api/v1/tags/:storage_name
```

**Example**:
```bash
curl http://localhost:3000/api/v1/tags/local-storage
```

**Response**:
```json
[
  "important",
  "legal",
  "2024",
  "finance",
  "q4",
  "draft"
]
```

---

### 8. Trash / Soft Delete

#### List Trash
```http
GET /api/v1/trash/:storage_name
```

**Example**:
```bash
curl http://localhost:3000/api/v1/trash/local-storage
```

**Response**:
```json
[
  {
    "file_name": "old-report.pdf",
    "is_deleted": true,
    "deleted_at": "2024-01-14T10:00:00Z",
    ...
  }
]
```

#### Restore File from Trash
```http
POST /api/v1/trash/:storage_name/:file_name/restore
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/trash/local-storage/old-report.pdf/restore
```

**Response**: Restored file metadata

#### Empty Trash
```http
DELETE /api/v1/trash/:storage_name
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/trash/local-storage
```

**Response**:
```json
{
  "message": "Trash emptied",
  "files_deleted": 5
}
```

---

### 9. Webhooks

#### Register Webhook
```http
POST /api/v1/webhooks
Content-Type: application/json
```

**Request Body**:
```json
{
  "name": "my-webhook",
  "url": "https://example.com/webhook",
  "events": ["uploaded", "deleted"],
  "headers": {
    "Authorization": "Bearer token123"
  },
  "enabled": true
}
```

**Example**:
```bash
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "slack-notifications",
    "url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
    "events": ["uploaded", "deleted"],
    "enabled": true
  }'
```

**Response**:
```json
{
  "message": "Webhook registered successfully",
  "name": "slack-notifications"
}
```

#### List Webhooks
```http
GET /api/v1/webhooks
```

**Example**:
```bash
curl http://localhost:3000/api/v1/webhooks
```

**Response**:
```json
[
  {
    "name": "slack-notifications",
    "config": {
      "url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
      "events": ["uploaded", "deleted"],
      "headers": {},
      "enabled": true
    }
  }
]
```

#### Unregister Webhook
```http
DELETE /api/v1/webhooks/:name
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/webhooks/slack-notifications
```

**Response**:
```json
{
  "message": "Webhook unregistered successfully"
}
```

**Webhook Payload Format**:
```json
{
  "event": "uploaded",
  "timestamp": "2024-01-15T10:30:00Z",
  "storage_name": "local-storage",
  "file_key": "report.pdf",
  "metadata": {...},
  "user_id": "user123"
}
```

---

### 10. Cache Management

#### Cache Statistics
```http
GET /api/v1/cache/stats
```

**Example**:
```bash
curl http://localhost:3000/api/v1/cache/stats
```

**Response**:
```json
{
  "entry_count": 150,
  "weighted_size": 52428800,
  "hit_rate": 0.0
}
```

#### Invalidate Cache Entry
```http
DELETE /api/v1/cache/:storage_name/:file_name
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/cache/local-storage/report.pdf
```

**Response**:
```json
{
  "message": "Cache entry invalidated"
}
```

#### Clear All Cache
```http
DELETE /api/v1/cache
```

**Example**:
```bash
curl -X DELETE http://localhost:3000/api/v1/cache
```

**Response**:
```json
{
  "message": "Cache cleared",
  "entries_removed": 150
}
```

---

### 11. Presigned URLs (S3 Only)

#### Generate Presigned Download URL
```http
GET /api/v1/file/:storage_name/:file_name/presigned-download?expires_in=3600
```

**Example**:
```bash
curl "http://localhost:3000/api/v1/file/s3-prod/report.pdf/presigned-download?expires_in=7200"
```

**Response**:
```json
{
  "url": "https://bucket.s3.amazonaws.com/files/report.pdf?X-Amz-Algorithm=...",
  "expires_at": "2024-01-15T12:30:00Z"
}
```

#### Generate Presigned Upload URL
```http
GET /api/v1/file/:storage_name/:file_name/presigned-upload?expires_in=3600
```

**Example**:
```bash
curl "http://localhost:3000/api/v1/file/s3-prod/new-file.pdf/presigned-upload?expires_in=3600"
```

**Response**:
```json
{
  "url": "https://bucket.s3.amazonaws.com/files/new-file.pdf?X-Amz-Algorithm=...",
  "expires_at": "2024-01-15T11:30:00Z"
}
```

**Usage**:
```bash
# Upload directly to presigned URL
curl -X PUT "PRESIGNED_URL" --data-binary @local-file.pdf
```

---

## Feature Examples & Use Cases

### 1. File Versioning - Complete Examples

#### Use Case 1: Document Version Control

**Scenario**: Track changes to a legal contract over time

```bash
# 1. Upload initial version
curl -X PUT http://localhost:3000/api/v1/file/legal-docs \
  -F "file=@contract-v1.pdf" \
  -F 'metadata={"client": "Acme Corp", "status": "draft"}'

# 2. Create version 2 with revisions
curl -X POST http://localhost:3000/api/v1/file/legal-docs/contract-v1.pdf/versions \
  -F "file=@contract-v2.pdf"

# 3. Create version 3 with final changes
curl -X POST http://localhost:3000/api/v1/file/legal-docs/contract-v1.pdf/versions \
  -F "file=@contract-v3-final.pdf"

# 4. List all versions to see history
curl http://localhost:3000/api/v1/file/legal-docs/contract-v1.pdf/versions

# 5. Download specific version for review
curl http://localhost:3000/api/v1/file/legal-docs/contract-v1.pdf/versions/VERSION_ID_V2 \
  -o contract-v2-review.pdf

# 6. Restore version 2 if version 3 had errors
curl -X POST http://localhost:3000/api/v1/file/legal-docs/contract-v1.pdf/versions/VERSION_ID_V2/restore
```

#### Use Case 2: Code Release Management

**Scenario**: Manage application binary releases

```bash
# Upload v1.0.0
curl -X PUT http://localhost:3000/api/v1/file/releases \
  -F "file=@app-v1.0.0.tar.gz" \
  -F 'metadata={"version": "1.0.0", "release_date": "2024-01-01"}'

# Upload v1.1.0 as new version
curl -X POST http://localhost:3000/api/v1/file/releases/app-v1.0.0.tar.gz/versions \
  -F "file=@app-v1.1.0.tar.gz"

# Upload v2.0.0
curl -X POST http://localhost:3000/api/v1/file/releases/app-v1.0.0.tar.gz/versions \
  -F "file=@app-v2.0.0.tar.gz"

# Critical bug in v2.0.0 - rollback to v1.1.0
curl -X POST http://localhost:3000/api/v1/file/releases/app-v1.0.0.tar.gz/versions/V1.1.0_UUID/restore

# List all releases
curl http://localhost:3000/api/v1/file/releases/app-v1.0.0.tar.gz/versions | jq '.[].version'
```

#### Use Case 3: Configuration File History

**Scenario**: Track changes to application configuration

```bash
# Initial config
curl -X PUT http://localhost:3000/api/v1/file/configs \
  -F "file=@app-config.yaml" \
  -F 'metadata={"environment": "production", "change_by": "admin"}'

# Update for database migration
curl -X POST http://localhost:3000/api/v1/file/configs/app-config.yaml/versions \
  -F "file=@app-config-db-update.yaml"

# Update for new feature
curl -X POST http://localhost:3000/api/v1/file/configs/app-config.yaml/versions \
  -F "file=@app-config-new-feature.yaml"

# Service down - quickly restore previous working config
VERSION_ID=$(curl http://localhost:3000/api/v1/file/configs/app-config.yaml/versions | jq -r '.[1].version_id')
curl -X POST http://localhost:3000/api/v1/file/configs/app-config.yaml/versions/$VERSION_ID/restore
```

---

### 2. File Sharing - Complete Examples

#### Use Case 1: Client File Delivery

**Scenario**: Share files with clients securely

```bash
# Create password-protected share link valid for 7 days
curl -X POST http://localhost:3000/api/v1/share/client-files \
  -H "Content-Type: application/json" \
  -d '{
    "file_name": "project-deliverables.zip",
    "expires_in_seconds": 604800,
    "max_downloads": 3,
    "password": "Client2024!"
  }'

# Response includes link_id: "abc123def456"
# Send to client: https://yourapi.com/api/v1/share/abc123def456/download?password=Client2024!

# Check download count
curl http://localhost:3000/api/v1/share/abc123def456

# Revoke if needed
curl -X DELETE http://localhost:3000/api/v1/share/abc123def456
```

#### Use Case 2: Temporary Public Download

**Scenario**: Share marketing materials with time limit, no password

```bash
# Create public link valid for 24 hours, unlimited downloads
curl -X POST http://localhost:3000/api/v1/share/marketing \
  -H "Content-Type: application/json" \
  -d '{
    "file_name": "product-brochure.pdf",
    "expires_in_seconds": 86400,
    "max_downloads": null,
    "password": null
  }'

# Response: {"link_id": "xyz789", ...}
# Share link: https://yourapi.com/api/v1/share/xyz789/download

# After event, revoke all marketing shares
curl http://localhost:3000/api/v1/webhooks | \
  jq -r '.[] | select(.name | contains("marketing")) | .name' | \
  xargs -I {} curl -X DELETE http://localhost:3000/api/v1/share/{}
```

#### Use Case 3: One-Time Download Link

**Scenario**: Send sensitive document that can only be downloaded once

```bash
# Create single-use link
curl -X POST http://localhost:3000/api/v1/share/secure-docs \
  -H "Content-Type: application/json" \
  -d '{
    "file_name": "sensitive-report.pdf",
    "expires_in_seconds": 3600,
    "max_downloads": 1,
    "password": "OneTimeUse123"
  }'

# First download works
curl "http://localhost:3000/api/v1/share/LINK_ID/download?password=OneTimeUse123" \
  -o report.pdf

# Second download fails (max_downloads exceeded)
curl "http://localhost:3000/api/v1/share/LINK_ID/download?password=OneTimeUse123"
# Response: 403 Forbidden
```

---

### 3. Bulk Operations - Complete Examples

#### Use Case 1: Batch Upload from Backup

**Scenario**: Restore multiple files from backup

```bash
# Prepare files as base64
FILE1=$(base64 -w 0 backup/file1.pdf)
FILE2=$(base64 -w 0 backup/file2.jpg)
FILE3=$(base64 -w 0 backup/file3.txt)

# Bulk upload
curl -X POST http://localhost:3000/api/v1/bulk/archive/upload \
  -H "Content-Type: application/json" \
  -d "{
    \"files\": [
      {\"name\": \"file1.pdf\", \"content\": \"$FILE1\", \"metadata\": {\"type\": \"document\"}},
      {\"name\": \"file2.jpg\", \"content\": \"$FILE2\", \"metadata\": {\"type\": \"image\"}},
      {\"name\": \"file3.txt\", \"content\": \"$FILE3\", \"metadata\": {\"type\": \"text\"}}
    ]
  }"
```

#### Use Case 2: Batch Download for Export

**Scenario**: Download all files from a project for archival

```bash
# Get list of project files
FILES=$(curl "http://localhost:3000/api/v1/search/projects" \
  -H "Content-Type: application/json" \
  -d '{"query": "project-alpha", "tags": ["alpha"]}' | \
  jq -r '.results[].file_name' | jq -R -s -c 'split("\n") | map(select(length > 0))')

# Bulk download
curl -X POST http://localhost:3000/api/v1/bulk/projects/download \
  -H "Content-Type: application/json" \
  -d "{\"file_names\": $FILES}" | \
  jq -r '.files[] | @base64d' > project-alpha-archive.tar

# Or download individually
echo "$FILES" | jq -r '.[]' | while read file; do
  curl http://localhost:3000/api/v1/file/projects/"$file" -o "export/$file"
done
```

#### Use Case 3: Cleanup Old Files

**Scenario**: Delete files older than 90 days

```bash
# Get old files
OLD_FILES=$(curl http://localhost:3000/api/v1/file/temp-storage | \
  jq -r --arg date "$(date -d '90 days ago' --iso-8601)" \
  '[.[] | select(.created_at < $date) | .file_name]')

# Bulk delete
curl -X POST http://localhost:3000/api/v1/bulk/temp-storage/delete \
  -H "Content-Type: application/json" \
  -d "{\"file_names\": $OLD_FILES}"

# Verify deletion
curl http://localhost:3000/api/v1/trash/temp-storage | jq length
```

---

### 4. Search - Complete Examples

#### Use Case 1: Find Documents by Content

**Scenario**: Search for financial reports from Q4

```bash
# Search with multiple criteria
curl -X POST http://localhost:3000/api/v1/search/finance \
  -H "Content-Type: application/json" \
  -d '{
    "query": "quarterly revenue profit",
    "tags": ["finance", "q4", "2024"],
    "content_type": "application/pdf",
    "name_pattern": "report",
    "include_deleted": false
  }' | jq '.results[] | {name: .file_name, score: .score, tags: .tags}'
```

#### Use Case 2: Find Images by Tag

**Scenario**: Locate all product photos for specific category

```bash
# Search images with product tags
curl -X POST http://localhost:3000/api/v1/search/media \
  -H "Content-Type: application/json" \
  -d '{
    "tags": ["product", "electronics", "phones"],
    "content_type": "image/jpeg"
  }' | jq '.results[] | .file_name'
```

#### Use Case 3: Search Including Deleted Files

**Scenario**: Find accidentally deleted file to restore

```bash
# Search including trash
curl -X POST http://localhost:3000/api/v1/search/documents \
  -H "Content-Type: application/json" \
  -d '{
    "query": "important contract",
    "include_deleted": true
  }' | jq '.results[] | select(.is_deleted == true) | {name: .file_name, deleted_at: .deleted_at}'

# Restore found file
curl -X POST http://localhost:3000/api/v1/trash/documents/important-contract.pdf/restore
```

---

### 5. Tags - Complete Examples

#### Use Case 1: Organize Project Files

**Scenario**: Tag files by project and status

```bash
# Upload and tag as draft
curl -X PUT http://localhost:3000/api/v1/file/projects \
  -F "file=@design-doc.pdf"

curl -X PUT http://localhost:3000/api/v1/file/projects/design-doc.pdf/tags \
  -H "Content-Type: application/json" \
  -d '{"tags": ["project-alpha", "design", "draft"]}'

# Update to final
curl -X PUT http://localhost:3000/api/v1/file/projects/design-doc.pdf/tags \
  -H "Content-Type: application/json" \
  -d '{"tags": ["project-alpha", "design", "final", "approved"]}'

# Find all final project-alpha documents
curl -X POST http://localhost:3000/api/v1/search/projects \
  -H "Content-Type: application/json" \
  -d '{"tags": ["project-alpha", "final"]}'
```

#### Use Case 2: Tag-Based Retention Policy

**Scenario**: Implement retention using tags

```bash
# Tag for auto-delete
curl -X PUT http://localhost:3000/api/v1/file/temp/cache-file.json/tags \
  -H "Content-Type: application/json" \
  -d '{"tags": ["temp", "auto-delete-30d", "cache"]}'

# Periodic cleanup script
curl -X POST http://localhost:3000/api/v1/search/temp \
  -H "Content-Type: application/json" \
  -d '{"tags": ["auto-delete-30d"]}' | \
  jq -r --arg date "$(date -d '30 days ago' --iso-8601)" \
  '[.results[] | select(.created_at < $date) | .file_name]' | \
  curl -X POST http://localhost:3000/api/v1/bulk/temp/delete \
    -H "Content-Type: application/json" \
    -d @-
```

#### Use Case 3: Tag Analytics

**Scenario**: Analyze file categorization

```bash
# Get all tags
curl http://localhost:3000/api/v1/tags/all-storage | jq -r '.[]' | sort | uniq -c

# Count files per tag
for tag in $(curl http://localhost:3000/api/v1/tags/all-storage | jq -r '.[]'); do
  count=$(curl -X POST http://localhost:3000/api/v1/search/all-storage \
    -H "Content-Type: application/json" \
    -d "{\"tags\": [\"$tag\"]}" | jq '.total')
  echo "$tag: $count files"
done
```

---

### 6. Trash / Soft Delete - Complete Examples

#### Use Case 1: Accidental Deletion Recovery

**Scenario**: User accidentally deleted important file

```bash
# User deletes file
curl -X DELETE http://localhost:3000/api/v1/file/user-docs/important.pdf

# File is soft-deleted, check trash
curl http://localhost:3000/api/v1/trash/user-docs | \
  jq '.[] | select(.file_name == "important.pdf")'

# Restore within recovery window
curl -X POST http://localhost:3000/api/v1/trash/user-docs/important.pdf/restore

# Verify restoration
curl http://localhost:3000/api/v1/file/user-docs/important.pdf/metadata | \
  jq '{name: .file_name, deleted: .is_deleted}'
```

#### Use Case 2: Scheduled Trash Cleanup

**Scenario**: Permanently delete files in trash after 30 days

```bash
# List trash items older than 30 days
curl http://localhost:3000/api/v1/trash/archive | \
  jq -r --arg date "$(date -d '30 days ago' --iso-8601)" \
  '.[] | select(.deleted_at < $date) | .file_name'

# Empty entire trash
curl -X DELETE http://localhost:3000/api/v1/trash/archive

# Or selectively delete old items
# (Would need custom endpoint for date-based permanent deletion)
```

#### Use Case 3: Audit Deleted Files

**Scenario**: Review what was deleted for compliance

```bash
# Export deleted files report
curl http://localhost:3000/api/v1/trash/compliance-docs | \
  jq '[.[] | {
    file: .file_name,
    deleted_at: .deleted_at,
    size: .size,
    original_tags: .tags,
    metadata: .custom
  }]' > deleted-files-report.json

# Count deletions by date
curl http://localhost:3000/api/v1/trash/compliance-docs | \
  jq -r '.[].deleted_at | split("T")[0]' | sort | uniq -c
```

---

### 7. Webhooks - Complete Examples

#### Use Case 1: Slack Notifications

**Scenario**: Notify Slack channel on file uploads

```bash
# Register Slack webhook
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "slack-uploads",
    "url": "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXX",
    "events": ["uploaded"],
    "enabled": true
  }'

# Upload triggers notification
curl -X PUT http://localhost:3000/api/v1/file/shared \
  -F "file=@presentation.pptx"

# Slack receives:
# {
#   "event": "uploaded",
#   "timestamp": "2024-01-15T10:30:00Z",
#   "storage_name": "shared",
#   "file_key": "presentation.pptx",
#   "metadata": {...}
# }
```

#### Use Case 2: Audit Logging

**Scenario**: Log all file operations to external system

```bash
# Register audit webhook
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "audit-logger",
    "url": "https://audit.example.com/file-events",
    "events": ["uploaded", "downloaded", "deleted", "restored"],
    "headers": {
      "Authorization": "Bearer audit-token-123",
      "X-System": "file-api"
    },
    "enabled": true
  }'

# All operations now logged
curl -X PUT http://localhost:3000/api/v1/file/docs -F "file=@doc.pdf"
curl http://localhost:3000/api/v1/file/docs/doc.pdf
curl -X DELETE http://localhost:3000/api/v1/file/docs/doc.pdf
```

#### Use Case 3: Automated Processing Pipeline

**Scenario**: Trigger processing when files uploaded

```bash
# Register processing webhook
curl -X POST http://localhost:3000/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "image-processor",
    "url": "https://processing.example.com/process-image",
    "events": ["uploaded"],
    "enabled": true
  }'

# Upload image triggers:
# 1. Webhook notification
# 2. Processing service downloads image
# 3. Generates thumbnails
# 4. Extracts metadata
# 5. Updates tags via API

curl -X PUT http://localhost:3000/api/v1/file/images \
  -F "file=@photo.jpg"

# External service processes and updates
curl -X PUT http://localhost:3000/api/v1/file/images/photo.jpg/tags \
  -H "Content-Type: application/json" \
  -d '{"tags": ["processed", "thumbnail-generated", "faces-detected"]}'
```

---

### 8. Caching - Complete Examples

#### Use Case 1: High-Traffic File Serving

**Scenario**: Optimize frequently accessed files

```bash
# First request - cache miss
time curl http://localhost:3000/api/v1/file/public/logo.png -o /dev/null
# Time: 250ms

# Second request - cache hit
time curl http://localhost:3000/api/v1/file/public/logo.png -o /dev/null
# Time: 5ms

# Check cache stats
curl http://localhost:3000/api/v1/cache/stats
# {
#   "entry_count": 150,
#   "weighted_size": 52428800,
#   "hit_rate": 0.85
# }
```

#### Use Case 2: Cache Invalidation on Update

**Scenario**: Clear cache when file updated

```bash
# Update file
curl -X PUT http://localhost:3000/api/v1/file/config \
  -F "file=@app-config.yaml"

# Invalidate cache to force reload
curl -X DELETE http://localhost:3000/api/v1/cache/config/app-config.yaml

# Next request gets fresh data
curl http://localhost:3000/api/v1/file/config/app-config.yaml
```

#### Use Case 3: Cache Management

**Scenario**: Monitor and manage cache size

```bash
# Check current cache size
curl http://localhost:3000/api/v1/cache/stats | \
  jq '{entries: .entry_count, size_mb: (.weighted_size / 1024 / 1024)}'

# Cache too large - clear all
curl -X DELETE http://localhost:3000/api/v1/cache

# Or selectively clear by storage
for file in $(curl http://localhost:3000/api/v1/file/temp | jq -r '.[].file_name'); do
  curl -X DELETE http://localhost:3000/api/v1/cache/temp/"$file"
done
```

---

### 9. Health Checks - Complete Examples

#### Use Case 1: Load Balancer Health Check

**Scenario**: Simple health endpoint for HAProxy/nginx

```bash
# Basic health check
curl http://localhost:3000/health
# Response: {"status": "healthy"}

# Use in HAProxy config:
# option httpchk GET /health
# http-check expect status 200
```

#### Use Case 2: Comprehensive Monitoring

**Scenario**: Detailed health status for all storage backends

```bash
# Full system health
curl http://localhost:3000/health/all | jq '.'
# {
#   "status": "Healthy",
#   "uptime_seconds": 86400,
#   "storages": {
#     "s3-prod": {"status": "Healthy", "response_time_ms": 120},
#     "postgres-main": {"status": "Healthy", "response_time_ms": 15},
#     "redis-cache": {"status": "Degraded", "response_time_ms": 5500}
#   }
# }

# Alert if any unhealthy
STATUS=$(curl -s http://localhost:3000/health/all | jq -r '.status')
if [ "$STATUS" != "Healthy" ]; then
  # Send alert
  curl -X POST https://alerts.example.com/incident \
    -d '{"message": "File API health check failed"}'
fi
```

#### Use Case 3: Per-Storage Monitoring

**Scenario**: Monitor critical storage separately

```bash
# Check production S3
curl http://localhost:3000/health/s3-prod | \
  jq '{status: .status, latency: .response_time_ms}'

# Create monitoring dashboard
while true; do
  for storage in s3-prod postgres-main redis-cache; do
    health=$(curl -s http://localhost:3000/health/$storage)
    echo "$(date) - $storage: $(echo $health | jq -r '.status') - $(echo $health | jq -r '.response_time_ms')ms"
  done
  sleep 60
done
```

---

### 10. Metrics - Complete Examples

#### Use Case 1: Prometheus Integration

**Scenario**: Scrape metrics into Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'pmp-file-api'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

```bash
# View raw metrics
curl http://localhost:3000/metrics

# Example output:
# file_upload_total{storage="s3-prod"} 1234
# file_download_total{storage="s3-prod"} 5678
# file_size_bytes_sum{storage="s3-prod"} 10485760
```

#### Use Case 2: Custom Alerting

**Scenario**: Alert on high error rate

```bash
# Query metrics
ERROR_RATE=$(curl -s http://localhost:3000/metrics | \
  grep 'file_error_total' | \
  awk '{sum+=$NF} END {print sum}')

TOTAL_OPS=$(curl -s http://localhost:3000/metrics | \
  grep -E 'file_(upload|download)_total' | \
  awk '{sum+=$NF} END {print sum}')

# Calculate error rate
ERROR_PERCENT=$(echo "scale=2; $ERROR_RATE / $TOTAL_OPS * 100" | bc)

if (( $(echo "$ERROR_PERCENT > 5" | bc -l) )); then
  echo "Alert: Error rate ${ERROR_PERCENT}% exceeds threshold"
fi
```

#### Use Case 3: Grafana Dashboard

**Scenario**: Visualize file operations

```bash
# Prometheus queries for Grafana:

# Upload rate
rate(file_upload_total[5m])

# Download rate by storage
sum by (storage) (rate(file_download_total[5m]))

# Average file size
rate(file_size_bytes_sum[5m]) / rate(file_size_bytes_count[5m])

# Error rate percentage
rate(file_error_total[5m]) / rate(file_operation_total[5m]) * 100

# Storage latency p95
histogram_quantile(0.95, rate(storage_operation_duration_seconds_bucket[5m]))
```

---

## Integration Examples

### Example 1: JavaScript/TypeScript Client

```typescript
// file-api-client.ts
import axios, { AxiosInstance } from 'axios';

class FileAPIClient {
  private client: AxiosInstance;

  constructor(baseURL: string) {
    this.client = axios.create({
      baseURL,
      timeout: 30000,
    });
  }

  // Upload file
  async upload(storage: string, file: File, metadata?: object) {
    const formData = new FormData();
    formData.append('file', file);
    if (metadata) {
      formData.append('metadata', JSON.stringify(metadata));
    }

    const response = await this.client.put(
      `/api/v1/file/${storage}`,
      formData,
      {
        headers: { 'Content-Type': 'multipart/form-data' },
      }
    );
    return response.data;
  }

  // Download file
  async download(storage: string, fileName: string) {
    const response = await this.client.get(
      `/api/v1/file/${storage}/${fileName}`,
      { responseType: 'blob' }
    );
    return response.data;
  }

  // Search files
  async search(storage: string, query: {
    query?: string;
    tags?: string[];
    content_type?: string;
  }) {
    const response = await this.client.post(
      `/api/v1/search/${storage}`,
      query
    );
    return response.data;
  }

  // Create share link
  async createShare(storage: string, options: {
    file_name: string;
    expires_in_seconds?: number;
    max_downloads?: number;
    password?: string;
  }) {
    const response = await this.client.post(
      `/api/v1/share/${storage}`,
      options
    );
    return response.data;
  }

  // Bulk upload
  async bulkUpload(storage: string, files: Array<{
    name: string;
    content: string; // base64
    metadata?: object;
  }>) {
    const response = await this.client.post(
      `/api/v1/bulk/${storage}/upload`,
      { files }
    );
    return response.data;
  }
}

// Usage
const api = new FileAPIClient('http://localhost:3000');

// Upload
const file = document.querySelector('input[type="file"]').files[0];
const result = await api.upload('my-storage', file, {
  project: 'alpha',
  author: 'john'
});

// Search
const results = await api.search('my-storage', {
  query: 'report',
  tags: ['finance', 'q4']
});

// Create share link
const share = await api.createShare('my-storage', {
  file_name: 'report.pdf',
  expires_in_seconds: 86400,
  password: 'secret123'
});
console.log(`Share link: ${share.link_id}`);
```

### Example 2: Python Client

```python
# file_api_client.py
import requests
import base64
from typing import Optional, List, Dict, Any

class FileAPIClient:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()

    def upload(self, storage: str, file_path: str,
               metadata: Optional[Dict] = None) -> Dict[str, Any]:
        """Upload file to storage"""
        with open(file_path, 'rb') as f:
            files = {'file': f}
            data = {}
            if metadata:
                data['metadata'] = metadata

            response = self.session.put(
                f'{self.base_url}/api/v1/file/{storage}',
                files=files,
                data=data
            )
            response.raise_for_status()
            return response.json()

    def download(self, storage: str, file_name: str,
                 output_path: str) -> None:
        """Download file from storage"""
        response = self.session.get(
            f'{self.base_url}/api/v1/file/{storage}/{file_name}'
        )
        response.raise_for_status()

        with open(output_path, 'wb') as f:
            f.write(response.content)

    def search(self, storage: str, query: Optional[str] = None,
               tags: Optional[List[str]] = None,
               content_type: Optional[str] = None) -> Dict[str, Any]:
        """Search files"""
        payload = {}
        if query:
            payload['query'] = query
        if tags:
            payload['tags'] = tags
        if content_type:
            payload['content_type'] = content_type

        response = self.session.post(
            f'{self.base_url}/api/v1/search/{storage}',
            json=payload
        )
        response.raise_for_status()
        return response.json()

    def create_share(self, storage: str, file_name: str,
                     expires_in_seconds: int = 3600,
                     max_downloads: Optional[int] = None,
                     password: Optional[str] = None) -> Dict[str, Any]:
        """Create share link"""
        payload = {
            'file_name': file_name,
            'expires_in_seconds': expires_in_seconds
        }
        if max_downloads:
            payload['max_downloads'] = max_downloads
        if password:
            payload['password'] = password

        response = self.session.post(
            f'{self.base_url}/api/v1/share/{storage}',
            json=payload
        )
        response.raise_for_status()
        return response.json()

    def bulk_upload(self, storage: str,
                    files: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Bulk upload files (base64 encoded)"""
        # Convert file paths to base64
        encoded_files = []
        for file_info in files:
            with open(file_info['path'], 'rb') as f:
                content = base64.b64encode(f.read()).decode('utf-8')
                encoded_files.append({
                    'name': file_info.get('name', file_info['path']),
                    'content': content,
                    'metadata': file_info.get('metadata', {})
                })

        response = self.session.post(
            f'{self.base_url}/api/v1/bulk/{storage}/upload',
            json={'files': encoded_files}
        )
        response.raise_for_status()
        return response.json()

# Usage
api = FileAPIClient('http://localhost:3000')

# Upload
result = api.upload('my-storage', 'report.pdf',
                    metadata={'project': 'alpha'})

# Search
results = api.search('my-storage', query='report',
                     tags=['finance', 'q4'])

# Bulk upload
api.bulk_upload('my-storage', [
    {'path': 'file1.pdf', 'metadata': {'type': 'doc'}},
    {'path': 'file2.jpg', 'metadata': {'type': 'image'}}
])

# Create share
share = api.create_share('my-storage', 'report.pdf',
                         expires_in_seconds=86400,
                         password='secret')
print(f"Share ID: {share['link_id']}")
```

### Example 3: Automated Backup Script

```bash
#!/bin/bash
# backup.sh - Automated file backup with versioning

API_BASE="http://localhost:3000"
STORAGE="backup-storage"
BACKUP_DIR="/data/to/backup"

# Function to upload file with version
backup_file() {
  local file=$1
  local filename=$(basename "$file")

  echo "Backing up: $filename"

  # Check if file exists (create version if exists)
  if curl -s -f "$API_BASE/api/v1/file/$STORAGE/$filename/metadata" > /dev/null; then
    # Create new version
    curl -X POST "$API_BASE/api/v1/file/$STORAGE/$filename/versions" \
      -F "file=@$file" \
      -s | jq -r '.version_id'
  else
    # Initial upload
    curl -X PUT "$API_BASE/api/v1/file/$STORAGE" \
      -F "file=@$file" \
      -F "metadata={\"backup_date\": \"$(date --iso-8601)\", \"source\": \"$file\"}" \
      -s | jq -r '.version_id'
  fi
}

# Backup all files
find "$BACKUP_DIR" -type f | while read file; do
  backup_file "$file"
done

# Cleanup old versions (keep last 10)
curl -s "$API_BASE/api/v1/file/$STORAGE" | \
  jq -r '.[].file_name' | \
  while read filename; do
    versions=$(curl -s "$API_BASE/api/v1/file/$STORAGE/$filename/versions" | jq -r '.[].version_id')
    count=$(echo "$versions" | wc -l)

    if [ $count -gt 10 ]; then
      # Delete old versions (keep last 10)
      echo "$versions" | head -n $((count - 10)) | while read version_id; do
        echo "Cleaning old version: $version_id"
        # Would need delete version endpoint
      done
    fi
  done

echo "Backup completed at $(date)"
```

---

## Best Practices

### 1. Storage Selection

- **S3**: Production deployments, large files, CDN integration
- **Local**: Development, single-server, fast access
- **PostgreSQL**: Transactional integrity, complex queries
- **MySQL**: Traditional RDBMS environments
- **SQLite**: Serverless, edge computing, embedded apps
- **Redis**: Temporary storage, caching, session data
- **Azure**: Azure cloud deployments
- **GCS**: Google Cloud deployments

### 2. Security Best Practices

```bash
# Use HTTPS in production
# Enable authentication (implement JWT/OAuth)
# Implement API key validation
# Use strong passwords for share links
# Set reasonable expiration times
# Enable rate limiting
# Regular security audits
# Encrypt sensitive metadata
```

### 3. Performance Optimization

```bash
# Enable caching for frequently accessed files
curl http://localhost:3000/api/v1/cache/stats

# Use bulk operations for multiple files
# Implement pagination for large file lists
# Use presigned URLs for large uploads/downloads
# Monitor metrics for bottlenecks
curl http://localhost:3000/metrics | grep latency
```

### 4. Data Management

```bash
# Implement retention policies
# Regular trash cleanup
curl -X DELETE http://localhost:3000/api/v1/trash/storage

# Use tags for organization
# Enable deduplication for storage savings
# Monitor storage usage via metrics
# Implement versioning for critical files
```

### 5. Monitoring & Alerting

```bash
# Set up Prometheus + Grafana
# Monitor health endpoints
# Configure webhook notifications
# Track error rates
# Set up alerts for:
#   - Storage failures
#   - High error rates
#   - Slow response times
#   - Disk space issues
```

---

## Performance & Scalability

### Performance Characteristics

- **Upload**: ~50-100 MB/s (network limited)
- **Download**: ~100-200 MB/s (network/storage limited)
- **Metadata operations**: <10ms (cached)
- **Search**: <100ms (small datasets), <500ms (large datasets)
- **Bulk operations**: 10-50 files/second

### Scaling Recommendations

1. **Horizontal Scaling**: Deploy multiple instances behind load balancer
2. **Database Scaling**: Use read replicas for metadata queries
3. **Cache Layer**: Redis for distributed caching
4. **Storage Tiering**: Hot data (Redis) → Warm data (S3) → Cold data (Glacier)
5. **CDN Integration**: Use presigned URLs with CloudFront/Fastly

---

## Troubleshooting

### Common Issues

#### 1. Upload Fails
```bash
# Check storage health
curl http://localhost:3000/health/storage-name

# Check disk space (local storage)
df -h

# Verify credentials (cloud storage)
# Check rate limiting
curl http://localhost:3000/metrics | grep rate_limit
```

#### 2. Slow Performance
```bash
# Check cache hit rate
curl http://localhost:3000/api/v1/cache/stats

# Monitor response times
curl http://localhost:3000/health/all | jq '.storages[].response_time_ms'

# Check database connections (if using SQL storage)
```

#### 3. Search Not Working
```bash
# Verify file has searchable metadata
curl http://localhost:3000/api/v1/file/storage/file.pdf/metadata | jq '.custom'

# Check if file is deleted
curl http://localhost:3000/api/v1/file/storage/file.pdf/metadata | jq '.is_deleted'

# Search with include_deleted
curl -X POST http://localhost:3000/api/v1/search/storage \
  -H "Content-Type: application/json" \
  -d '{"query": "term", "include_deleted": true}'
```

---

## Support & Contributing

For issues, questions, or contributions, visit:
- GitHub: https://github.com/yourusername/pmp-file-api
- Documentation: https://docs.pmp-file-api.example.com

---

**Document Version**: 1.0.0
**Last Updated**: 2024-01-15
**API Version**: v1
