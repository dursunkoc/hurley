//! Performance test runner.
//!
//! Executes concurrent HTTP requests using tokio and collects timing metrics.

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use indicatif::{ProgressBar, ProgressStyle};

use crate::http::{HttpClient, HttpRequest};
use crate::error::Result;
use super::datafile::DataFile;
use super::dataset::{Dataset, DatasetEntry};
use super::metrics::{MetricsCollector, PerfMetrics};
use super::substitute::{get_row_for_request, substitute};

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
///     None, // optional DataFile for template substitution
/// );
/// let metrics = runner.run(&dataset).await?;
/// ```
pub struct PerfRunner {
    base_url: String,
    base_request: HttpRequest,
    concurrency: usize,
    total_requests: usize,
    verbose: bool,
    /// Optional data file for `{{placeholder}}` substitution.
    data_file: Option<DataFile>,
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
    /// * `data_file` - Optional data file for template variable substitution
    pub fn new(
        base_url: String,
        base_request: HttpRequest,
        concurrency: usize,
        total_requests: usize,
        verbose: bool,
        data_file: Option<DataFile>,
    ) -> Self {
        Self {
            base_url,
            base_request,
            concurrency,
            total_requests,
            verbose,
            data_file,
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

        // Build (request_index, DatasetEntry) pairs so build_request can apply
        // the correct data row for each global request index.
        let requests_to_make: Vec<(usize, DatasetEntry)> = if dataset.len() >= self.total_requests {
            dataset.entries
                .iter()
                .take(self.total_requests)
                .cloned()
                .enumerate()
                .collect()
        } else {
            // Cycle through dataset entries
            dataset.entries
                .iter()
                .cycle()
                .take(self.total_requests)
                .cloned()
                .enumerate()
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

        for (request_index, entry) in requests_to_make {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let collector = Arc::clone(&collector);
            let pb = pb.clone();
            let request = self.build_request(&entry, request_index)?;
            let verbose = self.verbose;
            
            // Create label for metrics (e.g., "GET /api/v1/users")
            let path_label = entry.path.as_deref().unwrap_or("/");
            let label = format!("{} {}", entry.method, path_label);

            let handle = tokio::spawn(async move {
                let client = HttpClient::new(verbose);
                let start = Instant::now();
                let result = client.execute(&request).await;
                let duration = start.elapsed();

                {
                    let mut c = collector.lock().await;
                    match result {
                        Ok(response) if response.is_success() => {
                            c.record_success(duration, Some(&label));
                        }
                        Ok(_) => {
                            c.record_failure(duration, Some(&label));
                        }
                        Err(_) => {
                            c.record_failure(duration, Some(&label));
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

    /// Builds an [`HttpRequest`] for `entry` at global `request_index`.
    ///
    /// When a [`DataFile`] is present, every `{{placeholder}}` in the URL,
    /// header values, and body is replaced with the corresponding value from
    /// the cycling data row for that index.  When absent, existing behaviour
    /// is unchanged.
    fn build_request(&self, entry: &DatasetEntry, request_index: usize) -> Result<HttpRequest> {
        // Resolve raw URL (base + path or absolute path)
        let raw_url = if let Some(path) = &entry.path {
            if path.starts_with("http://") || path.starts_with("https://") {
                path.clone()
            } else {
                format!("{}{}", self.base_url.trim_end_matches('/'), path)
            }
        } else {
            self.base_url.clone()
        };

        // Resolve the data row for this request index (None when no data file)
        let row_opt = self
            .data_file
            .as_ref()
            .map(|df| get_row_for_request(df, request_index));

        // Apply substitution to URL (no-op when row_opt is None)
        let final_url = match row_opt {
            Some(row) => substitute(&raw_url, row)?,
            None => raw_url,
        };

        let mut request = HttpRequest::new(final_url)
            .method(&entry.method)?
            .timeout(self.base_request.timeout)
            .follow_redirects(self.base_request.follow_redirects);

        // Merge headers from base request (with optional substitution)
        for (key, value) in &self.base_request.headers {
            let v = match row_opt {
                Some(row) => substitute(value, row)?,
                None => value.clone(),
            };
            request = request.header(key, v);
        }

        // Override with entry-specific headers (with optional substitution)
        if let Some(headers) = &entry.headers {
            for (key, value) in headers {
                let v = match row_opt {
                    Some(row) => substitute(value, row)?,
                    None => value.clone(),
                };
                request = request.header(key, v);
            }
        }

        // Set body (entry body takes priority over base request body)
        if let Some(body) = entry.get_body_string() {
            let b = match row_opt {
                Some(row) => substitute(&body, row)?,
                None => body,
            };
            request = request.body(b);
        } else if let Some(ref body) = self.base_request.body {
            let b = match row_opt {
                Some(row) => substitute(body, row)?,
                None => body.clone(),
            };
            request = request.body(b);
        }

        Ok(request)
    }
}
