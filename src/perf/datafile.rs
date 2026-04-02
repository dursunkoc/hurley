//! Data file parsing for performance tests.
//!
//! Supports loading variable substitution datasets from:
//! - CSV files (extension `.csv`) — headers become column names
//! - JSON array files (extension `.json`) — each object becomes a row
//! - Newline-delimited JSON (NDJSON) — also accepted as `.json`
//!
//! All values are stored as `String` regardless of the source type.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Result, RurlError};

/// A single data row: a map from column name to string value.
pub type DataRow = HashMap<String, String>;

/// A parsed data file containing rows and their column names.
///
/// Produced by [`DataFile::from_path`] and consumed by downstream
/// variable-substitution logic.
///
/// # Example
///
/// ```rust,ignore
/// let df = DataFile::from_path(Path::new("users.csv"))?;
/// for row in df.rows() {
///     println!("{:?}", row.get("name"));
/// }
/// ```
#[derive(Debug)]
pub struct DataFile {
    rows: Vec<DataRow>,
    columns: Vec<String>,
}

impl DataFile {
    /// Loads and parses a data file from `path`.
    ///
    /// File format is detected from the extension (case-insensitive):
    /// - `.csv` → CSV with header row
    /// - `.json` → JSON array of objects or NDJSON
    ///
    /// # Errors
    ///
    /// Returns [`RurlError::DataFileError`] for unsupported extensions,
    /// malformed content, or empty files.
    pub fn from_path(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let content = std::fs::read_to_string(path)?;

        match ext.as_str() {
            "csv" => Self::parse_csv(&content),
            "json" => Self::parse_json(&content),
            other => Err(RurlError::DataFileError(format!(
                "Unsupported file extension: {}",
                other
            ))),
        }
    }

    /// Returns a slice of all data rows.
    pub fn rows(&self) -> &[DataRow] {
        &self.rows
    }

    /// Returns the ordered list of column names.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Returns the number of data rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns `true` if there are no data rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Parses CSV content into a [`DataFile`].
    ///
    /// The first row is treated as headers; subsequent rows become
    /// [`DataRow`] maps keyed by those headers.
    fn parse_csv(content: &str) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        // Clone headers before the borrow is tied to record iteration.
        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| RurlError::DataFileError(format!("CSV header error: {}", e)))?
            .iter()
            .map(|h| h.to_string())
            .collect();

        if headers.is_empty() {
            return Err(RurlError::DataFileError("CSV has no headers".to_string()));
        }

        let mut rows = Vec::new();
        for result in reader.records() {
            let record =
                result.map_err(|e| RurlError::DataFileError(format!("CSV parse error: {}", e)))?;
            let row: DataRow = headers
                .iter()
                .zip(record.iter())
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect();
            rows.push(row);
        }

        if rows.is_empty() {
            return Err(RurlError::DataFileError(
                "CSV file has headers but no data rows".to_string(),
            ));
        }

        Ok(Self {
            rows,
            columns: headers,
        })
    }

    /// Parses JSON content into a [`DataFile`].
    ///
    /// Tries JSON array first, then falls back to NDJSON (one object per line).
    /// All [`serde_json::Value`] variants are coerced to `String`.
    fn parse_json(content: &str) -> Result<Self> {
        // Try JSON array first.
        if let Ok(array) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(content)
        {
            return Self::from_value_rows(array);
        }

        // Fall back to NDJSON.
        let mut value_rows: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let obj: HashMap<String, serde_json::Value> = serde_json::from_str(line)
                .map_err(|e| RurlError::DataFileError(format!("JSON parse error: {}", e)))?;
            value_rows.push(obj);
        }

        Self::from_value_rows(value_rows)
    }

    /// Converts a `Vec<HashMap<String, serde_json::Value>>` into a [`DataFile`],
    /// coercing all values to strings.
    fn from_value_rows(value_rows: Vec<HashMap<String, serde_json::Value>>) -> Result<Self> {
        if value_rows.is_empty() {
            return Err(RurlError::DataFileError("JSON data is empty".to_string()));
        }

        // Column names from first row's keys (insertion order not guaranteed in HashMap,
        // but we collect them for the columns() accessor).
        let columns: Vec<String> = value_rows[0].keys().cloned().collect();

        let rows: Vec<DataRow> = value_rows
            .into_iter()
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        let s = match &v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k, s)
                    })
                    .collect()
            })
            .collect();

        Ok(Self { rows, columns })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CSV tests ──────────────────────────────────────────────────────────

    #[test]
    fn test_csv_basic() {
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let df = DataFile::parse_csv(csv).unwrap();
        assert_eq!(df.len(), 2);
        assert_eq!(df.rows()[0].get("name").map(String::as_str), Some("Alice"));
        assert_eq!(df.rows()[0].get("age").map(String::as_str), Some("30"));
        assert_eq!(df.rows()[1].get("city").map(String::as_str), Some("LA"));
    }

    #[test]
    fn test_csv_columns_extraction() {
        let csv = "id,username,email\n1,alice,alice@example.com";
        let df = DataFile::parse_csv(csv).unwrap();
        let cols = df.columns();
        assert!(cols.contains(&"id".to_string()));
        assert!(cols.contains(&"username".to_string()));
        assert!(cols.contains(&"email".to_string()));
        assert_eq!(cols.len(), 3);
    }

    #[test]
    fn test_csv_empty_content_error() {
        let result = DataFile::parse_csv("");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Data file error"), "unexpected: {}", msg);
    }

    #[test]
    fn test_csv_headers_no_data_rows_error() {
        let csv = "name,age,city\n";
        let result = DataFile::parse_csv(csv);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no data rows"), "unexpected: {}", msg);
    }

    // ── JSON tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_json_array_basic() {
        let json = r#"[{"name":"Alice","score":"100"},{"name":"Bob","score":"90"}]"#;
        let df = DataFile::parse_json(json).unwrap();
        assert_eq!(df.len(), 2);
        let names: Vec<&str> = df
            .rows()
            .iter()
            .filter_map(|r| r.get("name").map(|s| s.as_str()))
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Bob"));
    }

    #[test]
    fn test_json_number_coercion() {
        let json = r#"[{"id": 42, "value": 3.14}]"#;
        let df = DataFile::parse_json(json).unwrap();
        assert_eq!(df.rows()[0].get("id").map(String::as_str), Some("42"));
        assert_eq!(df.rows()[0].get("value").map(String::as_str), Some("3.14"));
    }

    #[test]
    fn test_json_boolean_coercion() {
        let json = r#"[{"active": true, "deleted": false}]"#;
        let df = DataFile::parse_json(json).unwrap();
        assert_eq!(df.rows()[0].get("active").map(String::as_str), Some("true"));
        assert_eq!(
            df.rows()[0].get("deleted").map(String::as_str),
            Some("false")
        );
    }

    #[test]
    fn test_json_empty_array_error() {
        let result = DataFile::parse_json("[]");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_columns_extraction() {
        let json = r#"[{"x": "1", "y": "2", "z": "3"}]"#;
        let df = DataFile::parse_json(json).unwrap();
        let cols = df.columns();
        assert_eq!(cols.len(), 3);
        assert!(cols.contains(&"x".to_string()));
        assert!(cols.contains(&"y".to_string()));
        assert!(cols.contains(&"z".to_string()));
    }

    // ── CSV edge-case tests ────────────────────────────────────────────────

    #[test]
    fn test_csv_quoted_fields() {
        // Fields containing commas and doubled double-quotes (RFC 4180 escaping).
        let csv = "name,desc\n\"Alice\",\"says \"\"hello\"\"\"\n\"Bob\",\"a, b, c\"";
        let df = DataFile::parse_csv(csv).unwrap();
        assert_eq!(df.len(), 2);
        assert_eq!(df.rows()[0].get("name").map(String::as_str), Some("Alice"));
        assert_eq!(
            df.rows()[0].get("desc").map(String::as_str),
            Some("says \"hello\"")
        );
        assert_eq!(
            df.rows()[1].get("desc").map(String::as_str),
            Some("a, b, c")
        );
    }

    // ── JSON edge-case tests ───────────────────────────────────────────────

    #[test]
    fn test_json_ndjson() {
        let ndjson = "{\"id\":\"1\"}\n{\"id\":\"2\"}";
        let df = DataFile::parse_json(ndjson).unwrap();
        assert_eq!(df.len(), 2);
        let ids: Vec<&str> = df
            .rows()
            .iter()
            .filter_map(|r| r.get("id").map(String::as_str))
            .collect();
        assert!(ids.contains(&"1"));
        assert!(ids.contains(&"2"));
    }

    #[test]
    fn test_json_mixed_coercion() {
        // Numbers and booleans in the same object coerce to strings.
        let json = r#"[{"id": 1, "active": true, "score": 9.5}]"#;
        let df = DataFile::parse_json(json).unwrap();
        assert_eq!(df.rows()[0].get("id").map(String::as_str), Some("1"));
        assert_eq!(df.rows()[0].get("active").map(String::as_str), Some("true"));
        assert_eq!(df.rows()[0].get("score").map(String::as_str), Some("9.5"));
    }

    // ── from_path routing tests ────────────────────────────────────────────

    #[test]
    fn test_from_path_csv() {
        use std::io::Write;
        let path = std::env::temp_dir().join("hurley_test_route.csv");
        let mut f = std::fs::File::create(&path).expect("create temp csv");
        write!(f, "x,y\n1,2\n3,4").unwrap();
        let df = DataFile::from_path(&path).expect("parse csv via from_path");
        let _ = std::fs::remove_file(&path);
        assert_eq!(df.len(), 2);
        assert_eq!(df.rows()[0].get("x").map(String::as_str), Some("1"));
    }

    #[test]
    fn test_from_path_json() {
        use std::io::Write;
        let path = std::env::temp_dir().join("hurley_test_route.json");
        let mut f = std::fs::File::create(&path).expect("create temp json");
        write!(f, r#"[{{"k":"v1"}},{{"k":"v2"}}]"#).unwrap();
        let df = DataFile::from_path(&path).expect("parse json via from_path");
        let _ = std::fs::remove_file(&path);
        assert_eq!(df.len(), 2);
        let vals: Vec<&str> = df
            .rows()
            .iter()
            .filter_map(|r| r.get("k").map(String::as_str))
            .collect();
        assert!(vals.contains(&"v1"));
        assert!(vals.contains(&"v2"));
    }

    #[test]
    fn test_extension_case_insensitive() {
        use std::io::Write;
        let path = std::env::temp_dir().join("hurley_test_case.CSV");
        let mut f = std::fs::File::create(&path).expect("create temp .CSV file");
        write!(f, "col\nval").unwrap();
        let df = DataFile::from_path(&path).expect("parse .CSV (uppercase) via from_path");
        let _ = std::fs::remove_file(&path);
        assert_eq!(df.len(), 1);
        assert_eq!(df.rows()[0].get("col").map(String::as_str), Some("val"));
    }

    // ── Negative / boundary tests ──────────────────────────────────────────

    #[test]
    fn test_unsupported_extension_error() {
        // Create a real temp file with a .txt extension so from_path reaches
        // the extension-matching logic (it reads the file first).
        use std::io::Write;
        let path = std::env::temp_dir().join("hurley_test_unsupported.txt");
        let mut f = std::fs::File::create(&path).expect("create temp file");
        write!(f, "hello").unwrap();
        let result = DataFile::from_path(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Unsupported file extension"),
            "unexpected: {}",
            msg
        );
    }
}
