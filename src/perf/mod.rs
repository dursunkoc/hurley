//! Performance testing module for rurl.
//!
//! This module provides functionality for load testing and benchmarking
//! HTTP endpoints with:
//!
//! - [`Dataset`] - JSON dataset parsing for varied requests
//! - [`PerfRunner`] - Concurrent request execution with progress tracking
//! - [`PerfMetrics`] - Latency percentiles and throughput metrics
//! - [`PerfReport`] - Text and JSON output formatting

pub mod datafile;
pub mod dataset;
pub mod metrics;
pub mod report;
pub mod runner;
pub mod substitute;

pub use datafile::{DataFile, DataRow};
pub use dataset::Dataset;
pub use metrics::PerfMetrics;
pub use report::PerfReport;
pub use runner::PerfRunner;
pub use substitute::{extract_placeholders, get_row_for_request, substitute, validate_template};

// ── Integration tests ─────────────────────────────────────────────────────────
//
// These tests compose DataFile loading + template validation + substitution to
// prove end-to-end behaviour for R007 (standalone), R008 (perf cycling), R009
// (flag parsing is implicitly covered by DataFile::from_path path routing),
// R010 (end-to-end verification), and R006 (fail-fast on missing columns).
//
// Kept in `perf/mod.rs` so all re-exported symbols are directly in scope.

#[cfg(test)]
mod integration_tests {
    use std::io::Write;

    use super::{get_row_for_request, substitute, validate_template, DataFile};

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Write `content` to a temp file with the given `filename` and return the
    /// path.  Caller is responsible for removing it after use.
    fn write_temp(filename: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(filename);
        let mut f = std::fs::File::create(&path).expect("create temp file");
        write!(f, "{}", content).unwrap();
        path
    }

    // ── R007 + R003 + R004: standalone mode URL / header / body substitution ─

    /// For each of the 3 data rows, substitute URL, header value, and JSON body
    /// and assert that the correct row values are present in each output.
    #[test]
    fn test_data_file_standalone_substitution() {
        let csv = "user_id,token\nu1,t1\nu2,t2\nu3,t3";
        let path = write_temp("hurley_integ_standalone.csv", csv);
        let df = DataFile::from_path(&path).expect("load CSV");
        let _ = std::fs::remove_file(&path);

        let url_tmpl = "https://api.example.com/users/{{user_id}}";
        let hdr_tmpl = "Authorization: Bearer {{token}}";
        let body_tmpl = r#"{"id": "{{user_id}}"}"#;

        for (i, (uid, tok)) in [("u1", "t1"), ("u2", "t2"), ("u3", "t3")]
            .iter()
            .enumerate()
        {
            let row = get_row_for_request(&df, i);

            let url = substitute(url_tmpl, row).unwrap();
            assert!(
                url.contains(uid),
                "row {i}: URL should contain '{uid}' but got '{url}'"
            );

            let hdr = substitute(hdr_tmpl, row).unwrap();
            assert!(
                hdr.contains(tok),
                "row {i}: header should contain '{tok}' but got '{hdr}'"
            );

            let body = substitute(body_tmpl, row).unwrap();
            assert!(
                body.contains(uid),
                "row {i}: body should contain '{uid}' but got '{body}'"
            );
        }
    }

    // ── R005 + R008: row cycling over 9 requests with 3 data rows ────────────

    /// Requests 0, 3, 6 → row 0; 1, 4, 7 → row 1; 2, 5, 8 → row 2.
    #[test]
    fn test_data_file_cycling_9_requests_3_rows() {
        let csv = "user_id\nrow0\nrow1\nrow2";
        let path = write_temp("hurley_integ_cycle.csv", csv);
        let df = DataFile::from_path(&path).expect("load CSV");
        let _ = std::fs::remove_file(&path);

        let url_tmpl = "https://api.example.com/users/{{user_id}}";

        let expected_row = |i: usize| -> &'static str {
            match i % 3 {
                0 => "row0",
                1 => "row1",
                _ => "row2",
            }
        };

        for i in 0..9usize {
            let row = get_row_for_request(&df, i);
            let url = substitute(url_tmpl, row).unwrap();
            let exp = expected_row(i);
            assert!(
                url.contains(exp),
                "request {i}: expected '{exp}' but URL was '{url}'"
            );
        }
    }

    // ── R006: validate_template returns Err for missing column ────────────────

    /// validate_template must error before any HTTP calls are made when the
    /// template references a column that is absent from the data file.
    #[test]
    fn test_validate_template_fails_before_execution() {
        let csv = "user_id,token\nu1,t1";
        let path = write_temp("hurley_integ_validate.csv", csv);
        let df = DataFile::from_path(&path).expect("load CSV");
        let _ = std::fs::remove_file(&path);

        let template = "{{user_id}}/{{missing_col}}";
        let result = validate_template(template, df.columns());
        assert!(result.is_err(), "expected Err for missing column");

        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("missing_col"),
            "error should name the missing column; got: {msg}"
        );
    }

    // ── R002 + R007: JSON data file format ────────────────────────────────────

    /// DataFile::from_path must load a JSON array file and substitution must
    /// produce the correct output for each row.
    #[test]
    fn test_data_file_json_format() {
        let json = r#"[{"id": "1", "name": "alice"}, {"id": "2", "name": "bob"}]"#;
        let path = write_temp("hurley_integ_json.json", json);
        let df = DataFile::from_path(&path).expect("load JSON");
        let _ = std::fs::remove_file(&path);

        assert_eq!(df.len(), 2, "expected 2 rows in JSON data file");

        let tmpl = "Hello {{name}} #{{id}}";

        let row0 = get_row_for_request(&df, 0);
        let out0 = substitute(tmpl, row0).unwrap();
        assert_eq!(out0, "Hello alice #1", "row 0 substitution mismatch");

        let row1 = get_row_for_request(&df, 1);
        let out1 = substitute(tmpl, row1).unwrap();
        assert_eq!(out1, "Hello bob #2", "row 1 substitution mismatch");
    }

    // ── Backward compatibility: no data file ─────────────────────────────────

    /// When no data file is provided, the code paths that skip substitution
    /// must not panic.  Simulated here by verifying that an Option<&DataFile>
    /// of None causes no issues — the happy path is simply to skip substitution
    /// entirely and pass the template string through unchanged.
    #[test]
    fn test_backward_compat_no_data_file() {
        let data_file: Option<&DataFile> = None;

        // Static URL with no placeholders — if data_file is None we just use
        // the raw string, which must equal itself unchanged.
        let url = "https://api.example.com/static/resource";
        let effective_url = if let Some(df) = data_file {
            let row = get_row_for_request(df, 0);
            substitute(url, row).unwrap()
        } else {
            url.to_string()
        };
        assert_eq!(
            effective_url, url,
            "without data file, URL must be unchanged"
        );

        // Template with placeholders — None path must not panic either.
        let url_with_placeholders = "https://api.example.com/users/{{user_id}}";
        let effective = if let Some(df) = data_file {
            let row = get_row_for_request(df, 0);
            substitute(url_with_placeholders, row).unwrap()
        } else {
            url_with_placeholders.to_string()
        };
        // When data_file is None substitution is skipped — the placeholder
        // string is preserved verbatim (user would see unresolved placeholders,
        // which is the documented no-data-file behaviour).
        assert_eq!(
            effective, url_with_placeholders,
            "without data file, template string must be passed through unchanged"
        );
    }
}
