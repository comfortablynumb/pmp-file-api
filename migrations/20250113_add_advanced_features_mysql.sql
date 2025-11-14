-- Migration to add advanced features support to MySQL
-- Add versioning, tags, soft delete, and deduplication columns

-- Add versioning columns
ALTER TABLE files ADD COLUMN version INT DEFAULT 1;
ALTER TABLE files ADD COLUMN version_id CHAR(36);
ALTER TABLE files ADD COLUMN parent_version_id CHAR(36);

-- Add tags support (JSON array)
ALTER TABLE files ADD COLUMN tags JSON;

-- Add soft delete support
ALTER TABLE files ADD COLUMN is_deleted BOOLEAN DEFAULT FALSE;
ALTER TABLE files ADD COLUMN deleted_at TIMESTAMP NULL;

-- Add deduplication support
ALTER TABLE files ADD COLUMN content_hash VARCHAR(64);

-- Create indexes for better performance
CREATE INDEX idx_files_version_id ON files(version_id);
CREATE INDEX idx_files_parent_version_id ON files(parent_version_id);
CREATE INDEX idx_files_is_deleted ON files(is_deleted);
CREATE INDEX idx_files_content_hash ON files(content_hash);

-- Update existing rows to have default values
UPDATE files SET version = 1 WHERE version IS NULL;
UPDATE files SET version_id = UUID() WHERE version_id IS NULL;
UPDATE files SET tags = JSON_ARRAY() WHERE tags IS NULL;
UPDATE files SET is_deleted = FALSE WHERE is_deleted IS NULL;
