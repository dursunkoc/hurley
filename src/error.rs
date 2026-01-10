//! Error types for rurl.
//!
//! This module defines custom error types using `thiserror` for clean
//! error handling throughout the application.

use thiserror::Error;

/// Main error type for rurl operations.
///
/// All errors in the application are converted to this type for
/// consistent error handling and display.
#[derive(Error, Debug)]
pub enum RurlError {
    /// HTTP request failed (network error, timeout, etc.)
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Unsupported or invalid HTTP method
    #[error("Invalid method: {0}")]
    InvalidMethod(String),

    /// Invalid header format (should be "Name: Value")
    #[error("Invalid header format: {0}")]
    InvalidHeader(String),

    /// File I/O error
    #[error("File read error: {0}")]
    FileError(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Dataset loading or parsing error
    #[error("Dataset error: {0}")]
    DatasetError(String),

    /// Performance test execution error
    #[error("Performance test error: {0}")]
    PerfError(String),
}

/// Result type alias using [`RurlError`].
pub type Result<T> = std::result::Result<T, RurlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_method_error() {
        let error = RurlError::InvalidMethod("INVALID".to_string());
        assert!(error.to_string().contains("Invalid method"));
    }

    #[test]
    fn test_invalid_header_error() {
        let error = RurlError::InvalidHeader("bad-header".to_string());
        assert!(error.to_string().contains("Invalid header format"));
    }

    #[test]
    fn test_dataset_error() {
        let error = RurlError::DatasetError("empty file".to_string());
        assert!(error.to_string().contains("Dataset error"));
    }
}
