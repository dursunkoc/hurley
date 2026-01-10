//! HTTP client module for rurl.
//!
//! This module provides the core HTTP functionality including:
//! - [`HttpClient`] - Executes HTTP requests
//! - [`HttpRequest`] - Request builder with method, headers, body
//! - [`HttpResponse`] - Response with status, headers, body, timing

pub mod client;
pub mod request;
pub mod response;

pub use client::HttpClient;
pub use request::HttpRequest;
pub use response::HttpResponse;
