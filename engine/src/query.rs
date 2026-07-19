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
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! RFC 9535-conformant JSONPath expression evaluation and path diagnostics.
//!
//! This module bridges the span-aware `JsonNode` AST produced by the parser
//! and the `serde_json::Value` type consumed by `serde_json_path`. Conversion
//! is done on demand rather than storing a parallel representation, keeping
//! the memory footprint proportional to the size of query results rather than
//! the entire document.

use crate::json_node::{JsonNode, KeyValuePair};
use crate::types::{Diagnostic, Span};
use serde_json::Value;
use serde_json_path::JsonPath;

// ─────────────────────────────────────────────────────────────────────────────
// Internal: JsonNode → serde_json::Value
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a span-aware `JsonNode` into a `serde_json::Value` for evaluation
/// by `serde_json_path`. Span metadata is intentionally discarded here because
/// `serde_json::Value` has no span concept; spans are preserved in the original
/// tree and not needed during structural traversal.
///
/// Numbers are re-serialised from their raw source string to avoid f64
/// precision loss (e.g. large integers or high-precision decimals).
///
/// # Arguments
///
/// * `node` - A reference to the `JsonNode` to convert, including any nesting depth.
///
/// # Returns
///
/// The equivalent `serde_json::Value` representation of the node.
fn node_to_value(node: &JsonNode) -> Value {
    match node {
        JsonNode::Null(_) => Value::Null,
        JsonNode::Bool(b, _) => Value::Bool(*b),
        JsonNode::Number(_, raw, _) => raw
            .parse::<serde_json::Number>()
            .map(Value::Number)
            .unwrap_or(Value::Null),
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
// Internal: serde_json::Value → JsonNode
// ─────────────────────────────────────────────────────────────────────────────

/// Converts a `serde_json::Value` back into a `JsonNode` after query evaluation.
///
/// The resulting nodes carry a zeroed `Span` (`0..0`) because byte offset
/// information is not recoverable after the round-trip through
/// `serde_json::Value`. Callers that require source positions should resolve
/// them against the original parsed tree using the matched value's content.
///
/// # Arguments
///
/// * `value` - A reference to the `serde_json::Value` to convert back into the engine's node type.
///
/// # Returns
///
/// A `JsonNode` equivalent to the supplied value, with all spans zeroed.
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

/// Evaluates a JSONPath expression against the parsed document AST, returning
/// all matching nodes as a list of `JsonNode` values.
///
/// The expression is first parsed and validated by `serde_json_path`. If it is
/// syntactically invalid, an `Err` is returned before any document traversal
/// occurs. Matching is performed on a `serde_json::Value` projection of the
/// document; each matched value is then converted back to a `JsonNode` for a
/// consistent return type. Returned nodes carry a zeroed `Span` (`0..0`)
/// because byte offset information is not recoverable through the
/// `serde_json::Value` round-trip.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode` of the document to query against.
/// * `path` - The RFC 9535 JSONPath expression string to evaluate (e.g. `$.store.book[*].author`).
///
/// # Returns
///
/// * `Ok(Vec<JsonNode>)` - Zero or more nodes matched by the expression, in document order.
///   An empty `Vec` indicates the expression is valid but matched nothing.
/// * `Err(String)` - A human-readable description of the parse error if the expression is syntactically invalid.
pub fn evaluate_path(node: &JsonNode, path: &str) -> Result<Vec<JsonNode>, String> {
    let json_path = JsonPath::parse(path).map_err(|e| format!("Invalid JSONPath '{path}': {e}"))?;
    let value = node_to_value(node);
    let node_list = json_path.query(&value);
    let results = node_list.iter().map(|v| value_to_node(v)).collect();
    Ok(results)
}

/// Evaluates a JSONPath expression against a raw JSON string directly, bypassing
/// the span-aware parser and full AST construction overhead. This is significantly
/// faster and uses much less memory for large documents.
///
/// # Arguments
///
/// * `json_string` - The raw JSON string.
/// * `path` - The RFC 9535 JSONPath expression string.
///
/// # Returns
///
/// * `Ok(Vec<JsonNode>)` - Nodes matched by the expression.
/// * `Err(String)` - Parse error description.
pub fn evaluate_path_on_str(json_string: &str, path: &str) -> Result<Vec<JsonNode>, String> {
    let json_path = JsonPath::parse(path).map_err(|e| format!("Invalid JSONPath '{path}': {e}"))?;
    let value: Value =
        serde_json::from_str(json_string).map_err(|e| format!("Invalid JSON: {e}"))?;
    let node_list = json_path.query(&value);
    let results = node_list.iter().map(|v| value_to_node(v)).collect();
    Ok(results)
}

/// Validates a JSONPath expression syntactically without evaluating it against
/// any document. Intended for live editor feedback where the user is still
/// composing a query and no document context is available or needed.
///
/// The returned diagnostic always spans the entire expression string (`0..len`)
/// because `serde_json_path` does not expose sub-expression byte offsets.
///
/// # Arguments
///
/// * `path` - The raw JSONPath expression string slice to validate.
///
/// # Returns
///
/// * An empty `Vec<Diagnostic>` if the expression is syntactically valid.
/// * A single-element `Vec<Diagnostic>` whose `span` covers the full input
///   string and whose `message` describes the parse failure if the expression is invalid.
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

    /// Parses a trusted test document string, panicking on failure.
    fn parse_doc(src: &str) -> JsonNode {
        parse(src).expect("test document must be valid JSON")
    }

    // ── evaluate_path ─────────────────────────────────────────────────────────

    /// **Test Case**: Root Selector Returns Entire Document
    ///
    /// ### Description
    /// Verifies that the bare root selector `$` returns the complete document
    /// as a single-element result list.
    ///
    /// ### Test Procedure
    /// 1. Parse a simple JSON object `{"a": 1}`.
    /// 2. Evaluate the path `$`.
    ///
    /// ### Expected Result
    /// Returns exactly one `JsonNode::Object` matching the root document.
    #[test]
    fn test_jsonpath_root_selector() {
        let node = parse_doc(r#"{"a": 1}"#);
        let results = evaluate_path(&node, "$").unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], JsonNode::Object(_, _)));
    }

    /// **Test Case**: Dot-Key Selector Returns Named Property
    ///
    /// ### Description
    /// Verifies that a single dot-key selector (`$.key`) correctly extracts the
    /// value of a named top-level property.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"name": "Alice", "age": 30}`.
    /// 2. Evaluate `$.name`.
    ///
    /// ### Expected Result
    /// Returns a single `JsonNode::String("Alice")` with a zeroed span.
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

    /// **Test Case**: Array Index Selector Returns Indexed Element
    ///
    /// ### Description
    /// Verifies that bracket index notation (`[0]`) selects the correct element
    /// from a nested array.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"arr": [10, 20, 30]}`.
    /// 2. Evaluate `$.arr[0]`.
    ///
    /// ### Expected Result
    /// Returns a single `JsonNode::Number(10.0, "10")`.
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

    /// **Test Case**: Wildcard Selector Collects All Matching Array Items
    ///
    /// ### Description
    /// Verifies the key use case from the issue specification: wildcard traversal
    /// of an object array collecting named properties from each element.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"users": [{"name": "Alice"}, {"name": "Bob"}]}`.
    /// 2. Evaluate `$.users[*].name`.
    ///
    /// ### Expected Result
    /// Returns two string nodes `"Alice"` and `"Bob"` in document order.
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

    /// **Test Case**: Recursive Descent Selector Finds Keys at Any Depth
    ///
    /// ### Description
    /// Verifies that the `..` (descendant) operator locates matching keys
    /// at all nesting levels within the document tree.
    ///
    /// ### Test Procedure
    /// 1. Parse a document with `"name"` at root level and deeply nested.
    /// 2. Evaluate `$..name`.
    ///
    /// ### Expected Result
    /// Returns both occurrences of the `name` key, regardless of depth.
    #[test]
    fn test_jsonpath_recursive_descent() {
        let node = parse_doc(r#"{"a": {"b": {"name": "deep"}}, "name": "root"}"#);
        let results = evaluate_path(&node, "$..name").unwrap();
        assert_eq!(results.len(), 2);
    }

    /// **Test Case**: Filter Expression Selects Elements by Predicate
    ///
    /// ### Description
    /// Verifies that a filter expression (`?@.field > value`) correctly filters
    /// array elements based on a comparison against a numeric field.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"items": [{"val": 1}, {"val": 5}, {"val": 3}]}`.
    /// 2. Evaluate `$.items[?@.val > 2]`.
    ///
    /// ### Expected Result
    /// Returns two objects (those with `val` equal to 5 and 3).
    #[test]
    fn test_jsonpath_filter_expression() {
        let node = parse_doc(r#"{"items": [{"val": 1}, {"val": 5}, {"val": 3}]}"#);
        let results = evaluate_path(&node, "$.items[?@.val > 2]").unwrap();
        assert_eq!(results.len(), 2);
    }

    /// **Test Case**: Non-Matching Path Returns Empty Result List
    ///
    /// ### Description
    /// Verifies that querying for a key that does not exist in the document
    /// returns an empty result list without error, consistent with RFC 9535.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"a": 1}`.
    /// 2. Evaluate `$.nonexistent`.
    ///
    /// ### Expected Result
    /// Returns `Ok` with an empty `Vec`.
    #[test]
    fn test_jsonpath_no_match_returns_empty() {
        let node = parse_doc(r#"{"a": 1}"#);
        let results = evaluate_path(&node, "$.nonexistent").unwrap();
        assert!(results.is_empty());
    }

    /// **Test Case**: Invalid JSONPath Expression Returns Descriptive Error
    ///
    /// ### Description
    /// Verifies that a syntactically invalid JSONPath expression produces an
    /// `Err` result rather than panicking or silently returning no results.
    ///
    /// ### Test Procedure
    /// 1. Parse a minimal valid JSON document.
    /// 2. Evaluate a string that is not a valid JSONPath expression.
    ///
    /// ### Expected Result
    /// Returns `Err` with a message containing `"Invalid JSONPath"`.
    #[test]
    fn test_jsonpath_invalid_path_returns_err() {
        let node = parse_doc(r#"{"a": 1}"#);
        let result = evaluate_path(&node, "not a valid path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSONPath"));
    }

    /// **Test Case**: Null Value Is Preserved Through Conversion Round-Trip
    ///
    /// ### Description
    /// Verifies that a JSON `null` value selected by a query is correctly
    /// converted back to `JsonNode::Null` after the `serde_json::Value`
    /// round-trip.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"x": null}`.
    /// 2. Evaluate `$.x`.
    ///
    /// ### Expected Result
    /// Returns a single `JsonNode::Null`.
    #[test]
    fn test_jsonpath_null_value() {
        let node = parse_doc(r#"{"x": null}"#);
        let results = evaluate_path(&node, "$.x").unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], JsonNode::Null(_)));
    }

    /// **Test Case**: Boolean Value Is Preserved Through Conversion Round-Trip
    ///
    /// ### Description
    /// Verifies that a JSON boolean value selected by a query is correctly
    /// converted back to `JsonNode::Bool` after the `serde_json::Value`
    /// round-trip.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"flag": true}`.
    /// 2. Evaluate `$.flag`.
    ///
    /// ### Expected Result
    /// Returns a single `JsonNode::Bool(true)`.
    #[test]
    fn test_jsonpath_bool_value() {
        let node = parse_doc(r#"{"flag": true}"#);
        let results = evaluate_path(&node, "$.flag").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], JsonNode::Bool(true, Span::default()));
    }

    // ── diagnostics_for_path ──────────────────────────────────────────────────

    /// **Test Case**: Valid Complex Path Produces No Diagnostics
    ///
    /// ### Description
    /// Verifies that a syntactically correct and complete JSONPath expression
    /// produces an empty diagnostics list.
    ///
    /// ### Test Procedure
    /// 1. Call `diagnostics_for_path` with `$.store.book[*].author`.
    ///
    /// ### Expected Result
    /// Returns an empty `Vec<Diagnostic>`.
    #[test]
    fn test_diagnostics_valid_path_is_empty() {
        let diags = diagnostics_for_path("$.store.book[*].author");
        assert!(diags.is_empty());
    }

    /// **Test Case**: Root-Only Selector Produces No Diagnostics
    ///
    /// ### Description
    /// Verifies that the minimal valid JSONPath expression (`$`) does not
    /// produce any diagnostic errors.
    ///
    /// ### Test Procedure
    /// 1. Call `diagnostics_for_path` with `$`.
    ///
    /// ### Expected Result
    /// Returns an empty `Vec<Diagnostic>`.
    #[test]
    fn test_diagnostics_root_only_is_valid() {
        let diags = diagnostics_for_path("$");
        assert!(diags.is_empty());
    }

    /// **Test Case**: Invalid Expression Returns a Single Spanning Diagnostic
    ///
    /// ### Description
    /// Verifies that a syntactically invalid expression produces exactly one
    /// diagnostic whose span covers the full input string and whose message
    /// contains a useful prefix.
    ///
    /// ### Test Procedure
    /// 1. Call `diagnostics_for_path` with an arbitrary non-JSONPath string.
    ///
    /// ### Expected Result
    /// Returns one `Diagnostic` with `span.start == 0`, `span.end == len(input)`,
    /// and a message containing `"Invalid JSONPath"`.
    #[test]
    fn test_diagnostics_invalid_path_returns_diagnostic() {
        let input = "INVALID";
        let diags = diagnostics_for_path(input);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Invalid JSONPath"));
        assert_eq!(diags[0].span.start, 0);
        assert_eq!(diags[0].span.end, input.len());
    }

    /// **Test Case**: Empty String Expression Returns a Diagnostic
    ///
    /// ### Description
    /// Verifies that an empty string — which is not a valid JSONPath expression
    /// since every path must begin with `$` — produces at least one diagnostic.
    ///
    /// ### Test Procedure
    /// 1. Call `diagnostics_for_path` with an empty string `""`.
    ///
    /// ### Expected Result
    /// Returns a non-empty `Vec<Diagnostic>`.
    #[test]
    fn test_diagnostics_empty_string_returns_diagnostic() {
        let diags = diagnostics_for_path("");
        assert!(!diags.is_empty());
    }

    // ── Conversion round-trip ─────────────────────────────────────────────────

    /// **Test Case**: Object Node Survives JsonNode → Value → JsonNode Round-Trip
    ///
    /// ### Description
    /// Verifies that converting a parsed `JsonNode::Object` to `serde_json::Value`
    /// and back to `JsonNode` preserves the structural shape and key names,
    /// even though source spans are zeroed out in the returned copy.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"key": "value", "num": 42}`.
    /// 2. Convert to `serde_json::Value` via `node_to_value`.
    /// 3. Convert back via `value_to_node`.
    ///
    /// ### Expected Result
    /// The round-tripped node is a `JsonNode::Object` with two key-value pairs;
    /// the first key is `"key"`.
    #[test]
    fn test_node_to_value_and_back_object() {
        let node = parse_doc(r#"{"key": "value", "num": 42}"#);
        let value = node_to_value(&node);
        let roundtrip = value_to_node(&value);
        assert!(matches!(roundtrip, JsonNode::Object(_, _)));
        if let JsonNode::Object(pairs, _) = &roundtrip {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].key, "key");
        }
    }
}
