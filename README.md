<div align="center">
  <img src="hurley.png" alt="Hurley Logo" width="200">
</div>

# hurley

A curl-like HTTP client with performance testing capabilities, written in Rust.


<div align="center">
  <a href="https://crates.io/crates/hurley">
    <img src="https://img.shields.io/crates/v/hurley.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <a href="https://crates.io/crates/hurley">
    <img src="https://img.shields.io/crates/d/hurley.svg?style=flat-square"
      alt="Download" />
  </a>
  <a href="https://docs.rs/hurley">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
    <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg"
      alt="License" />
  </a>
  </a>
    <a href="https://github.com/dursunkoc/hurley/actions/workflows/release.yml">
    <img src="https://github.com/dursunkoc/hurley/actions/workflows/release.yml/badge.svg"
      alt="Release" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="docs/DOCUMENTATION.md">
      Documentation
    </a>
    <span> | </span>
    <a href="https://docs.rs/hurley">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/dursunkoc/hurley/releases">
      Releases
    </a>
  </h3>
</div>

## Features

- **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
- **Custom Headers**: `-H "Content-Type: application/json"`
- **Request Body**: Inline (`-d`) or from file (`-f`)
- **Follow Redirects**: `-L`
- **Verbose Output**: `-v`
- **Performance Testing**: Concurrent requests with latency metrics

## Installation

### macOS and Linux

You can install the pre-compiled binary (for Apple Silicon/Intel Macs and Linux) via our shell script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/dursunkoc/hurley/releases/latest/download/hurley-installer.sh | sh
```

### Windows

You can install the pre-compiled binary via our PowerShell script:

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/dursunkoc/hurley/releases/latest/download/hurley-installer.ps1 | iex"
```

### Pre-compiled Binaries (Manual)

Standalone binaries for Windows, macOS, and Linux are available on the [Releases page](https://github.com/dursunkoc/hurley/releases). Simply download the archive for your system, extract it, and place the `hurley` executable in your system's `PATH`.

### Using Cargo

If you have Rust and Cargo installed, you can build and install directly:

```bash
cargo install hurley
```

### Build from source

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

### Parameterized Load Testing

hurley supports data-driven test scenarios using the `--data-file` flag with CSV or JSON datasets and request templates containing `{{column_name}}` placeholders.

**Features:**
- **Data Files:** `--data-file` accepts CSV (with headers) or JSON array of objects.
- **Substitution:** Placeholders like `{{user_id}}` are substituted in URL paths, headers, and request bodies.
- **Sequential Cycling:** Requests cycle sequentially through dataset rows, allowing deterministic load simulation.

```bash
# Parameterized load test with CSV data
hurley -X POST https://api.example.com/users/{{user_id}} \
  -H "Authorization: Bearer {{api_token}}" \
  -d '{"role": "{{role}}"}' \
  --data-file users.csv -c 10 -n 1000
```

```csv
# users.csv
user_id,api_token,role
101,abc123token,admin
102,def456token,user
103,ghi789token,viewer
```

```bash
# Single parameterized request (no -n or -c, creates one request per data row)
hurley https://api.example.com/users/{{user_id}} --data-file users.csv
```

### Workflows (Conditional Execution)

hurley supports running multi-step execution workflows based on conditional responses. By passing a JSON file to `--workflow`, you can execute dependent requests dynamically.

```bash
hurley --workflow flow.json https://httpbin.org
```

**`flow.json` format:**

```json
{
  "steps": [
    {
      "id": "get_user",
      "request": {
        "method": "GET",
        "path": "/json"
      }
    },
    {
      "id": "conditional_step",
      "condition": "responses.get_user.slideshow.author == \"Yours Truly\"",
      "request": {
        "method": "POST",
        "path": "/post",
        "body": {"message": "Success! The author matched."}
      }
    }
  ]
}
```

The condition is evaluated against JSON responses of previous steps. Use `responses.<step_id>.<json_path>` to query values.

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
- **Endpoint Breakdown**: Detailed metrics for each unique endpoint (when using datasets)

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
                    ENDPOINT BREAKDOWN
═══════════════════════════════════════════════════════════

📍 GET /users
───────────────────────────────────────────────────────────
📊 Request Summary
   Total Requests:      60
   Successful:          60
   Failed:              0
   Error Rate:          0.00%

⏱️  Timing
   Total Duration:      3120.45 ms
   Requests/sec:        19.23

📈 Latency Distribution
   Min:                 45.23 ms
   Max:                 150.12 ms
   Avg:                 75.34 ms
   p50 (Median):        72.10 ms
   p95:                 120.45 ms
   p99:                 140.23 ms

📍 POST /users
───────────────────────────────────────────────────────────
📊 Request Summary
   Total Requests:      40
   Successful:          38
   Failed:              2
   Error Rate:          5.00%

⏱️  Timing
   Total Duration:      2113.67 ms
   Requests/sec:        18.92

📈 Latency Distribution
   Min:                 80.12 ms
   Max:                 312.45 ms
   Avg:                 110.23 ms
   p50 (Median):        95.67 ms
   p95:                 250.34 ms
   p99:                 287.12 ms

═══════════════════════════════════════════════════════════
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Author

Dursun Koc - [@dursunkoc](https://github.com/dursunkoc)
