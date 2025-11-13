use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{ApiError, Result};
use crate::metadata::FileMetadata;
use crate::storage::Storage;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path).await?;

        Ok(Self { base_path })
    }

    fn get_file_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    fn get_metadata_path(&self, key: &str) -> PathBuf {
        self.base_path.join(format!("{}.metadata.json", key))
    }

    async fn read_metadata(&self, key: &str) -> Result<FileMetadata> {
        let metadata_path = self.get_metadata_path(key);
        let content = fs::read_to_string(metadata_path).await?;
        let metadata: FileMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    async fn write_metadata(&self, key: &str, metadata: &FileMetadata) -> Result<()> {
        let metadata_path = self.get_metadata_path(key);

        // Ensure parent directory exists
        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(metadata)?;
        let mut file = fs::File::create(metadata_path).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn put(&self, key: &str, data: Bytes, metadata: FileMetadata) -> Result<()> {
        let file_path = self.get_file_path(key);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file
        let mut file = fs::File::create(file_path).await?;
        file.write_all(&data).await?;

        // Write metadata
        self.write_metadata(key, &metadata).await?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<(Bytes, FileMetadata)> {
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Err(ApiError::FileNotFound(key.to_string()));
        }

        let data = fs::read(file_path).await?;
        let metadata = self.read_metadata(key).await?;

        Ok((Bytes::from(data), metadata))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let file_path = self.get_file_path(key);
        let metadata_path = self.get_metadata_path(key);

        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }

        if metadata_path.exists() {
            fs::remove_file(metadata_path).await?;
        }

        Ok(())
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<FileMetadata>> {
        let search_path = if let Some(prefix) = prefix {
            self.base_path.join(prefix)
        } else {
            self.base_path.clone()
        };

        let mut metadata_list = Vec::new();

        let mut entries = fs::read_dir(&search_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                // Return empty list if directory doesn't exist
                return ApiError::Storage("Directory not found".to_string());
            }
            ApiError::Io(e)
        })?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip metadata files
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".metadata.json"))
                .unwrap_or(false)
            {
                continue;
            }

            if path.is_file() {
                // Try to get the relative key
                let key = path
                    .strip_prefix(&self.base_path)
                    .ok()
                    .and_then(|p| p.to_str())
                    .ok_or_else(|| ApiError::Storage("Invalid file path".to_string()))?;

                if let Ok(metadata) = self.read_metadata(key).await {
                    metadata_list.push(metadata);
                }
            }
        }

        Ok(metadata_list)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let file_path = self.get_file_path(key);
        Ok(file_path.exists())
    }

    async fn get_metadata(&self, key: &str) -> Result<FileMetadata> {
        self.read_metadata(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).await.unwrap();

        let key = "test.txt";
        let data = Bytes::from("Hello, World!");
        let metadata = FileMetadata::new(key.to_string(), data.len() as u64);

        // Test put
        storage
            .put(key, data.clone(), metadata.clone())
            .await
            .unwrap();

        // Test exists
        assert!(storage.exists(key).await.unwrap());

        // Test get
        let (retrieved_data, _) = storage.get(key).await.unwrap();
        assert_eq!(retrieved_data, data);

        // Test delete
        storage.delete(key).await.unwrap();
        assert!(!storage.exists(key).await.unwrap());
    }
}
