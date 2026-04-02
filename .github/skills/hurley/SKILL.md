---
name: hurley
description: "Use hurley to make HTTP requests, run performance/load tests, parameterize requests with data files, or execute multi-step conditional workflows. Use when: sending curl-like HTTP requests; benchmarking APIs with concurrency; running data-driven load tests with CSV/JSON datasets; running dependent multi-step request workflows."
argument-hint: "Describe what HTTP action or test to run (e.g., POST JSON to /api/users, load test with 50 concurrency, run workflow flow.json)"
---

# hurley — HTTP Client & Performance Testing

## What hurley Does

hurley is a curl-like CLI HTTP client with four operating modes:

| Mode | Trigger | Description |
|------|---------|-------------|
| **Single request** | Default (no `-c`/`-n`/`--perf`) | One HTTP call, response to stdout |
| **Standalone data-file** | `--data-file` only (default `-n 1`) | One request per data row, sequential |
| **Performance test** | `-n > 1` or `-c > 1` or `--perf` | Concurrent load with latency metrics |
| **Workflow** | `--workflow <file>` | Sequential steps with conditional execution |

---

## 1. Single HTTP Requests

### Flags

| Flag | Long | Description |
|------|------|-------------|
| `-X` | `--method` | HTTP method: GET, POST, PUT, DELETE, PATCH, HEAD (default: GET) |
| `-H` | `--header` | Header in `"Name: Value"` format; repeatable |
| `-d` | `--data` | Inline request body |
| `-f` | `--file` | Read body from file |
| `-i` | `--include` | Print response headers in output |
| `-L` | `--location` | Follow redirects (up to 10) |
| `-v` | `--verbose` | Print request details before sending |
| `--timeout` | | Request timeout in seconds (default: 30) |

### Examples

```bash
# GET
hurley https://httpbin.org/get

# GET with response headers included
hurley -i https://httpbin.org/get

# GET verbose
hurley -v https://httpbin.org/get

# Follow redirects
hurley -L https://httpbin.org/redirect/3

# POST with inline JSON body
hurley -X POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 123}'

# POST body from file
hurley -X POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -f payload.json

# PUT with auth header
hurley -X PUT https://api.example.com/items/42 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"status": "active"}'

# DELETE
hurley -X DELETE https://api.example.com/items/42

# Custom timeout
hurley --timeout 10 https://slow-api.example.com/data
```

---

## 2. Performance Testing

Activate by setting `-n > 1`, `-c > 1`, or providing `--perf`.

### Additional Flags

| Flag | Long | Description |
|------|------|-------------|
| `-c` | `--concurrency` | Concurrent connections (default: 1) |
| `-n` | `--requests` | Total requests to send (default: 1) |
| `--perf` | | Path to JSON dataset for multi-endpoint load test |
| `--output` | | Output format: `text` (default) or `json` |

### Examples

```bash
# 100 requests, 10 concurrent
hurley https://httpbin.org/get -c 10 -n 100

# 500 requests with a multi-endpoint dataset
hurley https://httpbin.org --perf requests.json -c 20 -n 500

# Machine-readable JSON output
hurley https://httpbin.org/get -c 5 -n 50 --output json
```

### `--perf` Dataset Format (JSON array)

```json
[
  {"method": "GET", "path": "/users"},
  {"method": "POST", "path": "/users", "body": {"name": "test"}},
  {"method": "GET", "path": "/users/1", "headers": {"Authorization": "Bearer token"}}
]
```

### Performance Output

The text report includes:

- **Request Summary**: total, successful, failed, error rate
- **Timing**: total duration (ms), requests/sec
- **Latency Distribution**: min, max, avg, p50, p95, p99
- **Endpoint Breakdown**: per-path metrics when using `--perf` dataset

---

## 3. Parameterized Load Testing (`--data-file`)

Substitute `{{placeholder}}` tokens in the URL, headers, and body using rows from a CSV or JSON data file. Rows are cycled sequentially across total requests.

### Flag

| Flag | Description |
|------|-------------|
| `--data-file <path>` | CSV (with header row) or JSON array of objects |

### Substitution scope

`{{placeholder}}` is replaced in:
- URL path
- Header values (`-H`)
- Inline body (`-d`)

### CSV format

```csv
user_id,api_token,role
101,abc123,admin
102,def456,user
103,ghi789,viewer
```

### JSON format

```json
[
  {"user_id": "101", "api_token": "abc123", "role": "admin"},
  {"user_id": "102", "api_token": "def456", "role": "user"}
]
```

### Examples

```bash
# Load test — 1000 requests, cycling through CSV rows
hurley -X POST https://api.example.com/users/{{user_id}} \
  -H "Authorization: Bearer {{api_token}}" \
  -d '{"role": "{{role}}"}' \
  --data-file users.csv -c 10 -n 1000

# Standalone mode — one request per row (no -n or -c)
hurley https://api.example.com/users/{{user_id}} --data-file users.csv
```

> **Note:** `--data-file` alone (default `-n 1`) does **not** activate performance mode. It sends one request per row sequentially. Performance mode requires `-n > 1`, `-c > 1`, or `--perf`.

---

## 4. Workflows (`--workflow`)

Run multi-step sequential HTTP requests where later steps can be skipped or executed based on JSON values from earlier responses.

### Flag

| Flag | Description |
|------|-------------|
| `--workflow <file>` | Path to JSON workflow definition |

### workflow.json format

```json
{
  "steps": [
    {
      "id": "step_id",
      "request": {
        "method": "GET",
        "path": "/endpoint"
      }
    },
    {
      "id": "conditional_step",
      "condition": "responses.<step_id>.<json.path> == \"expected\"",
      "request": {
        "method": "POST",
        "path": "/other",
        "body": {"key": "value"}
      }
    }
  ]
}
```

### Condition syntax

- Reference prior responses: `responses.<step_id>.<dot.path>`
- Supports equality check: `== "string"` or `== number`
- Steps without `condition` always execute

### Example

Given `flow.json`:

```json
{
  "steps": [
    {
      "id": "get_user",
      "request": {
        "method": "GET",
        "path": "https://httpbin.org/json"
      }
    },
    {
      "id": "post_if_match",
      "condition": "responses.get_user.slideshow.author == \"Yours Truly\"",
      "request": {
        "method": "POST",
        "path": "https://httpbin.org/post",
        "body": {"message": "Author matched!"}
      }
    }
  ]
}
```

```bash
hurley --workflow flow.json https://httpbin.org
```

---

## Quick Decision Guide

| Goal | Command shape |
|------|--------------|
| Single API call | `hurley [-X METHOD] <url> [-H ...] [-d body]` |
| Check headers/verbose | add `-i` or `-v` |
| Follow redirects | add `-L` |
| Load test single endpoint | add `-c <N> -n <M>` |
| Load test multiple endpoints | `--perf dataset.json -c <N> -n <M>` |
| Data-driven substitution | `--data-file data.csv` (+ `-n` / `-c` for perf) |
| Multi-step conditional flow | `--workflow flow.json` |
| Machine-readable results | add `--output json` |

---

## Common Pitfalls

- **`--data-file` alone is not perf mode.** Without `-n > 1` or `-c > 1`, hurley sends one request per row sequentially.
- **`--perf` path vs `--workflow` path are mutually exclusive concerns.** `--perf` is for load testing multi-endpoint datasets; `--workflow` is for sequential dependent steps.
- **Placeholder names must match column headers exactly** (case-sensitive). All `{{placeholder}}` tokens must exist as columns; hurley reports all missing ones before aborting.
- **Workflow `path` must be absolute URLs** when there is no base-URL context implied by the step (check existing flow.json examples for your target API).
