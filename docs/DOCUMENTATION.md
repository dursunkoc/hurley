# Hurley — Comprehensive Documentation

**hurley** is a curl-like HTTP client with performance testing capabilities, written in Rust. It combines functional API testing and load testing into a single CLI tool, eliminating context switching between separate toolsets.

---

## Table of Contents

- [Hurley — Comprehensive Documentation](#hurley--comprehensive-documentation)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Via Cargo (Recommended)](#via-cargo-recommended)
    - [From Source](#from-source)
  - [Operating Modes](#operating-modes)
  - [Manual Usage](#manual-usage)
    - [1. Single HTTP Requests](#1-single-http-requests)
      - [Basic Examples](#basic-examples)
      - [POST with Body](#post-with-body)
      - [PUT, DELETE, PATCH](#put-delete-patch)
      - [Multiple Headers](#multiple-headers)
    - [2. Performance Testing](#2-performance-testing)
      - [Concurrency Flags](#concurrency-flags)
      - [Examples](#examples)
      - [Multi-endpoint Load Testing with `--perf`](#multi-endpoint-load-testing-with---perf)
    - [3. Parameterized Load Testing with `--data-file`](#3-parameterized-load-testing-with---data-file)
      - [Flag](#flag)
      - [Substitution Scope](#substitution-scope)
      - [CSV Format](#csv-format)
      - [JSON Format](#json-format)
      - [Examples](#examples-1)
    - [4. Workflows — Conditional Multi-step Execution](#4-workflows--conditional-multi-step-execution)
      - [Flag](#flag-1)
      - [Workflow JSON Format](#workflow-json-format)
      - [Condition Syntax](#condition-syntax)
      - [Full Workflow Example (`flow.json`)](#full-workflow-example-flowjson)
  - [Performance Metrics Output](#performance-metrics-output)
  - [CLI Flag Reference](#cli-flag-reference)
    - [Universal Flags](#universal-flags)
    - [Performance Flags](#performance-flags)
    - [Data Substitution Flag](#data-substitution-flag)
    - [Workflow Flag](#workflow-flag)
  - [Usage via SKILL with an LLM Agent](#usage-via-skill-with-an-llm-agent)
    - [What the Skill Does](#what-the-skill-does)
    - [How to Invoke It](#how-to-invoke-it)
    - [Example Prompts and Generated Commands](#example-prompts-and-generated-commands)
    - [Quick Decision Guide](#quick-decision-guide)
  - [Common Pitfalls](#common-pitfalls)

---

## Installation

### Via Cargo (Recommended)

```bash
cargo install hurley
```

### From Source

```bash
git clone https://github.com/dursunkoc/hurley.git
cd hurley
cargo build --release
# Binary available at: target/release/hurley
```

---

## Operating Modes

hurley has four distinct operating modes selected by the flags you provide:

| Mode | Trigger | Description |
|------|---------|-------------|
| **Single request** | Default (no `-c`, `-n`, `--perf`) | One HTTP call, response printed to stdout |
| **Standalone data-file** | `--data-file` only (default `-n 1`) | One request per data row, executed sequentially |
| **Performance test** | `-n > 1` or `-c > 1` or `--perf` | Concurrent load test with full latency metrics |
| **Workflow** | `--workflow <file>` | Sequential steps with conditional execution logic |

---

## Manual Usage

### 1. Single HTTP Requests

The default mode — sends one request and prints the response body.

#### Basic Examples

```bash
# Simple GET
hurley https://httpbin.org/get

# GET and include response headers in output
hurley -i https://httpbin.org/get

# GET with verbose output (shows request details before sending)
hurley -v https://httpbin.org/get

# Follow HTTP redirects (up to 10)
hurley -L https://httpbin.org/redirect/3

# Custom request timeout (seconds)
hurley --timeout 10 https://slow-api.example.com/data
```

#### POST with Body

```bash
# Inline JSON body
hurley -X POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "value": 123}'

# Body from file
hurley -X POST https://httpbin.org/post \
  -H "Content-Type: application/json" \
  -f payload.json
```

#### PUT, DELETE, PATCH

```bash
# PUT with auth header and body
hurley -X PUT https://api.example.com/items/42 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"status": "active"}'

# DELETE
hurley -X DELETE https://api.example.com/items/42

# PATCH
hurley -X PATCH https://api.example.com/items/42 \
  -H "Content-Type: application/json" \
  -d '{"field": "new_value"}'
```

#### Multiple Headers

```bash
hurley https://api.example.com/resource \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token123" \
  -H "X-Request-ID: abc-456"
```

---

### 2. Performance Testing

Activate by providing `-n > 1`, `-c > 1`, or `--perf`. hurley spawns concurrent async workers and collects latency metrics.

#### Concurrency Flags

| Flag | Description |
|------|-------------|
| `-c <N>` | Number of concurrent connections |
| `-n <M>` | Total number of requests to send |
| `--perf <file>` | Path to a JSON dataset for multi-endpoint load testing |
| `--output <fmt>` | Output format: `text` (default) or `json` |

#### Examples

```bash
# 100 requests, 10 concurrent
hurley https://httpbin.org/get -c 10 -n 100

# POST load test — 500 requests, 20 concurrent
hurley -X POST https://api.example.com/events \
  -H "Content-Type: application/json" \
  -d '{"event": "ping"}' \
  -c 20 -n 500

# Machine-readable JSON output
hurley https://httpbin.org/get -c 5 -n 50 --output json
```

#### Multi-endpoint Load Testing with `--perf`

Use a JSON dataset to distribute requests across multiple endpoints:

```bash
hurley https://api.example.com --perf requests.json -c 20 -n 500
```

**Dataset format** (`requests.json`):

```json
[
  {"method": "GET", "path": "/users"},
  {"method": "POST", "path": "/users", "body": {"name": "test"}},
  {"method": "GET", "path": "/users/1", "headers": {"Authorization": "Bearer token"}},
  {"method": "DELETE", "path": "/users/99"}
]
```

Each entry supports:
- `method` — HTTP method (default: `GET`)
- `path` — relative path appended to the base URL, or an absolute URL
- `body` — JSON object used as the request body (optional)
- `headers` — key-value map of additional headers (optional)

hurley cycles through the dataset entries to reach the total request count. Endpoint metrics are broken down individually in the report.

---

### 3. Parameterized Load Testing with `--data-file`

Use `{{placeholder}}` tokens in the URL, headers, and body, and provide a CSV or JSON file whose rows supply values for each placeholder. Rows are cycled sequentially across all requests.

#### Flag

| Flag | Description |
|------|-------------|
| `--data-file <path>` | CSV (with header row) or JSON array of objects |

#### Substitution Scope

Placeholders are replaced in:
- The URL path
- Header values passed via `-H`
- The inline body passed via `-d`

Placeholder names are **case-sensitive** and must exactly match column headers.

#### CSV Format

```csv
user_id,api_token,role
101,abc123token,admin
102,def456token,user
103,ghi789token,viewer
```

#### JSON Format

```json
[
  {"user_id": "101", "api_token": "abc123token", "role": "admin"},
  {"user_id": "102", "api_token": "def456token", "role": "user"},
  {"user_id": "103", "api_token": "ghi789token", "role": "viewer"}
]
```

#### Examples

```bash
# Performance load test — 1000 requests cycling through CSV rows
hurley -X POST https://api.example.com/users/{{user_id}} \
  -H "Authorization: Bearer {{api_token}}" \
  -d '{"role": "{{role}}"}' \
  --data-file users.csv -c 10 -n 1000

# Standalone mode — one sequential request per row (no -n or -c)
hurley https://api.example.com/users/{{user_id}} --data-file users.csv

# Parameterized with auth header in URL path and body
hurley -X GET https://api.example.com/products/{{product_id}} \
  -H "X-Tenant: {{tenant}}" \
  --data-file products.csv
```

> **Note:** `--data-file` alone (without `-n > 1` or `-c > 1`) does **not** enter performance mode. It sends one request per data row sequentially.

---

### 4. Workflows — Conditional Multi-step Execution

Use `--workflow` to run a sequence of HTTP steps where each step can access JSON from prior responses, and execute or skip based on a condition expression.

#### Flag

| Flag | Description |
|------|-------------|
| `--workflow <file>` | Path to JSON workflow definition |

The base URL argument is required even in workflow mode and is used for relative `path` values in steps.

```bash
hurley --workflow flow.json https://httpbin.org
```

#### Workflow JSON Format

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
      "condition": "responses.step_id.some.json.path == \"expected_value\"",
      "request": {
        "method": "POST",
        "path": "/other-endpoint",
        "body": {"key": "value"},
        "headers": {"X-Custom": "header-value"}
      }
    }
  ]
}
```

Each step may contain:

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier, used to reference the step's response in later conditions |
| `request.method` | No (default: GET) | HTTP method |
| `request.path` | Yes | Relative path or absolute URL |
| `request.body` | No | JSON object used as the request body |
| `request.headers` | No | Key-value map of additional headers |
| `condition` | No | Expression evaluated before the step; step is skipped if `false` |

#### Condition Syntax

Reference prior step responses using dot-path notation:

```
responses.<step_id>.<dot.separated.path>
```

Supported operators:
- Equality: `== "string"` or `== number` or `== true/false`
- Inequality: `!= "string"`
- Numeric comparison: `> number`, `< number`, `>= number`, `<= number`

Steps without a `condition` always execute.

#### Full Workflow Example (`flow.json`)

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
    },
    {
      "id": "always_runs",
      "request": {
        "method": "GET",
        "path": "https://httpbin.org/status/200"
      }
    }
  ]
}
```

```bash
hurley --workflow flow.json https://httpbin.org
```

Workflow output is colored and sequential:

```
🚀 Starting Workflow Execution
➜ Step: get_user
➜ Step: post_if_match
  Evaluating condition: responses.get_user.slideshow.author == "Yours Truly" ... Matched
➜ Step: always_runs
✅ Workflow Completed
```

---

## Performance Metrics Output

When running in performance mode, hurley prints a structured report:

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
   Total Requests:  60    Successful: 60    Error Rate: 0.00%
   Requests/sec:    19.23
   p50: 72.10 ms   p95: 120.45 ms   p99: 140.23 ms

📍 POST /users
   Total Requests:  40    Successful: 38    Error Rate: 5.00%
   Requests/sec:    18.92
   p50: 95.67 ms   p95: 250.34 ms   p99: 287.12 ms
═══════════════════════════════════════════════════════════
```

Use `--output json` to get machine-readable output:

```bash
hurley https://api.example.com/health -c 10 -n 100 --output json
```

---

## CLI Flag Reference

### Universal Flags

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| *(positional)* | | — | Target URL (required) |
| `-X` | `--method` | `GET` | HTTP method |
| `-H` | `--header` | — | Header in `"Name: Value"` format; repeatable |
| `-d` | `--data` | — | Inline request body |
| `-f` | `--file` | — | Read request body from file |
| `-i` | `--include` | false | Include response headers in output |
| `-L` | `--location` | false | Follow redirects (up to 10) |
| `-v` | `--verbose` | false | Print request details before sending |
| `--timeout` | | `30` | Request timeout in seconds |

### Performance Flags

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-c` | `--concurrency` | `1` | Concurrent connections |
| `-n` | `--requests` | `1` | Total number of requests |
| `--perf` | | — | JSON dataset file for multi-endpoint load test |
| `--output` | | `text` | Output format: `text` or `json` |

### Data Substitution Flag

| Flag | Long | Description |
|------|------|-------------|
| `--data-file` | | CSV or JSON file for `{{placeholder}}` substitution |

### Workflow Flag

| Flag | Long | Description |
|------|------|-------------|
| `--workflow` | | JSON workflow definition file |

---

## Usage via SKILL with an LLM Agent

Hurley ships with a **SKILL** file (`/.github/skills/hurley/SKILL.md`) that allows LLM-powered coding agents (GitHub Copilot, etc.) to understand and generate hurley commands automatically from natural language.

### What the Skill Does

The SKILL gives the agent:
- A complete reference of every flag hurley supports
- The four operating modes and their trigger conditions
- Dataset and workflow JSON schemas with annotated examples
- A quick-decision table mapping goals to command shapes
- A list of common pitfalls to avoid generating broken commands

This makes it possible to say *"run a load test against my users API with 50 concurrency"* and receive a correct, production-ready hurley command without memorizing flag syntax.

### How to Invoke It

The SKILL is loaded automatically by GitHub Copilot when your request matches hurley's description:

> "Use hurley to make HTTP requests, run performance/load tests, parameterize requests with data files, or execute multi-step conditional workflows."

The agent reads the SKILL.md before generating any response, ensuring it uses the correct flags and formats. You do not need to reference the SKILL explicitly — just describe what you want to do.

### Example Prompts and Generated Commands

---

**Prompt:** *"Send a GET request to https://api.example.com/status and show me the response headers."*

```bash
hurley -i https://api.example.com/status
```

---

**Prompt:** *"POST a JSON payload `{"event": "login", "user_id": 42}` to https://api.example.com/events with a Bearer token."*

```bash
hurley -X POST https://api.example.com/events \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"event": "login", "user_id": 42}'
```

---

**Prompt:** *"Load test my GET /products endpoint with 50 concurrent connections and 2000 total requests."*

```bash
hurley https://api.example.com/products -c 50 -n 2000
```

---

**Prompt:** *"Run a parameterized load test against POST /orders where each request uses a different customer_id and api_key from my data file customers.csv, with 20 concurrency and 500 requests."*

```bash
hurley -X POST https://api.example.com/orders/{{customer_id}} \
  -H "Authorization: Bearer {{api_key}}" \
  -H "Content-Type: application/json" \
  -d '{"source": "web"}' \
  --data-file customers.csv -c 20 -n 500
```

Assumed `customers.csv`:
```csv
customer_id,api_key
1001,tok_abc123
1002,tok_def456
```

---

**Prompt:** *"Create a workflow that first fetches the current user from /api/me, and if the account status is 'active', posts to /api/sessions to create a session."*

The agent generates `workflow.json`:

```json
{
  "steps": [
    {
      "id": "get_me",
      "request": {
        "method": "GET",
        "path": "/api/me"
      }
    },
    {
      "id": "create_session",
      "condition": "responses.get_me.status == \"active\"",
      "request": {
        "method": "POST",
        "path": "/api/sessions",
        "body": {"client": "cli"}
      }
    }
  ]
}
```

And the command:

```bash
hurley --workflow workflow.json https://api.example.com
```

---

**Prompt:** *"Give me JSON output for a load test on /health — 100 requests, 10 concurrent."*

```bash
hurley https://api.example.com/health -c 10 -n 100 --output json
```

---

**Prompt:** *"Iterate through every row in my users.json file and send one GET request per user — no load test, just sequentially."*

```bash
hurley https://api.example.com/users/{{user_id}} --data-file users.json
```

Assumed `users.json`:
```json
[
  {"user_id": "1"},
  {"user_id": "2"},
  {"user_id": "3"}
]
```

---

### Quick Decision Guide

The SKILL provides this decision table that the agent uses internally to select the right command shape:

| Goal | Command Shape |
|------|--------------|
| Single API call | `hurley [-X METHOD] <url> [-H ...] [-d body]` |
| Show response headers | add `-i` |
| Debug request details | add `-v` |
| Follow redirects | add `-L` |
| Load test single endpoint | add `-c <N> -n <M>` |
| Load test multiple endpoints | `--perf dataset.json -c <N> -n <M>` |
| Data-driven substitution | `--data-file data.csv` (+ `-n`/`-c` for perf mode) |
| Sequential one-per-row requests | `--data-file data.csv` (no `-n`/`-c`) |
| Multi-step conditional flow | `--workflow flow.json <base-url>` |
| Machine-readable results | add `--output json` |

---

## Common Pitfalls

| Pitfall | Explanation |
|---------|-------------|
| `--data-file` alone is not perf mode | Without `-n > 1` or `-c > 1`, hurley sends one request per row sequentially. This is intentional for iterating datasets without load. |
| `--perf` vs `--workflow` confusion | `--perf` is for load testing a JSON dataset of endpoints concurrently. `--workflow` is for sequential dependent steps with conditions. They serve different purposes. |
| Case-sensitive placeholder names | `{{User_Id}}` and `{{user_id}}` are different. Placeholder names must exactly match CSV/JSON column headers. Hurley reports all missing placeholders before aborting. |
| Absolute URLs in workflow steps | When a workflow `path` does not start with `http://` or `https://`, it is appended to the base URL. Always provide an appropriate base URL argument. |
| Missing `-X` for POST/PUT | The default method is GET. Always specify `-X POST`, `-X PUT`, etc. explicitly. |
| `-n 1` with `-c 1` is not perf mode | Both must satisfy `> 1` to trigger performance mode. Use at minimum `-n 2` or `-c 2`. |
