# hurley

A curl-like HTTP client with performance testing capabilities, written in Rust.

[![Crates.io](https://img.shields.io/crates/v/hurley.svg)](https://crates.io/crates/hurley)
[![Documentation](https://docs.rs/hurley/badge.svg)](https://docs.rs/hurley)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
- **Custom Headers**: `-H "Content-Type: application/json"`
- **Request Body**: Inline (`-d`) or from file (`-f`)
- **Follow Redirects**: `-L`
- **Verbose Output**: `-v`
- **Performance Testing**: Concurrent requests with latency metrics

## Installation

```bash
cargo install hurley
```

Or build from source:

```bash
git clone https://github.com/dursunkoc/hurley.git
cd hurley
cargo build --release
```

## Usage

### Basic HTTP Requests

```bash
# Simple GET request
hurley https://httpbin.org/get

# POST with JSON body
hurley -X POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 123}'

# Include response headers
hurley -i https://httpbin.org/get

# Verbose output
hurley -v https://httpbin.org/get

# Follow redirects
hurley -L https://httpbin.org/redirect/3
```

### Performance Testing

```bash
# 100 requests with 10 concurrent connections
hurley https://httpbin.org/get -c 10 -n 100

# Performance test with dataset
hurley https://httpbin.org --perf requests.json -c 20 -n 500

# JSON output for programmatic use
hurley https://httpbin.org/get -c 5 -n 50 --output json
```

### Dataset Format

Create a JSON file with request definitions:

```json
[
  {"method": "GET", "path": "/users"},
  {"method": "POST", "path": "/users", "body": {"name": "test"}},
  {"method": "GET", "path": "/users/1", "headers": {"Authorization": "Bearer token"}}
]
```

## Performance Metrics

The performance test output includes:

- **Request Summary**: Total, successful, failed requests
- **Timing**: Total duration, requests/second
- **Latency Distribution**: Min, max, avg, p50, p95, p99

```
═══════════════════════════════════════════════════════════
                    PERFORMANCE RESULTS
═══════════════════════════════════════════════════════════

📊 Request Summary
   Total Requests:      100
   Successful:          98
   Failed:              2
   Error Rate:          2.00%

⏱️  Timing
   Total Duration:      5234.12 ms
   Requests/sec:        19.11

📈 Latency Distribution
   Min:                 45.23 ms
   Max:                 312.45 ms
   Avg:                 89.67 ms
   p50 (Median):        78.34 ms
   p95:                 198.23 ms
   p99:                 287.12 ms

═══════════════════════════════════════════════════════════
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Author

Dursun Koc - [@dursunkoc](https://github.com/dursunkoc)
