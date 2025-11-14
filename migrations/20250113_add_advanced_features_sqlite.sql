-- Migration to add advanced features support to SQLite
-- Add versioning, tags, soft delete, and deduplication columns

-- Add versioning columns
ALTER TABLE files ADD COLUMN version INTEGER DEFAULT 1;
ALTER TABLE files ADD COLUMN version_id TEXT;
ALTER TABLE files ADD COLUMN parent_version_id TEXT;

-- Add tags support (JSON text)
ALTER TABLE files ADD COLUMN tags TEXT DEFAULT '[]';

-- Add soft delete support
ALTER TABLE files ADD COLUMN is_deleted INTEGER DEFAULT 0;
ALTER TABLE files ADD COLUMN deleted_at TEXT;

-- Add deduplication support
ALTER TABLE files ADD COLUMN content_hash TEXT;

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_files_version_id ON files(version_id);
CREATE INDEX IF NOT EXISTS idx_files_parent_version_id ON files(parent_version_id);
CREATE INDEX IF NOT EXISTS idx_files_is_deleted ON files(is_deleted);
CREATE INDEX IF NOT EXISTS idx_files_content_hash ON files(content_hash);

-- Update existing rows to have default values
UPDATE files SET version = 1 WHERE version IS NULL;
UPDATE files SET tags = '[]' WHERE tags IS NULL;
UPDATE files SET is_deleted = 0 WHERE is_deleted IS NULL;
