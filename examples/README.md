# PMP File API - Examples

This directory contains practical examples for using the PMP File API in various programming languages and scenarios.

## Directory Structure

```
examples/
├── bash/           # Bash/shell script examples
├── python/         # Python client examples
├── typescript/     # TypeScript/JavaScript examples
├── rust/           # Rust client examples
└── use-cases/      # Complete use case implementations
```

## Quick Start Examples

### Bash

```bash
# Run bash examples
cd examples/bash
./basic-operations.sh
./versioning-example.sh
./bulk-operations.sh
```

### Python

```bash
# Install dependencies
pip install requests

# Run Python examples
cd examples/python
python basic_client.py
python versioning_demo.py
python bulk_upload.py
```

### TypeScript

```bash
# Install dependencies
npm install axios form-data

# Run TypeScript examples
cd examples/typescript
ts-node file-api-client.ts
ts-node share-link-demo.ts
```

## Example Categories

### 1. Basic Operations
- Upload files
- Download files
- List files
- Delete files
- Get metadata

### 2. Advanced Features
- File versioning
- Share links with expiration
- Bulk operations
- Search and filtering
- Tag management

### 3. Enterprise Use Cases
- Document management system
- Photo gallery with sharing
- Backup and versioning system
- Multi-tenant file storage
- CDN integration with presigned URLs

### 4. Integration Examples
- Webhook integration
- Prometheus monitoring
- Health check monitoring
- Cache optimization
- Event-driven workflows

## Running Examples

Each example directory contains a README with specific instructions. Most examples assume the API is running on `http://localhost:3000`.

### Start the API

```bash
# In the project root
cargo run --release
```

### Run Examples

```bash
# Choose an example directory
cd examples/bash

# Make scripts executable
chmod +x *.sh

# Run an example
./basic-operations.sh
```

## Environment Variables

Set these environment variables before running examples:

```bash
export API_BASE_URL=http://localhost:3000
export STORAGE_NAME=local-storage
export API_KEY=your_api_key  # If authentication is enabled
```

## Example Files

- `basic-operations.*` - Simple CRUD operations
- `versioning-example.*` - File versioning workflow
- `share-link-demo.*` - Creating and using share links
- `bulk-operations.*` - Uploading/downloading multiple files
- `search-demo.*` - Searching files with filters
- `webhook-integration.*` - Setting up webhooks
- `backup-script.*` - Automated backup with versioning
- `photo-gallery.*` - Complete photo management system

## Documentation

For complete API reference, see [DOCUMENTATION.md](../DOCUMENTATION.md)

## Contributing

Feel free to contribute more examples! Submit a PR with:
- Clear use case description
- Well-commented code
- README with usage instructions
- Sample output/screenshots
