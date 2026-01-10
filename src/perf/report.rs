//! Performance test report formatting.
//!
//! Supports text output with colored formatting and JSON export.

use colored::Colorize;
use super::metrics::PerfMetrics;

/// Performance report formatter.
///
/// Outputs metrics in human-readable text format or machine-readable JSON.
pub struct PerfReport;

impl PerfReport {
    /// Prints metrics in colored text format.
    ///
    /// Includes request summary, timing information, and latency distribution.
    pub fn print_text(metrics: &PerfMetrics) {
        println!();
        println!("{}", "═══════════════════════════════════════════════════════════".cyan());
        println!("{}", "                    PERFORMANCE RESULTS                     ".cyan().bold());
        println!("{}", "═══════════════════════════════════════════════════════════".cyan());
        println!();

        // Request Summary
        println!("{}", "📊 Request Summary".white().bold());
        println!("   Total Requests:      {}", metrics.total_requests.to_string().cyan());
        println!("   Successful:          {}", metrics.successful_requests.to_string().green());
        println!("   Failed:              {}", 
            if metrics.failed_requests > 0 {
                metrics.failed_requests.to_string().red()
            } else {
                metrics.failed_requests.to_string().green()
            }
        );
        println!("   Error Rate:          {:.2}%", metrics.error_rate_percent);
        println!();

        // Timing
        println!("{}", "⏱️  Timing".white().bold());
        println!("   Total Duration:      {:.2} ms", metrics.total_duration_ms);
        println!("   Requests/sec:        {}", format!("{:.2}", metrics.requests_per_second).yellow().bold());
        println!();

        // Latency Distribution
        println!("{}", "📈 Latency Distribution".white().bold());
        println!("   Min:                 {:.2} ms", metrics.latency_min_ms);
        println!("   Max:                 {:.2} ms", metrics.latency_max_ms);
        println!("   Avg:                 {:.2} ms", metrics.latency_avg_ms);
        println!("   p50 (Median):        {:.2} ms", metrics.latency_p50_ms);
        println!("   p95:                 {:.2} ms", metrics.latency_p95_ms);
        println!("   p99:                 {:.2} ms", metrics.latency_p99_ms);
        println!();
        println!("{}", "═══════════════════════════════════════════════════════════".cyan());
    }

    /// Prints metrics in JSON format.
    ///
    /// Useful for programmatic consumption and integration with other tools.
    pub fn print_json(metrics: &PerfMetrics) {
        match serde_json::to_string_pretty(metrics) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Failed to serialize metrics: {}", e),
        }
    }

    /// Prints metrics in the specified format.
    ///
    /// # Arguments
    ///
    /// * `metrics` - Performance metrics to print
    /// * `format` - Output format ("json" or "text")
    pub fn print(metrics: &PerfMetrics, format: &str) {
        match format.to_lowercase().as_str() {
            "json" => Self::print_json(metrics),
            _ => Self::print_text(metrics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics() -> PerfMetrics {
        PerfMetrics {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            total_duration_ms: 1000.0,
            latency_min_ms: 10.0,
            latency_max_ms: 100.0,
            latency_avg_ms: 50.0,
            latency_p50_ms: 45.0,
            latency_p95_ms: 90.0,
            latency_p99_ms: 98.0,
            requests_per_second: 100.0,
            error_rate_percent: 5.0,
        }
    }

    #[test]
    fn test_json_serialization() {
        let metrics = sample_metrics();
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("total_requests"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_metrics_fields() {
        let metrics = sample_metrics();
        assert_eq!(metrics.total_requests, 100);
        assert_eq!(metrics.failed_requests, 5);
        assert!((metrics.error_rate_percent - 5.0).abs() < 0.01);
    }
}
