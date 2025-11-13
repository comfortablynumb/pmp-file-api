use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{ApiError, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storages: HashMap<String, StorageConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum StorageConfig {
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        region: String,
        #[serde(default)]
        prefix: String,
        #[serde(default)]
        endpoint: Option<String>,
    },
    #[serde(rename = "local")]
    Local { path: String },
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ApiError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| ApiError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    pub fn get_storage(&self, name: &str) -> Option<&StorageConfig> {
        self.storages.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080

storages:
  my-s3:
    type: s3
    bucket: my-bucket
    region: us-east-1
    prefix: files/

  local-storage:
    type: local
    path: /tmp/files
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.storages.len(), 2);
    }
}
