//! Variable substitution engine for request templates.
//!
//! Replaces `{{column_name}}` (and whitespace-tolerant `{{ column_name }}`)
//! placeholders in strings with values from a [`DataRow`].
//!
//! # Quick start
//!
//! ```rust,ignore
//! let mut row = DataRow::new();
//! row.insert("user_id".into(), "42".into());
//! let result = substitute("GET /users/{{user_id}}", &row)?;
//! assert_eq!(result, "GET /users/42");
//! ```

use std::sync::LazyLock;

use regex::Regex;

use crate::error::{Result, RurlError};

use super::datafile::{DataFile, DataRow};

/// Compiled regex that matches `{{placeholder}}` with optional whitespace.
///
/// Capture group 1 is the trimmed placeholder name (`\w+`).
static PLACEHOLDER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{\s*(\w+)\s*\}\}").expect("placeholder regex is valid")
});

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns the unique placeholder names found in `template`, in order of
/// first appearance.
pub fn extract_placeholders(template: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for cap in PLACEHOLDER_RE.captures_iter(template) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            result.push(name);
        }
    }
    result
}

/// Replaces every `{{column_name}}` placeholder in `template` with the
/// corresponding value from `row`.
///
/// # Errors
///
/// Returns [`RurlError::SubstitutionError`] if a placeholder names a column
/// that is absent from `row`.  The error message includes the missing
/// placeholder name and the list of available column keys.
pub fn substitute(template: &str, row: &DataRow) -> Result<String> {
    let mut error: Option<RurlError> = None;

    let result = PLACEHOLDER_RE.replace_all(template, |caps: &regex::Captures<'_>| {
        // Short-circuit once an error has been recorded.
        if error.is_some() {
            return String::new();
        }
        let name = &caps[1];
        match row.get(name) {
            Some(value) => value.clone(),
            None => {
                let available: Vec<&str> = row.keys().map(String::as_str).collect();
                let mut available_sorted = available;
                available_sorted.sort_unstable();
                error = Some(RurlError::SubstitutionError(format!(
                    "column '{}' not found in data row; available columns: [{}]",
                    name,
                    available_sorted.join(", ")
                )));
                String::new()
            }
        }
    });

    match error {
        Some(e) => Err(e),
        None => Ok(result.into_owned()),
    }
}

/// Validates that every placeholder in `template` is present in `columns`.
///
/// Checks *all* placeholders before returning, collecting every missing name
/// so the caller gets a single actionable error.
///
/// # Errors
///
/// Returns [`RurlError::SubstitutionError`] listing every missing placeholder.
pub fn validate_template(template: &str, columns: &[String]) -> Result<()> {
    let placeholders = extract_placeholders(template);
    let missing: Vec<&str> = placeholders
        .iter()
        .filter(|p| !columns.contains(p))
        .map(String::as_str)
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(RurlError::SubstitutionError(format!(
            "template references unknown columns: [{}]; available: [{}]",
            missing.join(", "),
            columns.join(", ")
        )))
    }
}

/// Returns a reference to the data row that should be used for `request_index`.
///
/// Rows are cycled deterministically using modulo arithmetic so that sequential
/// request indices fan out across all rows and then wrap back to the start.
///
/// # Panics
///
/// Panics if `data_file` contains no rows (empty files are rejected earlier by
/// [`DataFile::from_path`], so this should never be reached in practice).
pub fn get_row_for_request(data_file: &DataFile, request_index: usize) -> &DataRow {
    &data_file.rows()[request_index % data_file.len()]
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn row(pairs: &[(&str, &str)]) -> DataRow {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn cols(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    // R003 — basic single-placeholder substitution
    #[test]
    fn test_substitute_basic() {
        let r = row(&[("user_id", "42")]);
        let out = substitute("GET /users/{{user_id}}", &r).unwrap();
        assert_eq!(out, "GET /users/42");
    }

    // R003, R004 — multiple placeholders in a URL
    #[test]
    fn test_substitute_multiple() {
        let r = row(&[("host", "api.example.com"), ("version", "v2"), ("id", "7")]);
        let out = substitute("https://{{host}}/{{version}}/items/{{id}}", &r).unwrap();
        assert_eq!(out, "https://api.example.com/v2/items/7");
    }

    // Passthrough when no placeholders are present
    #[test]
    fn test_substitute_no_placeholders() {
        let r = row(&[("x", "1")]);
        let out = substitute("https://example.com/static", &r).unwrap();
        assert_eq!(out, "https://example.com/static");
    }

    // R006 — missing column yields SubstitutionError
    #[test]
    fn test_substitute_missing_column() {
        let r = row(&[("name", "Alice")]);
        let err = substitute("Hello {{missing_col}}", &r).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing_col"), "expected col name in: {}", msg);
        assert!(msg.contains("name"), "expected available cols in: {}", msg);
    }

    // Validate: all columns present — should succeed
    #[test]
    fn test_validate_template_ok() {
        let result = validate_template(
            "{{user_id}} {{api_key}}",
            &cols(&["user_id", "api_key", "extra"]),
        );
        assert!(result.is_ok());
    }

    // R006 — validate detects missing column
    #[test]
    fn test_validate_template_missing() {
        let err = validate_template(
            "{{user_id}} {{no_such_col}}",
            &cols(&["user_id", "email"]),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("no_such_col"), "expected missing col in: {}", msg);
        assert!(msg.contains("user_id"), "expected available in: {}", msg);
    }

    // R004 — realistic URL path, header value, and JSON body substitution
    #[test]
    fn test_substitute_in_url_header_body() {
        let r = row(&[
            ("user_id", "99"),
            ("api_key", "secret-token"),
            ("payload", r#"{"name":"Alice"}"#),
        ]);

        // URL path
        let url = substitute("https://api.example.com/users/{{user_id}}/profile", &r).unwrap();
        assert_eq!(url, "https://api.example.com/users/99/profile");

        // Header value
        let header = substitute("Authorization: Bearer {{api_key}}", &r).unwrap();
        assert_eq!(header, "Authorization: Bearer secret-token");

        // JSON body
        let body = substitute(r#"{"id":{{user_id}},"data":{{payload}}}"#, &r).unwrap();
        assert_eq!(body, r#"{"id":99,"data":{"name":"Alice"}}"#);
    }

    // Whitespace-tolerant placeholders: {{ user_id }} should work
    #[test]
    fn test_substitute_whitespace_tolerant() {
        let r = row(&[("user_id", "42")]);
        let out = substitute("Hello {{ user_id }}, you are {{  user_id  }}", &r).unwrap();
        assert_eq!(out, "Hello 42, you are 42");
    }

    // ── Helpers for DataFile-based tests ──────────────────────────────────

    /// Creates a temporary CSV file, loads it as a DataFile, and removes the
    /// temp file.  The caller receives the loaded DataFile.
    fn make_csv_datafile(filename: &str, csv_content: &str) -> DataFile {
        use std::io::Write;
        let path = std::env::temp_dir().join(filename);
        let mut f = std::fs::File::create(&path).expect("create temp csv");
        write!(f, "{}", csv_content).unwrap();
        let df = DataFile::from_path(&path).expect("parse temp csv");
        let _ = std::fs::remove_file(&path);
        df
    }

    // R005 — basic cycling over 3 rows, requests 0-5
    #[test]
    fn test_get_row_cycling_basic() {
        let df = make_csv_datafile(
            "hurley_sub_cycle3.csv",
            "id\nrow0\nrow1\nrow2",
        );
        assert_eq!(df.len(), 3);
        assert_eq!(get_row_for_request(&df, 0).get("id").map(String::as_str), Some("row0"));
        assert_eq!(get_row_for_request(&df, 1).get("id").map(String::as_str), Some("row1"));
        assert_eq!(get_row_for_request(&df, 2).get("id").map(String::as_str), Some("row2"));
        // Wrap-around
        assert_eq!(get_row_for_request(&df, 3).get("id").map(String::as_str), Some("row0"));
        assert_eq!(get_row_for_request(&df, 4).get("id").map(String::as_str), Some("row1"));
        assert_eq!(get_row_for_request(&df, 5).get("id").map(String::as_str), Some("row2"));
    }

    // R005 — 100 rows: request 999 → 999 % 100 = 99 → row index 99
    #[test]
    fn test_get_row_cycling_large() {
        // Build CSV header + 100 rows (id = "row0" .. "row99")
        let mut csv = String::from("id\n");
        for i in 0..100usize {
            csv.push_str(&format!("row{}\n", i));
        }
        let df = make_csv_datafile("hurley_sub_cycle100.csv", &csv);
        assert_eq!(df.len(), 100);
        let target = get_row_for_request(&df, 999);
        // 999 % 100 == 99
        assert_eq!(target.get("id").map(String::as_str), Some("row99"));
    }

    // R005 — single row: every request index returns the same row
    #[test]
    fn test_get_row_single_row() {
        let df = make_csv_datafile("hurley_sub_cycle1.csv", "val\nonly");
        assert_eq!(df.len(), 1);
        for idx in [0, 1, 100, 9999] {
            assert_eq!(
                get_row_for_request(&df, idx).get("val").map(String::as_str),
                Some("only"),
                "request {} should return the sole row",
                idx
            );
        }
    }

    // `{{}}` has no \w+ inside — regex must NOT match it
    #[test]
    fn test_substitute_empty_placeholder() {
        let r = row(&[]);
        let out = substitute("before {{}} after", &r).unwrap();
        assert_eq!(out, "before {{}} after", "empty braces must be left intact");
    }

    // `{{{col}}}` — outer brace is literal, inner `{{col}}` is substituted
    #[test]
    fn test_substitute_nested_braces() {
        let r = row(&[("col", "value")]);
        let out = substitute("{{{col}}}", &r).unwrap();
        // Leading `{` stays, `{{col}}` → "value", trailing `}` stays
        assert_eq!(out, "{value}");
    }

    // Single-brace JSON `{"key": "value"}` must remain untouched
    #[test]
    fn test_substitute_json_body_no_false_match() {
        let r = row(&[]);
        let template = r#"{"key": "value", "num": 42}"#;
        let out = substitute(template, &r).unwrap();
        assert_eq!(out, template, "single-brace JSON must not be altered");
    }

    // Same placeholder appearing twice is replaced twice
    #[test]
    fn test_substitute_repeated_placeholder() {
        let r = row(&[("id", "42")]);
        let out = substitute("{{id}}-{{id}}", &r).unwrap();
        assert_eq!(out, "42-42");
    }

    // validate_template succeeds when the template contains no placeholders
    #[test]
    fn test_validate_template_no_placeholders() {
        let result = validate_template("https://example.com/static", &cols(&[]));
        assert!(result.is_ok(), "no-placeholder template must always pass");
    }

    // extract_placeholders returns each name only once, even when duplicated
    #[test]
    fn test_extract_placeholders_dedup() {
        let names = extract_placeholders("{{a}} {{b}} {{a}} {{b}} {{c}}");
        assert_eq!(names, vec!["a", "b", "c"], "duplicates must be deduplicated");
    }
}
