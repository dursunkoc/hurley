//! Dataset parsing for performance tests.
//!
//! Supports loading request definitions from JSON files in various formats:
//! - JSON array: `[{"method": "GET"}, {"method": "POST", "body": {...}}]`
//! - Single object: `{"method": "GET", "path": "/api"}`
//! - Newline-delimited JSON (NDJSON)

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Result, RurlError};

/// A single entry in a performance test dataset.
///
/// Each entry defines an HTTP request with optional method, path, body, and headers.
/// Fields default to sensible values if not specified.
#[derive(Debug, Clone, Deserialize)]
pub struct DatasetEntry {
    /// HTTP method (defaults to "GET")
    #[serde(default = "default_method")]
    pub method: String,

    /// Request path (appended to base URL)
    #[serde(default)]
    pub path: Option<String>,

    /// Request body as JSON value
    #[serde(default)]
    pub body: Option<serde_json::Value>,

    /// Additional headers for this request
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

fn default_method() -> String {
    "GET".to_string()
}

impl DatasetEntry {
    /// Returns the body as a JSON string, if present.
    pub fn get_body_string(&self) -> Option<String> {
        self.body.as_ref().map(|v| v.to_string())
    }
}

/// A collection of dataset entries for performance testing.
///
/// # Example
///
/// ```rust,ignore
/// let dataset = Dataset::from_file(&PathBuf::from("requests.json"))?;
/// println!("Loaded {} requests", dataset.len());
/// ```
#[derive(Debug)]
pub struct Dataset {
    /// List of request entries
    pub entries: Vec<DatasetEntry>,
}

impl Dataset {
    /// Loads a dataset from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parses a dataset from a JSON string.
    ///
    /// Supports:
    /// - JSON array: `[{...}, {...}]`
    /// - Single object: `{...}`
    /// - Newline-delimited JSON
    pub fn from_json(content: &str) -> Result<Self> {
        // Try parsing as array first
        if let Ok(entries) = serde_json::from_str::<Vec<DatasetEntry>>(content) {
            return Ok(Self { entries });
        }

        // Try parsing as single object
        if let Ok(entry) = serde_json::from_str::<DatasetEntry>(content) {
            return Ok(Self { entries: vec![entry] });
        }

        // Try parsing as newline-delimited JSON (NDJSON)
        let mut entries = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: DatasetEntry = serde_json::from_str(line)
                .map_err(|e| RurlError::DatasetError(format!("Failed to parse line: {}", e)))?;
            entries.push(entry);
        }

        if entries.is_empty() {
            return Err(RurlError::DatasetError("Empty dataset".to_string()));
        }

        Ok(Self { entries })
    }

    /// Creates a simple dataset with GET requests (no path override).
    ///
    /// Used when no dataset file is provided but multiple requests are needed.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of entries to create
    pub fn simple(count: usize) -> Self {
        let entries = (0..count)
            .map(|_| DatasetEntry {
                method: "GET".to_string(),
                path: None,
                body: None,
                headers: None,
            })
            .collect();
        Self { entries }
    }

    /// Returns the number of entries in the dataset.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the dataset is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_array() {
        let json = r#"[{"method": "GET"}, {"method": "POST"}]"#;
        let dataset = Dataset::from_json(json).unwrap();
        assert_eq!(dataset.len(), 2);
        assert_eq!(dataset.entries[0].method, "GET");
        assert_eq!(dataset.entries[1].method, "POST");
    }

    #[test]
    fn test_parse_single_object() {
        let json = r#"{"method": "POST", "path": "/api"}"#;
        let dataset = Dataset::from_json(json).unwrap();
        assert_eq!(dataset.len(), 1);
        assert_eq!(dataset.entries[0].method, "POST");
        assert_eq!(dataset.entries[0].path, Some("/api".to_string()));
    }

    #[test]
    fn test_parse_ndjson() {
        let ndjson = r#"{"method": "GET"}
{"method": "POST"}"#;
        let dataset = Dataset::from_json(ndjson).unwrap();
        assert_eq!(dataset.len(), 2);
    }

    #[test]
    fn test_default_method() {
        let json = r#"[{}]"#;
        let dataset = Dataset::from_json(json).unwrap();
        assert_eq!(dataset.entries[0].method, "GET");
    }

    #[test]
    fn test_simple_dataset() {
        let dataset = Dataset::simple(5);
        assert_eq!(dataset.len(), 5);
        assert!(!dataset.is_empty());
    }

    #[test]
    fn test_body_with_json() {
        let json = r#"[{"method": "POST", "body": {"key": "value"}}]"#;
        let dataset = Dataset::from_json(json).unwrap();
        let body = dataset.entries[0].get_body_string().unwrap();
        assert!(body.contains("key"));
    }

    #[test]
    fn test_empty_dataset_error() {
        let result = Dataset::from_json("");
        assert!(result.is_err());
    }
}
