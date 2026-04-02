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
pub mod workflow;

use clap::Parser;
use colored::Colorize;
use std::time::Duration;

use cli::Cli;
use error::Result;
use http::{HttpClient, HttpRequest};
use perf::{
    get_row_for_request, substitute, validate_template, DataFile, Dataset, PerfReport, PerfRunner,
};
use workflow::{Workflow, WorkflowRunner};

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

    // Load and validate data file if specified — fail fast before any HTTP call.
    let data_file: Option<DataFile> = if let Some(ref path) = cli.data_file {
        let df = DataFile::from_path(path)?;

        // Validate every template string (URL + each header + body).
        let mut templates: Vec<String> = vec![cli.url.clone()];
        templates.extend(cli.headers.iter().cloned());
        if let Some(ref data) = cli.data {
            templates.push(data.clone());
        }
        for tmpl in &templates {
            validate_template(tmpl, df.columns())?;
        }

        Some(df)
    } else {
        None
    };

    if let Some(workflow_file) = &cli.workflow_file {
        // Workflow mode
        let workflow = Workflow::from_file(workflow_file)?;
        let runner = WorkflowRunner::new(cli.url.clone(), request, cli.verbose);
        runner.run(&workflow).await?;
    } else if cli.is_perf_mode() {
        // Performance test mode
        run_perf_test(&cli, request, data_file).await?;
    } else {
        // Single request mode
        run_single_request(&cli, request, data_file.as_ref()).await?;
    }

    Ok(())
}

async fn run_single_request(
    cli: &Cli,
    request: HttpRequest,
    data_file: Option<&DataFile>,
) -> Result<()> {
    let client = HttpClient::new(cli.verbose);

    if let Some(df) = data_file {
        // Execute one request per data row — total = data_file.len()
        for i in 0..df.len() {
            let row = get_row_for_request(df, i);

            // Substitute URL
            let url = substitute(&request.url, row)?;

            // Substitute raw CLI header strings, then re-parse
            let substituted_headers: Vec<String> = cli
                .headers
                .iter()
                .map(|h| substitute(h, row))
                .collect::<Result<Vec<_>>>()?;

            // Substitute body
            let body = request
                .body
                .as_ref()
                .map(|b| substitute(b, row))
                .transpose()?;

            // Build a fresh request for this row
            let mut row_request = HttpRequest::new(url)
                .method(request.method.as_str())?
                .timeout(request.timeout)
                .follow_redirects(request.follow_redirects)
                .headers_from_strings(&substituted_headers)?;

            if let Some(b) = body {
                row_request = row_request.body(b);
            }

            let response = client.execute(&row_request).await?;
            response.print(cli.include_headers, cli.verbose);
        }
    } else {
        let response = client.execute(&request).await?;
        response.print(cli.include_headers, cli.verbose);
    }

    Ok(())
}

async fn run_perf_test(
    cli: &Cli,
    base_request: HttpRequest,
    data_file: Option<DataFile>,
) -> Result<()> {
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
        data_file,
    );

    let metrics = runner.run(&dataset).await?;

    PerfReport::print(&metrics, &cli.output_format);

    Ok(())
}
