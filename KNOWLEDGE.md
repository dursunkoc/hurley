# Project Knowledge

## Rust / Cargo

### Binary-only crate — `cargo test --lib` is invalid
**Context:** `hurley` is a binary-only crate (`[[bin]]` target, no `[lib]` section in `Cargo.toml`).
`cargo test --lib <module>` fails with *"no library targets found in package"*.
**Fix:** Use plain `cargo test` to run all unit tests, or `cargo test <filter>` to target a module (e.g., `cargo test perf::datafile`).
**Why it matters:** Planning templates and verification commands in GSD tasks frequently use `cargo test --lib`; this will always fail for hurley. Use `cargo test` instead.
Added: M001/S01

---

## DataFile module (`src/perf/datafile.rs`)

### JSON column order is non-deterministic
HashMap iteration order is not guaranteed in Rust. `DataFile::columns()` returns column names in HashMap insertion order (which for JSON objects depends on serde_json's parsing). This is fine for substitution-by-name but don't rely on column index ordering.
Added: M001/S01

### CSV headers must be cloned before iterating records
The `csv::Reader` borrows headers from the reader; iterating records while holding a `&headers` reference causes a borrow conflict. Clone headers into an owned `Vec<String>` before the record loop.
Added: M001/S01
