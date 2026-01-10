//! HTTP request builder.
//!
//! Provides a builder pattern for constructing HTTP requests with
//! method, headers, body, timeout, and redirect settings.

use reqwest::Method;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{Result, RurlError};

/// HTTP request configuration.
///
/// Use the builder pattern to construct requests:
///
/// ```rust,ignore
/// let request = HttpRequest::new("https://api.example.com")
///     .method("POST")?
///     .header("Content-Type", "application/json")
///     .body(r#"{"key": "value"}"#)
///     .timeout(Duration::from_secs(30));
/// ```
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: Method,
    /// Target URL
    pub url: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (optional)
    pub body: Option<String>,
    /// Request timeout
    pub timeout: Duration,
    /// Whether to follow HTTP redirects
    pub follow_redirects: bool,
}

impl HttpRequest {
    /// Creates a new HTTP request with default settings.
    ///
    /// Defaults:
    /// - Method: GET
    /// - Timeout: 30 seconds
    /// - Follow redirects: true
    ///
    /// # Arguments
    ///
    /// * `url` - The target URL for the request
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            method: Method::GET,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: Duration::from_secs(30),
            follow_redirects: true,
        }
    }

    /// Sets the HTTP method.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method string (GET, POST, PUT, DELETE, PATCH, HEAD)
    ///
    /// # Errors
    ///
    /// Returns [`RurlError::InvalidMethod`] if the method is not valid.
    pub fn method(mut self, method: &str) -> Result<Self> {
        self.method = method.to_uppercase().parse().map_err(|_| {
            RurlError::InvalidMethod(method.to_string())
        })?;
        Ok(self)
    }

    /// Adds a single header to the request.
    ///
    /// # Arguments
    ///
    /// * `key` - Header name
    /// * `value` - Header value
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Parses and adds headers from string slice.
    ///
    /// Each string should be in "Name: Value" format.
    ///
    /// # Errors
    ///
    /// Returns [`RurlError::InvalidHeader`] if any header is malformed.
    pub fn headers_from_strings(mut self, headers: &[String]) -> Result<Self> {
        for header in headers {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(RurlError::InvalidHeader(header.clone()));
            }
            self.headers.insert(
                parts[0].trim().to_string(),
                parts[1].trim().to_string(),
            );
        }
        Ok(self)
    }

    /// Sets the request body.
    ///
    /// # Arguments
    ///
    /// * `body` - Request body as a string
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Reads the request body from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file containing the request body
    ///
    /// # Errors
    ///
    /// Returns [`RurlError::FileError`] if the file cannot be read.
    pub fn body_from_file(mut self, path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        self.body = Some(content);
        Ok(self)
    }

    /// Sets the request timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Request timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets whether to follow HTTP redirects.
    ///
    /// # Arguments
    ///
    /// * `follow` - true to follow redirects, false otherwise
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request() {
        let request = HttpRequest::new("https://example.com");
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.method, Method::GET);
        assert!(request.follow_redirects);
    }

    #[test]
    fn test_method_post() {
        let request = HttpRequest::new("https://example.com")
            .method("POST")
            .unwrap();
        assert_eq!(request.method, Method::POST);
    }

    #[test]
    fn test_method_case_insensitive() {
        let request = HttpRequest::new("https://example.com")
            .method("post")
            .unwrap();
        assert_eq!(request.method, Method::POST);
    }

    #[test]
    fn test_invalid_method() {
        // Empty string is truly invalid
        let result = HttpRequest::new("https://example.com")
            .method("");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_method_allowed() {
        // reqwest allows custom methods like "CUSTOM"
        let result = HttpRequest::new("https://example.com")
            .method("CUSTOM");
        assert!(result.is_ok());
    }

    #[test]
    fn test_headers() {
        let request = HttpRequest::new("https://example.com")
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer token");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_headers_from_strings() {
        let headers = vec![
            "Content-Type: application/json".to_string(),
            "X-Custom: value".to_string(),
        ];
        let request = HttpRequest::new("https://example.com")
            .headers_from_strings(&headers)
            .unwrap();
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_invalid_header_format() {
        let headers = vec!["invalid-header-no-colon".to_string()];
        let result = HttpRequest::new("https://example.com")
            .headers_from_strings(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_body() {
        let request = HttpRequest::new("https://example.com")
            .body(r#"{"key": "value"}"#);
        assert_eq!(request.body, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn test_timeout() {
        let request = HttpRequest::new("https://example.com")
            .timeout(Duration::from_secs(60));
        assert_eq!(request.timeout, Duration::from_secs(60));
    }
}
