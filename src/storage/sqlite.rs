use async_trait::async_trait;
use bytes::Bytes;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to connect to SQLite: {}", e)))?;

        // Create table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_name TEXT NOT NULL UNIQUE,
                content_type TEXT,
                size INTEGER NOT NULL,
                data BLOB NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to create table: {}", e)))?;

        // Create index on file_name for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_files_file_name ON files(file_name)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to create index: {}", e)))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let metadata_json = serde_json::to_string(&metadata.custom)?;

        sqlx::query(
            r#"
            INSERT INTO files (file_name, content_type, size, data, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(file_name) DO UPDATE SET
                content_type = excluded.content_type,
                size = excluded.size,
                data = excluded.data,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(&metadata.content_type)
        .bind(metadata.size as i64)
        .bind(data.as_ref())
        .bind(&metadata_json)
        .bind(metadata.created_at.to_rfc3339())
        .bind(metadata.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to insert file: {}", e)))?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let row = sqlx::query(
            r#"
            SELECT file_name, content_type, size, data, metadata, created_at, updated_at
            FROM files
            WHERE file_name = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to fetch file: {}", e)))?
        .ok_or_else(|| ApiError::FileNotFound(key.to_string()))?;

        let file_name: String = row.get("file_name");
        let content_type: Option<String> = row.get("content_type");
        let size: i64 = row.get("size");
        let data: Vec<u8> = row.get("data");
        let metadata_str: String = row.get("metadata");
        let custom: serde_json::Value = serde_json::from_str(&metadata_str)?;
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ApiError::Storage(format!("Failed to parse created_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ApiError::Storage(format!("Failed to parse updated_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        let mut metadata = FileMetadata::new(file_name, size as u64);
        metadata.content_type = content_type;
        metadata.created_at = created_at;
        metadata.updated_at = updated_at;
        metadata.custom = custom;

        Ok((Bytes::from(data), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM files WHERE file_name = ?
            "#,
        )
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to delete file: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(ApiError::FileNotFound(key.to_string()));
        }

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>> {
        let rows = if let Some(prefix) = prefix {
            sqlx::query(
                r#"
                SELECT file_name, content_type, size, metadata, created_at, updated_at
                FROM files
                WHERE file_name LIKE ?
                ORDER BY created_at DESC
                "#,
            )
            .bind(format!("{}%", prefix))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT file_name, content_type, size, metadata, created_at, updated_at
                FROM files
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| ApiError::Storage(format!("Failed to list files: {}", e)))?;

        let mut metadata_list = Vec::new();
        for row in rows {
            let metadata_str: String = row.get("metadata");
            let custom: serde_json::Value = serde_json::from_str(&metadata_str)?;
            let created_at_str: String = row.get("created_at");
            let updated_at_str: String = row.get("updated_at");

            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| ApiError::Storage(format!("Failed to parse created_at: {}", e)))?
                .with_timezone(&chrono::Utc);

            let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| ApiError::Storage(format!("Failed to parse updated_at: {}", e)))?
                .with_timezone(&chrono::Utc);

            let file_name: String = row.get("file_name");
            let size: i64 = row.get("size");
            let mut metadata = FileMetadata::new(file_name, size as u64);
            metadata.content_type = row.get("content_type");
            metadata.created_at = created_at;
            metadata.updated_at = updated_at;
            metadata.custom = custom;
            metadata_list.push(metadata);
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM files WHERE file_name = ?
            "#,
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to check existence: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        let row = sqlx::query(
            r#"
            SELECT file_name, content_type, size, metadata, created_at, updated_at
            FROM files
            WHERE file_name = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to fetch metadata: {}", e)))?
        .ok_or_else(|| ApiError::FileNotFound(key.to_string()))?;

        let metadata_str: String = row.get("metadata");
        let custom: serde_json::Value = serde_json::from_str(&metadata_str)?;
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| ApiError::Storage(format!("Failed to parse created_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| ApiError::Storage(format!("Failed to parse updated_at: {}", e)))?
            .with_timezone(&chrono::Utc);

        let file_name: String = row.get("file_name");
        let size: i64 = row.get("size");
        let mut metadata = FileMetadata::new(file_name, size as u64);
        metadata.content_type = row.get("content_type");
        metadata.created_at = created_at;
        metadata.updated_at = updated_at;
        metadata.custom = custom;

        Ok(metadata)
    }
}
