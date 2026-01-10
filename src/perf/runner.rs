//! Performance test runner.
//!
//! Executes concurrent HTTP requests using tokio and collects timing metrics.

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use indicatif::{ProgressBar, ProgressStyle};

use crate::http::{HttpClient, HttpRequest};
use crate::error::Result;
use super::dataset::{Dataset, DatasetEntry};
use super::metrics::{MetricsCollector, PerfMetrics};

/// Performance test runner.
///
/// Executes HTTP requests concurrently using tokio with configurable
/// concurrency limits and progress tracking.
///
/// # Example
///
/// ```rust,ignore
/// let runner = PerfRunner::new(
///     "https://api.example.com".to_string(),
///     base_request,
///     10,  // concurrency
///     100, // total requests
///     false,
/// );
/// let metrics = runner.run(&dataset).await?;
/// ```
pub struct PerfRunner {
    base_url: String,
    base_request: HttpRequest,
    concurrency: usize,
    total_requests: usize,
    verbose: bool,
}

impl PerfRunner {
    /// Creates a new performance test runner.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL for requests
    /// * `base_request` - Template request with shared settings
    /// * `concurrency` - Maximum number of concurrent connections
    /// * `total_requests` - Total number of requests to execute
    /// * `verbose` - Whether to print verbose output
    pub fn new(
        base_url: String,
        base_request: HttpRequest,
        concurrency: usize,
        total_requests: usize,
        verbose: bool,
    ) -> Self {
        Self {
            base_url,
            base_request,
            concurrency,
            total_requests,
            verbose,
        }
    }

    /// Runs the performance test and returns collected metrics.
    ///
    /// Executes requests concurrently according to the concurrency limit,
    /// cycling through dataset entries if needed to reach the total request count.
    pub async fn run(&self, dataset: &Dataset) -> Result<PerfMetrics> {
        let collector = Arc::new(Mutex::new(MetricsCollector::new()));
        
        // Create progress bar
        let pb = ProgressBar::new(self.total_requests as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec})")
                .expect("Invalid progress bar template")
                .progress_chars("#>-")
        );

        // Determine how many requests to make
        let requests_to_make: Vec<DatasetEntry> = if dataset.len() >= self.total_requests {
            dataset.entries.iter().take(self.total_requests).cloned().collect()
        } else {
            // Cycle through dataset entries
            dataset.entries
                .iter()
                .cycle()
                .take(self.total_requests)
                .cloned()
                .collect()
        };

        // Record start time
        {
            let mut c = collector.lock().await;
            c.start();
        }

        // Create semaphore for concurrency control
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.concurrency));

        let mut handles = Vec::new();

        for entry in requests_to_make {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let collector = Arc::clone(&collector);
            let pb = pb.clone();
            let request = self.build_request(&entry)?;
            let verbose = self.verbose;

            let handle = tokio::spawn(async move {
                let client = HttpClient::new(verbose);
                let start = Instant::now();
                let result = client.execute(&request).await;
                let duration = start.elapsed();

                {
                    let mut c = collector.lock().await;
                    match result {
                        Ok(response) if response.is_success() => {
                            c.record_success(duration);
                        }
                        Ok(_) => {
                            c.record_failure(duration);
                        }
                        Err(_) => {
                            c.record_failure(duration);
                        }
                    }
                }

                pb.inc(1);
                drop(permit);
            });

            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            let _ = handle.await;
        }

        // Record end time
        {
            let mut c = collector.lock().await;
            c.finish();
        }

        pb.finish_with_message("Done!");

        let metrics = collector.lock().await.compute_metrics();
        Ok(metrics)
    }

    fn build_request(&self, entry: &DatasetEntry) -> Result<HttpRequest> {
        let url = if let Some(path) = &entry.path {
            if path.starts_with("http://") || path.starts_with("https://") {
                path.clone()
            } else {
                format!("{}{}", self.base_url.trim_end_matches('/'), path)
            }
        } else {
            self.base_url.clone()
        };

        let mut request = HttpRequest::new(url)
            .method(&entry.method)?
            .timeout(self.base_request.timeout)
            .follow_redirects(self.base_request.follow_redirects);

        // Merge headers from base request
        for (key, value) in &self.base_request.headers {
            request = request.header(key, value);
        }

        // Override with entry-specific headers
        if let Some(headers) = &entry.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        // Set body
        if let Some(body) = entry.get_body_string() {
            request = request.body(body);
        } else if let Some(body) = &self.base_request.body {
            request = request.body(body.clone());
        }

        Ok(request)
    }
}
