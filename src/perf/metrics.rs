//! Performance metrics collection and calculation.
//!
//! Uses HdrHistogram for accurate latency percentile calculations
//! (p50, p95, p99) with minimal memory overhead.

use std::time::Duration;
use hdrhistogram::Histogram;
use serde::Serialize;

/// Performance test metrics.
///
/// Contains aggregate statistics about request execution including
/// latency distribution and throughput.
#[derive(Debug, Serialize)]
pub struct PerfMetrics {
    /// Total number of requests made
    pub total_requests: usize,
    /// Number of successful requests (2xx status)
    pub successful_requests: usize,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Total test duration in milliseconds
    pub total_duration_ms: f64,
    /// Minimum latency in milliseconds
    pub latency_min_ms: f64,
    /// Maximum latency in milliseconds
    pub latency_max_ms: f64,
    /// Average latency in milliseconds
    pub latency_avg_ms: f64,
    /// 50th percentile (median) latency
    pub latency_p50_ms: f64,
    /// 95th percentile latency
    pub latency_p95_ms: f64,
    /// 99th percentile latency
    pub latency_p99_ms: f64,
    /// Requests per second throughput
    pub requests_per_second: f64,
    /// Percentage of failed requests
    pub error_rate_percent: f64,
}

/// Collects timing data during performance tests.
///
/// Records individual request durations and computes aggregate metrics.
pub struct MetricsCollector {
    histogram: Histogram<u64>,
    successful: usize,
    failed: usize,
    start_time: Option<std::time::Instant>,
    end_time: Option<std::time::Instant>,
}

impl MetricsCollector {
    /// Creates a new metrics collector.
    ///
    /// The histogram is configured to track latencies up to 60 seconds
    /// with 3 significant figures of precision.
    pub fn new() -> Self {
        // Create histogram with max value of 60 seconds (in microseconds)
        // sigfig=3 gives us good precision for latency measurements
        let histogram = Histogram::new_with_bounds(1, 60_000_000, 3)
            .expect("Failed to create histogram");
        
        Self {
            histogram,
            successful: 0,
            failed: 0,
            start_time: None,
            end_time: None,
        }
    }

    /// Marks the start of the performance test.
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }

    /// Marks the end of the performance test.
    pub fn finish(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }

    /// Records a successful request with its duration.
    pub fn record_success(&mut self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        // Clamp to histogram max value
        let micros = micros.min(self.histogram.high());
        let _ = self.histogram.record(micros);
        self.successful += 1;
    }

    /// Records a failed request with its duration.
    pub fn record_failure(&mut self, duration: Duration) {
        let micros = duration.as_micros() as u64;
        let micros = micros.min(self.histogram.high());
        let _ = self.histogram.record(micros);
        self.failed += 1;
    }

    /// Computes final metrics from collected data.
    ///
    /// Returns a [`PerfMetrics`] struct with all aggregate statistics.
    pub fn compute_metrics(&self) -> PerfMetrics {
        let total = self.successful + self.failed;
        let total_duration = match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => end.duration_since(start),
            _ => Duration::ZERO,
        };

        let total_duration_ms = total_duration.as_secs_f64() * 1000.0;
        
        let requests_per_second = if total_duration.as_secs_f64() > 0.0 {
            total as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let error_rate = if total > 0 {
            (self.failed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Convert microseconds to milliseconds
        let to_ms = |micros: u64| micros as f64 / 1000.0;

        PerfMetrics {
            total_requests: total,
            successful_requests: self.successful,
            failed_requests: self.failed,
            total_duration_ms,
            latency_min_ms: to_ms(self.histogram.min()),
            latency_max_ms: to_ms(self.histogram.max()),
            latency_avg_ms: to_ms(self.histogram.mean() as u64),
            latency_p50_ms: to_ms(self.histogram.value_at_percentile(50.0)),
            latency_p95_ms: to_ms(self.histogram.value_at_percentile(95.0)),
            latency_p99_ms: to_ms(self.histogram.value_at_percentile(99.0)),
            requests_per_second,
            error_rate_percent: error_rate,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector() {
        let collector = MetricsCollector::new();
        let metrics = collector.compute_metrics();
        assert_eq!(metrics.total_requests, 0);
    }

    #[test]
    fn test_record_success() {
        let mut collector = MetricsCollector::new();
        collector.record_success(Duration::from_millis(100));
        collector.record_success(Duration::from_millis(200));
        let metrics = collector.compute_metrics();
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[test]
    fn test_record_failure() {
        let mut collector = MetricsCollector::new();
        collector.record_failure(Duration::from_millis(100));
        let metrics = collector.compute_metrics();
        assert_eq!(metrics.failed_requests, 1);
    }

    #[test]
    fn test_error_rate() {
        let mut collector = MetricsCollector::new();
        collector.record_success(Duration::from_millis(100));
        collector.record_failure(Duration::from_millis(100));
        let metrics = collector.compute_metrics();
        assert!((metrics.error_rate_percent - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut collector = MetricsCollector::new();
        for i in 1..=100 {
            collector.record_success(Duration::from_millis(i));
        }
        let metrics = collector.compute_metrics();
        assert!(metrics.latency_p50_ms >= 49.0 && metrics.latency_p50_ms <= 51.0);
        assert!(metrics.latency_p99_ms >= 98.0);
    }
}
