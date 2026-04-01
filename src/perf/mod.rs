//! Performance testing module for rurl.
//!
//! This module provides functionality for load testing and benchmarking
//! HTTP endpoints with:
//!
//! - [`Dataset`] - JSON dataset parsing for varied requests
//! - [`PerfRunner`] - Concurrent request execution with progress tracking
//! - [`PerfMetrics`] - Latency percentiles and throughput metrics
//! - [`PerfReport`] - Text and JSON output formatting

pub mod dataset;
pub mod datafile;
pub mod metrics;
pub mod runner;
pub mod report;
pub mod substitute;

pub use dataset::Dataset;
pub use datafile::{DataFile, DataRow};
pub use metrics::PerfMetrics;
pub use runner::PerfRunner;
pub use report::PerfReport;
pub use substitute::{extract_placeholders, substitute, validate_template};
