use async_trait::async_trait;
use bytes::Bytes;
use sqlx::mysql::MySqlPool;
use sqlx::Row;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct MySqlStorage {
    pool: MySqlPool,
}

impl MySqlStorage {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = MySqlPool::connect(connection_string)
            .await
            .map_err(|e| ApiError::Storage(format!("Failed to connect to MySQL: {}", e)))?;

        // Create table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id INT AUTO_INCREMENT PRIMARY KEY,
                file_name VARCHAR(255) NOT NULL UNIQUE,
                content_type VARCHAR(255),
                size BIGINT NOT NULL,
                data LONGBLOB NOT NULL,
                metadata JSON NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                INDEX idx_file_name (file_name)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to create table: {}", e)))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for MySqlStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let metadata_json = serde_json::to_value(&metadata.custom)?;

        sqlx::query(
            r#"
            INSERT INTO files (file_name, content_type, size, data, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                content_type = VALUES(content_type),
                size = VALUES(size),
                data = VALUES(data),
                metadata = VALUES(metadata),
                updated_at = VALUES(updated_at)
            "#,
        )
        .bind(key)
        .bind(&metadata.content_type)
        .bind(metadata.size as i64)
        .bind(data.as_ref())
        .bind(&metadata_json)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
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
        let custom: serde_json::Value = row.get("metadata");
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");

        let metadata = FileMetadata {
            file_name,
            content_type,
            size: size as u64,
            created_at,
            updated_at,
            custom,
        };

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
            let metadata = FileMetadata {
                file_name: row.get("file_name"),
                content_type: row.get("content_type"),
                size: row.get::<i64, _>("size") as u64,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                custom: row.get("metadata"),
            };
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

        Ok(FileMetadata {
            file_name: row.get("file_name"),
            content_type: row.get("content_type"),
            size: row.get::<i64, _>("size") as u64,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            custom: row.get("metadata"),
        })
    }
}
