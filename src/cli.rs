//! CLI argument definitions for hurley.
//!
//! This module uses the `clap` crate with derive macros to define
//! command-line arguments for both single HTTP requests and performance testing.

use clap::Parser;
use std::path::PathBuf;

/// A curl-like HTTP client with performance testing capabilities.
///
/// hurley supports standard HTTP operations like GET, POST, PUT, DELETE with
/// custom headers and body. It also includes a performance testing mode
/// for benchmarking API endpoints.
///
/// # Examples
///
/// ```bash
/// # Simple GET request
/// hurley https://httpbin.org/get
///
/// # POST with JSON body
/// hurley -X POST https://httpbin.org/post -d '{"key": "value"}'
///
/// # Performance test with 10 concurrent connections
/// hurley https://api.example.com -c 10 -n 100
/// ```
#[derive(Parser, Debug)]
#[command(name = "hurley")]
#[command(author = "Dursun Koc <dursunkoc@gmail.com>")]
#[command(version = "0.1.1")]
#[command(about = "A curl-like HTTP client with performance testing capabilities", long_about = None)]
pub struct Cli {
    /// Target URL for the HTTP request.
    pub url: String,

    /// HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD).
    ///
    /// Defaults to GET if not specified.
    #[arg(short = 'X', long, default_value = "GET")]
    pub method: String,

    /// Request headers (can be used multiple times).
    ///
    /// Format: "Header-Name: Header-Value"
    ///
    /// # Example
    /// ```bash
    /// hurley https://api.example.com -H "Content-Type: application/json" -H "Authorization: Bearer token"
    /// ```
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Request body (inline data).
    ///
    /// # Example
    /// ```bash
    /// hurley -X POST https://api.example.com -d '{"name": "test"}'
    /// ```
    #[arg(short = 'd', long = "data")]
    pub data: Option<String>,

    /// Read request body from file.
    ///
    /// # Example
    /// ```bash
    /// hurley -X POST https://api.example.com -f payload.json
    /// ```
    #[arg(short = 'f', long = "file")]
    pub body_file: Option<PathBuf>,

    /// Include response headers in output.
    #[arg(short = 'i', long = "include")]
    pub include_headers: bool,

    /// Follow HTTP redirects (up to 10 redirects).
    #[arg(short = 'L', long = "location")]
    pub follow_redirects: bool,

    /// Verbose output showing request details.
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Request timeout in seconds.
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Run performance test with dataset file (JSON format).
    ///
    /// The dataset should be a JSON array of request objects:
    /// ```json
    /// [
    ///   {"method": "GET", "path": "/users"},
    ///   {"method": "POST", "path": "/users", "body": {"name": "test"}}
    /// ]
    /// ```
    #[arg(long = "perf")]
    pub perf_file: Option<PathBuf>,

    /// Number of concurrent connections for performance test.
    #[arg(short = 'c', long = "concurrency", default_value = "1")]
    pub concurrency: usize,

    /// Total number of requests for performance test.
    #[arg(short = 'n', long = "requests", default_value = "1")]
    pub total_requests: usize,

    /// Output format for performance results (text, json).
    #[arg(long = "output", default_value = "text")]
    pub output_format: String,
}

impl Cli {
    /// Returns true if the CLI arguments indicate performance test mode.
    ///
    /// Performance mode is activated when:
    /// - A performance dataset file is specified (`--perf`)
    /// - Total requests is greater than 1 (`-n`)
    /// - Concurrency is greater than 1 (`-c`)
    pub fn is_perf_mode(&self) -> bool {
        self.perf_file.is_some() || self.total_requests > 1 || self.concurrency > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let cli = Cli::parse_from(["hurley", "https://example.com"]);
        assert_eq!(cli.url, "https://example.com");
        assert_eq!(cli.method, "GET");
        assert_eq!(cli.timeout, 30);
        assert_eq!(cli.concurrency, 1);
        assert_eq!(cli.total_requests, 1);
        assert!(!cli.is_perf_mode());
    }

    #[test]
    fn test_post_with_data() {
        let cli = Cli::parse_from([
            "hurley",
            "-X", "POST",
            "https://example.com",
            "-d", r#"{"key": "value"}"#,
        ]);
        assert_eq!(cli.method, "POST");
        assert_eq!(cli.data, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn test_headers() {
        let cli = Cli::parse_from([
            "hurley",
            "https://example.com",
            "-H", "Content-Type: application/json",
            "-H", "Authorization: Bearer token",
        ]);
        assert_eq!(cli.headers.len(), 2);
        assert_eq!(cli.headers[0], "Content-Type: application/json");
    }

    #[test]
    fn test_perf_mode_with_concurrency() {
        let cli = Cli::parse_from([
            "hurley",
            "https://example.com",
            "-c", "10",
            "-n", "100",
        ]);
        assert!(cli.is_perf_mode());
        assert_eq!(cli.concurrency, 10);
        assert_eq!(cli.total_requests, 100);
    }

    #[test]
    fn test_flags() {
        let cli = Cli::parse_from([
            "hurley",
            "https://example.com",
            "-i", "-L", "-v",
        ]);
        assert!(cli.include_headers);
        assert!(cli.follow_redirects);
        assert!(cli.verbose);
    }
}
