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
