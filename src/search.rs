// Search functionality for file metadata
#![allow(dead_code)]

use crate::error::Result;
use crate::metadata::FileMetadata;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub name_pattern: Option<String>,
    #[serde(default)]
    pub include_deleted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResults {
    pub results: Vec<FileMetadata>,
    pub total_count: usize,
}

pub struct SearchEngine {
    storage: Arc<dyn Storage>,
}

impl SearchEngine {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Search files based on query parameters
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResults> {
        // Get all files
        let all_files = self.storage.list(None).await?;

        // Filter based on search criteria
        let mut results: Vec<FileMetadata> = all_files
            .into_iter()
            .filter(|file| self.matches_query(file, &query))
            .collect();

        // Sort by relevance (most recent first for now)
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let total_count = results.len();

        Ok(SearchResults {
            results,
            total_count,
        })
    }

    fn matches_query(&self, file: &FileMetadata, query: &SearchQuery) -> bool {
        // Filter deleted files unless explicitly included
        if !query.include_deleted && file.is_deleted {
            return false;
        }

        // Check name pattern
        if let Some(pattern) = &query.name_pattern {
            if !file
                .file_name
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return false;
            }
        }

        // Check full-text query in filename and custom metadata
        if let Some(q) = &query.query {
            let q_lower = q.to_lowercase();
            let matches_name = file.file_name.to_lowercase().contains(&q_lower);
            let matches_custom = file.custom.to_string().to_lowercase().contains(&q_lower);

            if !matches_name && !matches_custom {
                return false;
            }
        }

        // Check tags (must have ALL specified tags)
        if let Some(query_tags) = &query.tags {
            for tag in query_tags {
                if !file.tags.contains(tag) {
                    return false;
                }
            }
        }

        // Check content type
        if let Some(ct) = &query.content_type {
            match &file.content_type {
                Some(file_ct) if file_ct == ct => {}
                _ => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_deserialization() {
        let json = r#"{"query": "test", "tags": ["important"]}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.query, Some("test".to_string()));
        assert_eq!(query.tags, Some(vec!["important".to_string()]));
    }
}
