/*
 * Copyright (c) 2026 DevEtte.
 *
 * This project is dual-licensed under both the MIT License and the
 * Apache License, Version 2.0 (the "License"). You may not use this
 * file except in compliance with one of these licenses.
 *
 * You may obtain a copy of the Licenses at:
 * - MIT: https://opensource.org
 * - Apache 2.0: http://apache.org
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the licenses.
 */

//! JSONPath expression evaluation and validation (RFC 9535).
//!
//! Uses `serde_json_path` for conformant JSONPath parsing and evaluation.
//! A bidirectional conversion layer bridges our span-aware `JsonNode` type
//! and the `serde_json::Value` type that `serde_json_path` operates on.

use crate::json_node::{JsonNode, KeyValuePair};
use crate::types::{Diagnostic, Span};
use serde_json::Value;
use serde_json_path::JsonPath;

// ─────────────────────────────────────────────────────────────────────────────
// Conversion: JsonNode → serde_json::Value
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a span-aware `JsonNode` into a `serde_json::Value` for use with
/// `serde_json_path`. Span information is discarded during this conversion.
fn node_to_value(node: &JsonNode) -> Value {
    match node {
        JsonNode::Null(_) => Value::Null,
        JsonNode::Bool(b, _) => Value::Bool(*b),
        JsonNode::Number(_, raw, _) => {
            // Re-parse from the original raw string to avoid f64 precision loss.
            if let Ok(n) = raw.parse::<serde_json::Number>() {
                Value::Number(n)
            } else {
                Value::Null
            }
        }
        JsonNode::String(s, _) => Value::String(s.clone()),
        JsonNode::Array(items, _) => Value::Array(items.iter().map(node_to_value).collect()),
        JsonNode::Object(pairs, _) => {
            let map = pairs
                .iter()
                .map(|kv| (kv.key.clone(), node_to_value(&kv.value)))
                .collect();
            Value::Object(map)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversion: serde_json::Value → JsonNode (span = 0..0, unknown at query time)
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a `serde_json::Value` back into a `JsonNode`.
///
/// The resulting node carries a zeroed `Span` because the original byte offsets
/// cannot be recovered after the round-trip through `serde_json::Value`.
/// Callers that need source spans should use the original parsed tree instead.
fn value_to_node(value: &Value) -> JsonNode {
    let span = Span::default();
    match value {
        Value::Null => JsonNode::Null(span),
        Value::Bool(b) => JsonNode::Bool(*b, span),
        Value::Number(n) => JsonNode::Number(n.as_f64().unwrap_or(0.0), n.to_string(), span),
        Value::String(s) => JsonNode::String(s.clone(), span),
        Value::Array(items) => JsonNode::Array(items.iter().map(value_to_node).collect(), span),
        Value::Object(map) => {
            let pairs = map
                .iter()
                .map(|(k, v)| KeyValuePair {
                    key: k.clone(),
                    value: value_to_node(v),
                })
                .collect();
            JsonNode::Object(pairs, span)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluates a JSONPath query against the AST, returning all matching nodes.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode` to query.
/// * `path` - The JSONPath expression string slice (e.g. `$.store.book[*].author`).
///
/// # Returns
///
/// * `Ok(Vec<JsonNode>)` - A list of JSON nodes that match the query. Each node
///   carries a zeroed `Span` because span data is lost during the `serde_json::Value`
///   round-trip; callers needing source positions should use the original parsed tree.
/// * `Err(String)` - An error message if the JSONPath expression is invalid.
pub fn evaluate_path(node: &JsonNode, path: &str) -> Result<Vec<JsonNode>, String> {
    let json_path = JsonPath::parse(path).map_err(|e| format!("Invalid JSONPath '{path}': {e}"))?;
    let value = node_to_value(node);
    let node_list = json_path.query(&value);
    let results = node_list.iter().map(|v| value_to_node(*v)).collect();
    Ok(results)
}

/// Validates the syntax of a JSONPath expression without evaluating it against a document.
///
/// # Arguments
///
/// * `path` - The JSONPath expression string slice to validate.
///
/// # Returns
///
/// A list of `Diagnostic` errors found in the path syntax. If the path is valid, returns
/// an empty `Vec`. The span covers the entire expression since `serde_json_path` does not
/// provide sub-expression byte offsets.
pub fn diagnostics_for_path(path: &str) -> Vec<Diagnostic> {
    match JsonPath::parse(path) {
        Ok(_) => vec![],
        Err(e) => vec![Diagnostic {
            span: Span {
                start: 0,
                end: path.len(),
            },
            message: format!("Invalid JSONPath: {e}"),
        }],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod jsonpath_tests {
    use super::*;
    use crate::parser::parse;

    fn parse_doc(src: &str) -> JsonNode {
        parse(src).expect("test document must be valid JSON")
    }

    // ── evaluate_path ────────────────────────────────────────────────────────

    #[test]
    fn test_jsonpath_root_selector() {
        let node = parse_doc(r#"{"a": 1}"#);
        let results = evaluate_path(&node, "$").unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], JsonNode::Object(_, _)));
    }

    #[test]
    fn test_jsonpath_dot_key() {
        let node = parse_doc(r#"{"name": "Alice", "age": 30}"#);
        let results = evaluate_path(&node, "$.name").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0],
            JsonNode::String("Alice".into(), Span::default())
        );
    }

    #[test]
    fn test_jsonpath_array_index() {
        let node = parse_doc(r#"{"arr": [10, 20, 30]}"#);
        let results = evaluate_path(&node, "$.arr[0]").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0],
            JsonNode::Number(10.0, "10".into(), Span::default())
        );
    }

    #[test]
    fn test_jsonpath_wildcard_array() {
        let node = parse_doc(r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#);
        let results = evaluate_path(&node, "$.users[*].name").unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(
            results[0],
            JsonNode::String("Alice".into(), Span::default())
        );
        assert_eq!(results[1], JsonNode::String("Bob".into(), Span::default()));
    }

    #[test]
    fn test_jsonpath_recursive_descent() {
        let node = parse_doc(r#"{"a": {"b": {"name": "deep"}}, "name": "root"}"#);
        let results = evaluate_path(&node, "$..name").unwrap();
        // Should find both "root" and "deep"
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_jsonpath_filter_expression() {
        let node = parse_doc(r#"{"items": [{"val": 1}, {"val": 5}, {"val": 3}]}"#);
        let results = evaluate_path(&node, "$.items[?@.val > 2]").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_jsonpath_no_match_returns_empty() {
        let node = parse_doc(r#"{"a": 1}"#);
        let results = evaluate_path(&node, "$.nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_jsonpath_invalid_path_returns_err() {
        let node = parse_doc(r#"{"a": 1}"#);
        let result = evaluate_path(&node, "not a valid path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSONPath"));
    }

    #[test]
    fn test_jsonpath_null_value() {
        let node = parse_doc(r#"{"x": null}"#);
        let results = evaluate_path(&node, "$.x").unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], JsonNode::Null(_)));
    }

    #[test]
    fn test_jsonpath_bool_value() {
        let node = parse_doc(r#"{"flag": true}"#);
        let results = evaluate_path(&node, "$.flag").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], JsonNode::Bool(true, Span::default()));
    }

    // ── diagnostics_for_path ─────────────────────────────────────────────────

    #[test]
    fn test_diagnostics_valid_path_is_empty() {
        let diags = diagnostics_for_path("$.store.book[*].author");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_root_only_is_valid() {
        let diags = diagnostics_for_path("$");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_diagnostics_invalid_path_returns_diagnostic() {
        let diags = diagnostics_for_path("INVALID");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Invalid JSONPath"));
        assert_eq!(diags[0].span.start, 0);
        assert_eq!(diags[0].span.end, "INVALID".len());
    }

    #[test]
    fn test_diagnostics_empty_string_returns_diagnostic() {
        let diags = diagnostics_for_path("");
        assert!(!diags.is_empty());
    }

    // ── conversion round-trip ─────────────────────────────────────────────────

    #[test]
    fn test_node_to_value_and_back_object() {
        let node = parse_doc(r#"{"key": "value", "num": 42}"#);
        let value = node_to_value(&node);
        let roundtrip = value_to_node(&value);
        // Both should be Object variants with matching structure
        assert!(matches!(roundtrip, JsonNode::Object(_, _)));
        if let JsonNode::Object(pairs, _) = &roundtrip {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].key, "key");
        }
    }
}
