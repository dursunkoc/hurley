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

use super::datafile::DataRow;

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
}
