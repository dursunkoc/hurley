//! HTTP response handling.
//!
//! Provides response parsing and formatted output with colored
//! status codes and headers.

use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use std::time::Duration;
use colored::Colorize;

/// HTTP response with timing information.
///
/// Contains the response status, headers, body, and the time
/// taken to receive the response.
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: StatusCode,
    /// Response headers
    pub headers: HeaderMap,
    /// Response body as string
    pub body: String,
    /// Time taken to receive the response
    pub duration: Duration,
}

impl HttpResponse {
    /// Creates a new HTTP response.
    pub fn new(
        status: StatusCode,
        headers: HeaderMap,
        body: String,
        duration: Duration,
    ) -> Self {
        Self {
            status,
            headers,
            body,
            duration,
        }
    }

    /// Returns true if the response status is successful (2xx).
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Formats the status line with color based on status code.
    ///
    /// - 2xx: Green
    /// - 4xx: Yellow
    /// - 5xx: Red
    pub fn format_status(&self) -> String {
        let status_str = format!("HTTP/1.1 {} {}", self.status.as_u16(), self.status.canonical_reason().unwrap_or(""));
        
        if self.status.is_success() {
            status_str.green().to_string()
        } else if self.status.is_client_error() {
            status_str.yellow().to_string()
        } else if self.status.is_server_error() {
            status_str.red().to_string()
        } else {
            status_str
        }
    }

    /// Formats response headers with colored names.
    pub fn format_headers(&self) -> String {
        let mut result = String::new();
        for (key, value) in self.headers.iter() {
            result.push_str(&format!(
                "{}: {}\n",
                key.as_str().cyan(),
                value.to_str().unwrap_or("<binary>")
            ));
        }
        result
    }

    /// Formats the response duration in milliseconds.
    pub fn format_duration(&self) -> String {
        format!("Time: {:.3}ms", self.duration.as_secs_f64() * 1000.0)
    }

    /// Prints the response to stdout.
    ///
    /// # Arguments
    ///
    /// * `include_headers` - Whether to print response headers
    /// * `verbose` - Whether to print timing information
    pub fn print(&self, include_headers: bool, verbose: bool) {
        if verbose {
            println!("{}", self.format_duration().dimmed());
            println!();
        }

        if include_headers {
            println!("{}", self.format_status());
            print!("{}", self.format_headers());
            println!();
        }

        // Try to pretty print JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.body) {
            if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                println!("{}", pretty);
                return;
            }
        }

        println!("{}", self.body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_success() {
        let response = HttpResponse::new(
            StatusCode::OK,
            HeaderMap::new(),
            "OK".to_string(),
            Duration::from_millis(100),
        );
        assert!(response.is_success());
    }

    #[test]
    fn test_is_not_success() {
        let response = HttpResponse::new(
            StatusCode::NOT_FOUND,
            HeaderMap::new(),
            "Not Found".to_string(),
            Duration::from_millis(100),
        );
        assert!(!response.is_success());
    }

    #[test]
    fn test_format_duration() {
        let response = HttpResponse::new(
            StatusCode::OK,
            HeaderMap::new(),
            "OK".to_string(),
            Duration::from_millis(150),
        );
        assert!(response.format_duration().contains("150"));
    }
}
