use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_name: String,
    pub content_type: Option<String>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub custom: JsonValue,

    // Versioning support
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub version_id: Option<Uuid>,
    #[serde(default)]
    pub parent_version_id: Option<Uuid>,

    // Tags and categories
    #[serde(default)]
    pub tags: Vec<String>,

    // Soft delete support
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,

    // File deduplication
    #[serde(default)]
    pub content_hash: Option<String>,
}

fn default_version() -> u32 {
    1
}

impl FileMetadata {
    pub fn new(file_name: String, size: u64) -> Self {
        let now = Utc::now();
        Self {
            file_name,
            content_type: None,
            size,
            created_at: now,
            updated_at: now,
            custom: JsonValue::Object(serde_json::Map::new()),
            version: 1,
            version_id: Some(Uuid::new_v4()),
            parent_version_id: None,
            tags: Vec::new(),
            is_deleted: false,
            deleted_at: None,
            content_hash: None,
        }
    }

    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn with_custom(mut self, custom: JsonValue) -> Self {
        self.custom = custom;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_hash(mut self, hash: String) -> Self {
        self.content_hash = Some(hash);
        self
    }

    pub fn create_new_version(&self) -> Self {
        let now = Utc::now();
        Self {
            file_name: self.file_name.clone(),
            content_type: self.content_type.clone(),
            size: self.size,
            created_at: now,
            updated_at: now,
            custom: self.custom.clone(),
            version: self.version + 1,
            version_id: Some(Uuid::new_v4()),
            parent_version_id: self.version_id,
            tags: self.tags.clone(),
            is_deleted: false,
            deleted_at: None,
            content_hash: None,
        }
    }

    pub fn soft_delete(&mut self) {
        self.is_deleted = true;
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn restore(&mut self) {
        self.is_deleted = false;
        self.deleted_at = None;
        self.updated_at = Utc::now();
    }

    pub fn matches_filter(&self, filters: &FilterParams) -> bool {
        // Exclude deleted files by default unless explicitly included
        if !filters.include_deleted && self.is_deleted {
            return false;
        }

        // Filter by file name pattern
        if let Some(pattern) = &filters.name_pattern {
            if !self.file_name.contains(pattern) {
                return false;
            }
        }

        // Filter by content type
        if let Some(ct) = &filters.content_type {
            match &self.content_type {
                Some(file_ct) if file_ct == ct => {}
                _ => return false,
            }
        }

        // Filter by tags (file must have ALL specified tags)
        if let Some(filter_tags) = &filters.tags {
            for tag in filter_tags {
                if !self.tags.contains(tag) {
                    return false;
                }
            }
        }

        // Filter by custom metadata
        if let Some(custom_filters) = &filters.custom {
            for (key, value) in custom_filters.iter() {
                if self.custom.get(key) != Some(value) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterParams {
    pub name_pattern: Option<String>,
    pub content_type: Option<String>,
    pub custom: Option<serde_json::Map<String, JsonValue>>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub include_deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_metadata_filter() {
        let mut metadata = FileMetadata::new("test.txt".to_string(), 100);
        metadata.content_type = Some("text/plain".to_string());
        metadata.custom = json!({"author": "test", "version": 1});

        let filters = FilterParams {
            name_pattern: Some("test".to_string()),
            content_type: Some("text/plain".to_string()),
            custom: None,
            tags: None,
            include_deleted: false,
        };

        assert!(metadata.matches_filter(&filters));

        let filters = FilterParams {
            name_pattern: Some("other".to_string()),
            content_type: None,
            custom: None,
            tags: None,
            include_deleted: false,
        };

        assert!(!metadata.matches_filter(&filters));
    }
}
