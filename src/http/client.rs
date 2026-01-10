//! HTTP client implementation.
//!
//! Provides the [`HttpClient`] which executes HTTP requests using reqwest.

use reqwest::redirect::Policy;
use reqwest::Client;
use std::time::Instant;
use colored::Colorize;

use crate::error::Result;
use super::request::HttpRequest;
use super::response::HttpResponse;

/// HTTP client for executing requests.
///
/// The client handles request execution with configurable verbosity
/// for debugging request/response details.
pub struct HttpClient {
    verbose: bool,
}

impl HttpClient {
    /// Creates a new HTTP client.
    ///
    /// # Arguments
    ///
    /// * `verbose` - Whether to print verbose request/response details
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Executes an HTTP request and returns the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request to execute
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (network error, timeout, etc.).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = HttpClient::new(false);
    /// let request = HttpRequest::new("https://httpbin.org/get");
    /// let response = client.execute(&request).await?;
    /// ```
    pub async fn execute(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let redirect_policy = if request.follow_redirects {
            Policy::limited(10)
        } else {
            Policy::none()
        };

        let client = Client::builder()
            .timeout(request.timeout)
            .redirect(redirect_policy)
            .build()?;

        if self.verbose {
            self.print_request_info(request);
        }

        let start = Instant::now();

        let mut req_builder = client.request(request.method.clone(), &request.url);

        // Add headers
        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add body
        if let Some(body) = &request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder.send().await?;
        let duration = start.elapsed();

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        Ok(HttpResponse::new(status, headers, body, duration))
    }

    fn print_request_info(&self, request: &HttpRequest) {
        println!("{}", ">>> Request".blue().bold());
        println!("{} {}", request.method.as_str().green(), request.url.cyan());
        
        for (key, value) in &request.headers {
            println!("{}: {}", key.yellow(), value);
        }
        
        if let Some(body) = &request.body {
            println!();
            // Try to pretty print JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                    println!("{}", pretty);
                } else {
                    println!("{}", body);
                }
            } else {
                println!("{}", body);
            }
        }
        
        println!();
        println!("{}", "<<< Response".blue().bold());
    }
}
