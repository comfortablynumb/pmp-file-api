// File processing module - ready for integration into upload handlers
#![allow(dead_code)]

use bytes::Bytes;
use image::imageops::FilterType;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::error::{ApiError, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileProcessingConfig {
    #[serde(default)]
    pub max_file_size: Option<u64>,
    #[serde(default)]
    pub allowed_mime_types: Option<Vec<String>>,
    #[serde(default)]
    pub blocked_mime_types: Option<Vec<String>>,
    #[serde(default)]
    pub enable_compression: bool,
    #[serde(default)]
    pub compression_type: CompressionType,
    #[serde(default)]
    pub enable_thumbnail: bool,
    #[serde(default)]
    pub thumbnail_width: Option<u32>,
    #[serde(default)]
    pub thumbnail_height: Option<u32>,
    #[serde(default)]
    pub enable_virus_scan: bool,
    #[serde(default)]
    pub clamav_host: Option<String>,
    #[serde(default)]
    pub clamav_port: Option<u16>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    #[default]
    None,
    Gzip,
    Brotli,
}

impl Default for FileProcessingConfig {
    fn default() -> Self {
        Self {
            max_file_size: None,
            allowed_mime_types: None,
            blocked_mime_types: None,
            enable_compression: false,
            compression_type: CompressionType::None,
            enable_thumbnail: false,
            thumbnail_width: Some(200),
            thumbnail_height: Some(200),
            enable_virus_scan: false,
            clamav_host: Some("localhost".to_string()),
            clamav_port: Some(3310),
        }
    }
}

pub struct FileProcessor {
    config: FileProcessingConfig,
}

impl FileProcessor {
    pub fn new(config: FileProcessingConfig) -> Self {
        Self { config }
    }

    /// Validate file size
    pub fn validate_size(&self, size: u64) -> Result<()> {
        if let Some(max_size) = self.config.max_file_size {
            if size > max_size {
                return Err(ApiError::Storage(format!(
                    "File size {} bytes exceeds maximum allowed size {} bytes",
                    size, max_size
                )));
            }
        }
        Ok(())
    }

    /// Validate MIME type
    pub fn validate_mime_type(&self, mime_type: &str) -> Result<()> {
        // Check blocklist first
        if let Some(blocked) = &self.config.blocked_mime_types {
            if blocked.iter().any(|b| mime_type.starts_with(b)) {
                return Err(ApiError::Storage(format!(
                    "MIME type '{}' is blocked",
                    mime_type
                )));
            }
        }

        // Check allowlist if configured
        if let Some(allowed) = &self.config.allowed_mime_types {
            if !allowed.iter().any(|a| mime_type.starts_with(a)) {
                return Err(ApiError::Storage(format!(
                    "MIME type '{}' is not in the allowed list",
                    mime_type
                )));
            }
        }

        Ok(())
    }

    /// Detect MIME type from file data
    pub fn detect_mime_type(&self, data: &[u8], filename: &str) -> String {
        // Try to guess from filename first
        if let Some(mime) = mime_guess::from_path(filename).first() {
            return mime.to_string();
        }

        // Fallback to binary detection (basic magic number detection)
        if data.len() >= 4 {
            match &data[0..4] {
                [0xFF, 0xD8, 0xFF, _] => return "image/jpeg".to_string(),
                [0x89, 0x50, 0x4E, 0x47] => return "image/png".to_string(),
                [0x47, 0x49, 0x46, 0x38] => return "image/gif".to_string(),
                [0x25, 0x50, 0x44, 0x46] => return "application/pdf".to_string(),
                [0x50, 0x4B, 0x03, 0x04] | [0x50, 0x4B, 0x05, 0x06] => {
                    return "application/zip".to_string()
                }
                _ => {}
            }
        }

        "application/octet-stream".to_string()
    }

    /// Compress file data
    pub async fn compress(&self, data: Bytes) -> Result<Bytes> {
        if !self.config.enable_compression {
            return Ok(data);
        }

        use async_compression::tokio::bufread::{BrotliEncoder, GzipEncoder};
        use tokio::io::AsyncReadExt;

        match self.config.compression_type {
            CompressionType::None => Ok(data),
            CompressionType::Gzip => {
                let cursor = Cursor::new(data);
                let mut encoder = GzipEncoder::new(tokio::io::BufReader::new(cursor));
                let mut compressed = Vec::new();
                encoder
                    .read_to_end(&mut compressed)
                    .await
                    .map_err(|e| ApiError::Storage(format!("Compression failed: {}", e)))?;
                Ok(Bytes::from(compressed))
            }
            CompressionType::Brotli => {
                let cursor = Cursor::new(data);
                let mut encoder = BrotliEncoder::new(tokio::io::BufReader::new(cursor));
                let mut compressed = Vec::new();
                encoder
                    .read_to_end(&mut compressed)
                    .await
                    .map_err(|e| ApiError::Storage(format!("Compression failed: {}", e)))?;
                Ok(Bytes::from(compressed))
            }
        }
    }

    /// Generate thumbnail for images
    pub fn generate_thumbnail(&self, data: &[u8], mime_type: &str) -> Result<Option<Bytes>> {
        if !self.config.enable_thumbnail {
            return Ok(None);
        }

        // Only process images
        if !mime_type.starts_with("image/") {
            return Ok(None);
        }

        let width = self.config.thumbnail_width.unwrap_or(200);
        let height = self.config.thumbnail_height.unwrap_or(200);

        let img = image::load_from_memory(data)
            .map_err(|e| ApiError::Storage(format!("Failed to load image: {}", e)))?;

        let thumbnail = img.resize(width, height, FilterType::Lanczos3);

        let mut buffer = Cursor::new(Vec::new());
        thumbnail
            .write_to(&mut buffer, ImageFormat::Jpeg)
            .map_err(|e| ApiError::Storage(format!("Failed to write thumbnail: {}", e)))?;

        Ok(Some(Bytes::from(buffer.into_inner())))
    }

    /// Scan file for viruses using ClamAV
    #[cfg(feature = "virus-scan")]
    pub async fn scan_virus(&self, data: &[u8]) -> Result<()> {
        if !self.config.enable_virus_scan {
            return Ok(());
        }

        let host = self
            .config
            .clamav_host
            .as_deref()
            .unwrap_or("localhost");
        let port = self.config.clamav_port.unwrap_or(3310);
        let host_address = format!("{}:{}", host, port);

        // Convert data to owned Vec for spawn_blocking
        let data_vec = data.to_vec();

        // Use clamav-client 0.4 functional API
        // scan_buffer_tcp returns a Vec<u8> response from ClamAV
        let response_bytes = tokio::task::spawn_blocking(move || {
            clamav_client::scan_buffer_tcp(&data_vec, &host_address, None)
        })
        .await
        .map_err(|e| ApiError::Storage(format!("Failed to spawn virus scan task: {}", e)))?
        .map_err(|e| ApiError::Storage(format!("Virus scan failed: {}", e)))?;

        // Convert response to string for checking
        let response = String::from_utf8_lossy(&response_bytes);

        // Check if virus was found (response contains "FOUND")
        if response.contains("FOUND") {
            return Err(ApiError::Storage(format!(
                "Virus detected: {}",
                response.trim_end_matches('\0')
            )));
        }

        Ok(())
    }

    /// Placeholder when virus-scan feature is not enabled
    #[cfg(not(feature = "virus-scan"))]
    pub async fn scan_virus(&self, _data: &[u8]) -> Result<()> {
        if self.config.enable_virus_scan {
            tracing::warn!("Virus scanning is enabled but 'virus-scan' feature is not compiled");
        }
        Ok(())
    }

    /// Process file: validate, compress, scan
    pub async fn process_file(
        &self,
        data: Bytes,
        filename: &str,
        provided_mime_type: Option<&str>,
    ) -> Result<ProcessedFile> {
        // Validate size
        self.validate_size(data.len() as u64)?;

        // Detect or use provided MIME type
        let mime_type = if let Some(mt) = provided_mime_type {
            mt.to_string()
        } else {
            self.detect_mime_type(&data, filename)
        };

        // Validate MIME type
        self.validate_mime_type(&mime_type)?;

        // Scan for viruses
        self.scan_virus(&data).await?;

        // Generate thumbnail if applicable
        let thumbnail = self.generate_thumbnail(&data, &mime_type)?;

        // Compress if enabled
        let processed_data = self.compress(data).await?;

        Ok(ProcessedFile {
            data: processed_data,
            mime_type,
            thumbnail,
        })
    }
}

pub struct ProcessedFile {
    pub data: Bytes,
    pub mime_type: String,
    pub thumbnail: Option<Bytes>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_validation() {
        let config = FileProcessingConfig {
            max_file_size: Some(1024),
            ..Default::default()
        };
        let processor = FileProcessor::new(config);

        assert!(processor.validate_size(512).is_ok());
        assert!(processor.validate_size(1024).is_ok());
        assert!(processor.validate_size(2048).is_err());
    }

    #[test]
    fn test_mime_type_validation() {
        let config = FileProcessingConfig {
            allowed_mime_types: Some(vec!["image/".to_string(), "text/".to_string()]),
            ..Default::default()
        };
        let processor = FileProcessor::new(config);

        assert!(processor.validate_mime_type("image/jpeg").is_ok());
        assert!(processor.validate_mime_type("text/plain").is_ok());
        assert!(processor.validate_mime_type("application/pdf").is_err());
    }

    #[test]
    fn test_mime_type_blocklist() {
        let config = FileProcessingConfig {
            blocked_mime_types: Some(vec!["application/x-executable".to_string()]),
            ..Default::default()
        };
        let processor = FileProcessor::new(config);

        assert!(processor.validate_mime_type("image/jpeg").is_ok());
        assert!(processor
            .validate_mime_type("application/x-executable")
            .is_err());
    }
}
