use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_name: String,
    pub content_type: Option<String>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub custom: JsonValue,
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

    pub fn matches_filter(&self, filters: &FilterParams) -> bool {
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
        };

        assert!(metadata.matches_filter(&filters));

        let filters = FilterParams {
            name_pattern: Some("other".to_string()),
            content_type: None,
            custom: None,
        };

        assert!(!metadata.matches_filter(&filters));
    }
}
