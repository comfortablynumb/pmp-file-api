-- Migration to add advanced features support to PostgreSQL
-- Add versioning, tags, soft delete, and deduplication columns

-- Add versioning columns
ALTER TABLE files ADD COLUMN IF NOT EXISTS version INTEGER DEFAULT 1;
ALTER TABLE files ADD COLUMN IF NOT EXISTS version_id UUID;
ALTER TABLE files ADD COLUMN IF NOT EXISTS parent_version_id UUID;

-- Add tags support (array of text)
ALTER TABLE files ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';

-- Add soft delete support
ALTER TABLE files ADD COLUMN IF NOT EXISTS is_deleted BOOLEAN DEFAULT FALSE;
ALTER TABLE files ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP;

-- Add deduplication support
ALTER TABLE files ADD COLUMN IF NOT EXISTS content_hash TEXT;

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_files_version_id ON files(version_id);
CREATE INDEX IF NOT EXISTS idx_files_parent_version_id ON files(parent_version_id);
CREATE INDEX IF NOT EXISTS idx_files_is_deleted ON files(is_deleted);
CREATE INDEX IF NOT EXISTS idx_files_content_hash ON files(content_hash);
CREATE INDEX IF NOT EXISTS idx_files_tags ON files USING GIN(tags);

-- Update existing rows to have default values
UPDATE files SET version = 1 WHERE version IS NULL;
UPDATE files SET version_id = gen_random_uuid() WHERE version_id IS NULL;
UPDATE files SET tags = '{}' WHERE tags IS NULL;
UPDATE files SET is_deleted = FALSE WHERE is_deleted IS NULL;
