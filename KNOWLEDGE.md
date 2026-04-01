# Project Knowledge

## Rust / Cargo

### Binary-only crate — `cargo test --lib` is invalid
**Context:** `hurley` is a binary-only crate (`[[bin]]` target, no `[lib]` section in `Cargo.toml`).
`cargo test --lib <module>` fails with *"no library targets found in package"*.
**Fix:** Use plain `cargo test` to run all unit tests, or `cargo test <filter>` to target a module (e.g., `cargo test perf::datafile`).
**Why it matters:** Planning templates and verification commands in GSD tasks frequently use `cargo test --lib`; this will always fail for hurley. Use `cargo test` instead.
Added: M001/S01

---

## Substitute module (`src/perf/substitute.rs`)

### `replace_all` closures cannot return `Result` — use a side-channel
`regex::Regex::replace_all` takes a closure that must return `String`, not `Result<String>`.
To propagate errors out of the closure, declare a `let mut error: Option<RurlError> = None;` before calling `replace_all`, set it inside the closure on failure, and inspect it after the call.
**Why it matters:** Attempting to use `?` inside a `replace_all` closure won't compile. The side-channel pattern is the correct idiom.
Added: M001/S02

### `validate_template` collects ALL missing placeholders before returning
Rather than failing on the first missing column, `validate_template` accumulates every missing name and returns one error. This gives callers an actionable, complete list instead of an iterative discovery process.
Added: M001/S02

### `get_row_for_request` takes `&DataFile`, not `&[DataRow]`
The function signature is `get_row_for_request(data_file: &DataFile, request_index: usize) -> &DataRow`. Callers hold a parsed `DataFile`, so accepting the whole struct keeps the API consistent with how callers naturally hold data.
Added: M001/S02

### Test helpers for DataFile: write CSV to `/tmp`, parse, delete
The test pattern in substitute.rs (function `make_csv_datafile`) writes a temp CSV file, calls `DataFile::from_path`, and immediately removes the file. No `tempfile` crate is needed. Use `std::env::temp_dir()` to avoid absolute `/tmp` paths on non-Unix platforms.
Added: M001/S02

---

## DataFile module (`src/perf/datafile.rs`)

### JSON column order is non-deterministic
HashMap iteration order is not guaranteed in Rust. `DataFile::columns()` returns column names in HashMap insertion order (which for JSON objects depends on serde_json's parsing). This is fine for substitution-by-name but don't rely on column index ordering.
Added: M001/S01

### CSV headers must be cloned before iterating records
The `csv::Reader` borrows headers from the reader; iterating records while holding a `&headers` reference causes a borrow conflict. Clone headers into an owned `Vec<String>` before the record loop.
Added: M001/S01

---

## CLI / main.rs wiring

### `is_perf_mode()` three-condition form — Clippy rejects a redundant fourth clause
The plan proposed adding `data_file.is_some() && total_requests > 1` as a fourth condition to `is_perf_mode()`. Clippy's `overly_complex_bool_expr` lint rejects this because the condition is a logical subset of the already-present `total_requests > 1`. Keep `is_perf_mode()` at three conditions; `--data-file` alone (default `-n 1`) does NOT trigger perf mode by design.
Added: M001/S03

### Standalone `--data-file` mode: iterate df.len() times via run_single_request, not PerfRunner
When `--data-file` is present but `-n` is 1 (default), the runner must issue one HTTP request per data row. This is handled by looping 0..df.len() in `run()` and calling `run_single_request` with the substituted request for each row index — it does not route through PerfRunner. PerfRunner is only entered when total_requests > 1.
Added: M001/S03

### Perf mode uses global enumerate() index, not per-entry index
In PerfRunner, the row cycling index must be the global request count (0..total_requests), not a per-dataset-entry counter. Using `enumerate()` on the flat requests-to-make vec gives the correct global index for modulo cycling.
Added: M001/S03

### Header substitution differs between standalone and perf paths
- Standalone: CLI headers are `Vec<String>` in "Name: Value" format. After substitution, the substituted string must be re-parsed via `headers_from_strings` or equivalent.
- Perf mode: headers inside PerfRunner are already a `HashMap<String, String>`. Substitute directly on the values without re-parsing.
Added: M001/S03

### Integration tests in `src/perf/mod.rs` can access all re-exported symbols without visibility changes
A `#[cfg(test)] mod integration_tests` block at the bottom of `src/perf/mod.rs` has direct access to everything re-exported from the perf module (DataFile, get_row_for_request, substitute, validate_template). No visibility changes to production code were needed.
Added: M001/S03
