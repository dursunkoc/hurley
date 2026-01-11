# Hurley: A High-Performance HTTP Client and Load Testing Tool Engineered in Rust

This article examines the technical architecture, capabilities, and use cases of **hurley**, a project developed in Rust that functions as both a general-purpose HTTP client and a performance testing tool. It explores the efficiency advantages gained by managing API testing and performance analysis through a unified tool within software development processes.

---

## 1. Introduction and Motivation

With the proliferation of microservices architectures and distributed systems, communication via the HTTP protocol has become the lifeblood of the software ecosystem. In this context, developers face two fundamental needs: (1) A flexible HTTP client to verify the functional correctness of API endpoints, and (2) Performance testing tools to analyze system behavior under load.

Typically, distinct toolsets are employed for these two requirements (specialized HTTP clients vs. load testing tools like `wrk` or `Apache Benchmark`). **hurley** aims to minimize context switching in development and testing processes and offer a unified testing experience by consolidating these two functions into a single command-line interface (CLI).

---

## 2. Core Capabilities and HTTP Client Mode

hurley features a client mode that demonstrates full compliance with modern HTTP standards. It supports all fundamental operations required by RESTful architectures.

### 2.1. Protocol Support and Request Structure

The tool supports all standard HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD). Request configuration can be flexibly structured via command-line arguments:

*   **Header Management:** Defining custom HTTP headers using the `-H` parameter.
*   **Payload Management:** Inline data submission via the `-d` parameter or file-based data streaming via the `-f` parameter.
*   **Redirect Policies:** Automatic tracking of HTTP 3xx series responses using the `-L` parameter.

```bash
# Example: POST request containing custom headers and payload
hurley -X POST https://api.example.com/v1/resource \
  -H "Content-Type: application/json" \
  -H "X-Client-ID: system-a" \
  -d '{"key": "value", "timestamp": 1678900000}'
```

---

## 3. Performance Testing and Load Simulation

A distinguishing feature of the tool is its ability to instantaneously transform existing HTTP requests into a load test without requiring external configuration.

### 3.1. Concurrency Model

hurley is built upon Rust's `Tokio` asynchronous runtime. This architecture allows for the management of a high number of concurrent connections while utilizing system resources (CPU and Memory) at minimal levels. The intensity of the test scenario is determined by the `-c` (concurrency) and `-n` (total requests) parameters.

### 3.2. Dataset-Based Stochastic Testing

To simulate real-world traffic patterns, hurley supports non-deterministic test scenarios. Through a dataset defined in JSON format, requests with different endpoints, methods, and payloads can be distributed randomly or sequentially. This approach is critical for eliminating the misleading effects of cache mechanisms (cache warming bias) and measuring the general stability of the system.

```json
/* Example Dataset Schema */
[
  { "method": "GET", "path": "/api/users/101" },
  { "method": "POST", "path": "/api/orders", "body": { "id": 55, "item": "A-1" } }
]
```

---

## 4. Performance Metrics and Statistical Analysis

In reporting test results, the tool goes beyond average values to present statistical distribution analyses. **Percentile** metrics are vital for detecting tail latency.

The fundamental metrics reported include:

*   **Throughput:** The number of requests processed per second (RPS).
*   **Latency Distribution:**
    *   **P50 (Median):** The completion time for 50% of requests.
    *   **P95 and P99:** System performance in the slowest 5% and 1% segments. These values are critical indicators for Service Level Agreement (SLA) compliance.
    *   **Jitter:** The standard deviation and range of variation in response times.
    *   **Endpoint Breakdown:** Detailed performance metrics separated for each endpoint (when using datasets).

```text
📊 Statistical Summary
   Total Requests:      1000
   Error Rate:          0.00%
   Requests/sec:        450.25

📈 Latency Distribution (Percentiles)
   p50 (Median):        45.12 ms
   p95:                120.45 ms
   p99:                210.88 ms

📍 Endpoint Breakdown (Details)
   GET /api/users/101
   └── p95: 110.20 ms
   POST /api/orders
   └── p95: 145.50 ms
```

---

## 5. Technical Architecture

The project is constructed upon the performance and reliability-oriented libraries of the Rust ecosystem:

*   **Asynchronous I/O:** Non-blocking I/O operations via `tokio` and `reqwest` libraries.
*   **Statistical Computation:** High dynamic range histogram analysis via the `hdrhistogram` library.
*   **Error Management:** Deterministic management of runtime errors via a strong type system and the `thiserror` library.

These architectural choices ensure that the tool delivers performance at the C/C++ level without compromising memory safety.

---

## 6. Installation

hurley can be easily installed via Cargo, Rust's package manager, or built from source.

### Via Cargo (Recommended)

```bash
cargo install hurley
```

### From Source

```bash
git clone https://github.com/dursunkoc/hurley.git
cd hurley
cargo build --release
```

The binary will be available at `target/release/hurley`.

---

## 7. Conclusion

hurley is a unified tool designed to remove the barriers between "functional testing" and "performance testing" processes in the modern API development cycle. By combining the performance advantages offered by the Rust language with a user-friendly interface, it offers strong analysis capabilities to developers and system engineers.

The project continues to be developed as open source, with plans to add features such as distributed testing capabilities and HTTP/3 support to the roadmap.

Project Source Code: [https://github.com/dursunkoc/hurley](https://github.com/dursunkoc/hurley)