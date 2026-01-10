//! # hurley - HTTP URL Client
//!
//! A curl-like HTTP client with performance testing capabilities.
//!
//! ## Features
//!
//! - **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
//! - **Custom Headers**: `-H "Content-Type: application/json"`
//! - **Request Body**: Inline (`-d`) or from file (`-f`)
//! - **Performance Testing**: Concurrent requests with latency metrics
//!
//! ## Usage Examples
//!
//! ```bash
//! # Simple GET request
//! hurley https://httpbin.org/get
//!
//! # POST with JSON body
//! hurley -X POST https://httpbin.org/post \
//!   -H "Content-Type: application/json" \
//!   -d '{"name": "test"}'
//!
//! # Performance test: 100 requests, 10 concurrent
//! hurley https://httpbin.org/get -c 10 -n 100
//!
//! # Performance test with dataset
//! hurley https://httpbin.org --perf data.json -c 20 -n 500
//! ```

pub mod cli;
pub mod error;
pub mod http;
pub mod perf;

use clap::Parser;
use std::time::Duration;
use colored::Colorize;

use cli::Cli;
use error::Result;
use http::{HttpClient, HttpRequest};
use perf::{Dataset, PerfRunner, PerfReport};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Build base request from CLI arguments
    let mut request = HttpRequest::new(&cli.url)
        .method(&cli.method)?
        .headers_from_strings(&cli.headers)?
        .timeout(Duration::from_secs(cli.timeout))
        .follow_redirects(cli.follow_redirects);

    // Add body from CLI
    if let Some(data) = &cli.data {
        request = request.body(data.clone());
    } else if let Some(file) = &cli.body_file {
        request = request.body_from_file(file)?;
    }

    // Performance test mode
    if cli.is_perf_mode() {
        run_perf_test(&cli, request).await?;
    } else {
        // Single request mode
        run_single_request(&cli, request).await?;
    }

    Ok(())
}

async fn run_single_request(cli: &Cli, request: HttpRequest) -> Result<()> {
    let client = HttpClient::new(cli.verbose);
    let response = client.execute(&request).await?;
    response.print(cli.include_headers, cli.verbose);
    Ok(())
}

async fn run_perf_test(cli: &Cli, base_request: HttpRequest) -> Result<()> {
    println!("{}", "🚀 Starting Performance Test".cyan().bold());
    println!("   URL: {}", cli.url.yellow());
    println!("   Concurrency: {}", cli.concurrency);
    println!("   Total Requests: {}", cli.total_requests);
    println!();

    // Load dataset
    let dataset = if let Some(file) = &cli.perf_file {
        println!("   Dataset: {}", file.display().to_string().yellow());
        Dataset::from_file(file)?
    } else {
        Dataset::simple(cli.total_requests)
    };

    let runner = PerfRunner::new(
        cli.url.clone(),
        base_request,
        cli.concurrency,
        cli.total_requests,
        cli.verbose,
    );

    let metrics = runner.run(&dataset).await?;
    
    PerfReport::print(&metrics, &cli.output_format);

    Ok(())
}
