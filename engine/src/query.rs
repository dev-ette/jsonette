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

/// Configuration options for bounded JSONPath queries.
///
/// Allows bounding query execution across matched count (`limit`),
/// pagination offset (`offset`), and serialized memory/byte footprint (`max_bytes`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryOptions {
    /// Maximum number of matched nodes to return.
    pub limit: Option<usize>,
    /// Number of matched nodes to skip before collecting.
    pub offset: Option<usize>,
    /// Maximum serialized byte budget across collected matches before truncation.
    pub max_bytes: Option<usize>,
}

/// The result of evaluating a JSONPath query under `QueryOptions` constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundedQueryResult {
    /// Matched nodes after applying offset, limit, and byte constraints.
    pub matches: Vec<JsonNode>,
    /// Total count of matching nodes found across the entire document before applying limits.
    pub total_matches: usize,
    /// Number of matches omitted due to offset, limit, or max_bytes.
    pub omitted: usize,
    /// True if matches were truncated due to `limit` or `max_bytes`.
    pub is_truncated: bool,
}

fn collect_bounded_matches<'a, I>(
    nodes_iter: I,
    options: &QueryOptions,
) -> Result<BoundedQueryResult, String>
where
    I: Iterator<Item = &'a Value>,
{
    let all_nodes: Vec<&'a Value> = nodes_iter.collect();
    let total_matches = all_nodes.len();

    let offset = options.offset.unwrap_or(0);
    let limit = options.limit.unwrap_or(usize::MAX);

    let mut matches = Vec::new();
    let mut current_bytes = 0;
    let mut hit_byte_limit = false;

    for (idx, val) in all_nodes.into_iter().enumerate() {
        if idx < offset {
            continue;
        }
        if matches.len() >= limit {
            break;
        }

        let json_node = value_to_node(val);
        if let Some(max_b) = options.max_bytes {
            let byte_size = crate::formatter::minify(&json_node).len();
            if current_bytes + byte_size > max_b {
                hit_byte_limit = true;
                break;
            }
            current_bytes += byte_size;
        }

        matches.push(json_node);
    }

    let returned_count = matches.len();
    let omitted = total_matches.saturating_sub(returned_count);
    let is_truncated = hit_byte_limit
        || (omitted > 0
            && (options.limit.is_some()
                || options.offset.is_some()
                || options.max_bytes.is_some()));

    Ok(BoundedQueryResult {
        matches,
        total_matches,
        omitted,
        is_truncated,
    })
}

/// Evaluates a JSONPath expression with bounds against the parsed document AST.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode`.
/// * `path` - The RFC 9535 JSONPath expression.
/// * `options` - Bounded query options (limit, offset, max_bytes).
///
/// # Returns
///
/// * `Ok(BoundedQueryResult)` containing matched nodes and bounding statistics.
/// * `Err(String)` if the JSONPath syntax is invalid.
pub fn evaluate_path_bounded(
    node: &JsonNode,
    path: &str,
    options: &QueryOptions,
) -> Result<BoundedQueryResult, String> {
    let json_path = JsonPath::parse(path).map_err(|e| format!("Invalid JSONPath '{path}': {e}"))?;
    let value = node_to_value(node);
    let node_list = json_path.query(&value);
    collect_bounded_matches(node_list.iter().copied(), options)
}

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
    evaluate_path_bounded(node, path, &QueryOptions::default()).map(|r| r.matches)
}

/// Evaluates a JSONPath expression with bounds directly against a raw JSON string.
///
/// # Arguments
///
/// * `json_string` - The raw JSON string.
/// * `path` - The RFC 9535 JSONPath expression string.
/// * `options` - Bounded query options (limit, offset, max_bytes).
///
/// # Returns
///
/// * `Ok(BoundedQueryResult)` containing matched nodes and bounding statistics.
/// * `Err(String)` if the JSON syntax or JSONPath expression is invalid.
pub fn evaluate_path_on_str_bounded(
    json_string: &str,
    path: &str,
    options: &QueryOptions,
) -> Result<BoundedQueryResult, String> {
    let json_path = JsonPath::parse(path).map_err(|e| format!("Invalid JSONPath '{path}': {e}"))?;
    let value: Value =
        serde_json::from_str(json_string).map_err(|e| format!("Invalid JSON: {e}"))?;
    let node_list = json_path.query(&value);
    collect_bounded_matches(node_list.iter().copied(), options)
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
    evaluate_path_on_str_bounded(json_string, path, &QueryOptions::default()).map(|r| r.matches)
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

    // ── evaluate_path_bounded ─────────────────────────────────────────────────

    /// **Test Case**: Bounded Query Limit and Offset Pagination
    ///
    /// ### Description
    /// Verifies that `limit` and `offset` correctly paginate results and mark `is_truncated`.
    ///
    /// ### Test Procedure
    /// 1. Query an array of 10 numbers with `offset: Some(2)` and `limit: Some(3)`.
    ///
    /// ### Expected Result
    /// Elements at index 2, 3, 4 are returned, `total_matches` is 10, `omitted` is 7, and `is_truncated` is true.
    #[test]
    fn test_evaluate_path_bounded_limit_and_offset() {
        let json = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#;
        let opts = QueryOptions {
            limit: Some(3),
            offset: Some(2),
            max_bytes: None,
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 10);
        assert_eq!(res.matches.len(), 3);
        assert_eq!(res.omitted, 7);
        assert!(res.is_truncated);

        assert_eq!(
            res.matches[0],
            JsonNode::Number(3.0, "3".to_string(), Span::default())
        );
        assert_eq!(
            res.matches[1],
            JsonNode::Number(4.0, "4".to_string(), Span::default())
        );
        assert_eq!(
            res.matches[2],
            JsonNode::Number(5.0, "5".to_string(), Span::default())
        );
    }

    /// **Test Case**: Bounded Query Max Bytes Enforcement
    ///
    /// ### Description
    /// Verifies that `max_bytes` stops collecting matches once the serialized budget is exceeded.
    ///
    /// ### Test Procedure
    /// 1. Query an array of 3 strings with `max_bytes: Some(15)`.
    ///
    /// ### Expected Result
    /// Only the first string fits within budget; `omitted` is 2, and `is_truncated` is true.
    #[test]
    fn test_evaluate_path_bounded_max_bytes() {
        let json = r#"["short", "a very long string that takes quite a few bytes to serialize", "another"]"#;
        let opts = QueryOptions {
            limit: None,
            offset: None,
            max_bytes: Some(15),
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 3);
        assert_eq!(res.matches.len(), 1);
        assert_eq!(res.omitted, 2);
        assert!(res.is_truncated);
    }

    /// **Test Case**: Bounded Query Max Bytes Smaller Than First Match
    ///
    /// ### Description
    /// Verifies that if `max_bytes` is 0 or smaller than the first serialized match,
    /// no matches are returned, `omitted` equals `total_matches`, and `is_truncated` is true.
    ///
    /// ### Expected Result
    /// Zero matches returned, contract strictly enforced.
    #[test]
    fn test_evaluate_path_bounded_max_bytes_smaller_than_first_match() {
        let json = r#"["first_long_element", "second"]"#;
        let opts = QueryOptions {
            limit: None,
            offset: None,
            max_bytes: Some(5), // "first_long_element" is 20 bytes minified
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 2);
        assert_eq!(res.matches.len(), 0);
        assert_eq!(res.omitted, 2);
        assert!(res.is_truncated);

        let opts_zero = QueryOptions {
            limit: None,
            offset: None,
            max_bytes: Some(0),
        };
        let res_zero = evaluate_path_on_str_bounded(json, "$[*]", &opts_zero).unwrap();
        assert_eq!(res_zero.total_matches, 2);
        assert_eq!(res_zero.matches.len(), 0);
        assert_eq!(res_zero.omitted, 2);
        assert!(res_zero.is_truncated);
    }

    /// **Test Case**: Offset Past Result Length
    ///
    /// ### Description
    /// Verifies behavior when `offset` exceeds the total count of matches.
    ///
    /// ### Test Procedure
    /// 1. Query an array of 3 numbers with `offset: Some(10)`.
    ///
    /// ### Expected Result
    /// `matches` is empty, `total_matches` is 3, `omitted` is 3, and `is_truncated` is true.
    #[test]
    fn test_evaluate_path_bounded_offset_past_length() {
        let json = r#"[1, 2, 3]"#;
        let opts = QueryOptions {
            limit: Some(5),
            offset: Some(10),
            max_bytes: None,
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 3);
        assert!(res.matches.is_empty());
        assert_eq!(res.omitted, 3);
        assert!(res.is_truncated);
    }

    /// **Test Case**: Limit Set to Zero
    ///
    /// ### Description
    /// Verifies that setting `limit: Some(0)` collects zero matches and correctly sets truncation flags.
    ///
    /// ### Test Procedure
    /// 1. Query `[1, 2, 3]` with `limit: Some(0)`.
    ///
    /// ### Expected Result
    /// `matches` is empty, `total_matches` is 3, `omitted` is 3, and `is_truncated` is true.
    #[test]
    fn test_evaluate_path_bounded_limit_zero() {
        let json = r#"[1, 2, 3]"#;
        let opts = QueryOptions {
            limit: Some(0),
            offset: None,
            max_bytes: None,
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 3);
        assert!(res.matches.is_empty());
        assert_eq!(res.omitted, 3);
        assert!(res.is_truncated);
    }

    /// **Test Case**: Bounded Query on AST Node
    ///
    /// ### Description
    /// Verifies `evaluate_path_bounded` directly on a parsed `JsonNode` AST.
    ///
    /// ### Test Procedure
    /// 1. Parse `{"tags": ["rust", "wasm", "json"]}` into a `JsonNode`.
    /// 2. Evaluate `$.tags[*]` with `limit: Some(2)`.
    ///
    /// ### Expected Result
    /// Returns 2 matches, `total_matches` is 3, `omitted` is 1, `is_truncated` is true.
    #[test]
    fn test_evaluate_path_bounded_ast_node() {
        let doc = parse_doc(r#"{"tags": ["rust", "wasm", "json"]}"#);
        let opts = QueryOptions {
            limit: Some(2),
            offset: None,
            max_bytes: None,
        };
        let res = evaluate_path_bounded(&doc, "$.tags[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 3);
        assert_eq!(res.matches.len(), 2);
        assert_eq!(res.omitted, 1);
        assert!(res.is_truncated);
        assert_eq!(
            res.matches[0],
            JsonNode::String("rust".into(), Span::default())
        );
        assert_eq!(
            res.matches[1],
            JsonNode::String("wasm".into(), Span::default())
        );
    }

    /// **Test Case**: Primitive Roots and Invalid Document
    ///
    /// ### Description
    /// Verifies bounded queries on primitive roots and error handling on empty/invalid documents.
    ///
    /// ### Test Procedure
    /// 1. Query primitive root `42` with `$`.
    /// 2. Query empty string `""` with `$`.
    /// 3. Query valid document with invalid JSONPath syntax.
    ///
    /// ### Expected Result
    /// Primitive root query succeeds with 1 match; empty doc and invalid path return descriptive errors.
    #[test]
    fn test_evaluate_path_bounded_edge_cases() {
        let opts = QueryOptions::default();
        let res = evaluate_path_on_str_bounded("42", "$", &opts).unwrap();
        assert_eq!(res.total_matches, 1);
        assert_eq!(res.matches.len(), 1);
        assert_eq!(res.omitted, 0);
        assert!(!res.is_truncated);
        assert_eq!(
            res.matches[0],
            JsonNode::Number(42.0, "42".into(), Span::default())
        );

        // Empty string -> parse error
        let err_doc = evaluate_path_on_str_bounded("", "$", &opts);
        assert!(err_doc.is_err());
        assert!(err_doc.unwrap_err().contains("Invalid JSON"));

        // Invalid JSONPath syntax -> path error
        let err_path = evaluate_path_on_str_bounded("42", "INVALID_PATH", &opts);
        assert!(err_path.is_err());
        assert!(err_path.unwrap_err().contains("Invalid JSONPath"));
    }

    /// **Test Case**: Byte Budget Guarantees First Item Included
    ///
    /// ### Description
    /// Verifies that even if `max_bytes` is smaller than the first item, at least
    /// one item is yielded before bounding terminates collection.
    ///
    /// ### Test Procedure
    /// 1. Query `["hello_world_long_string", "second_item"]` with `max_bytes: Some(1)`.
    /// **Test Case**: First Item Exceeding Byte Budget Is Not Returned
    ///
    /// ### Description
    /// Verifies that if the first match exceeds `max_bytes`, it is excluded to honor the budget.
    ///
    /// ### Expected Result
    /// `matches.len() == 0`, `omitted == 2`, `is_truncated == true`.
    #[test]
    fn test_evaluate_path_bounded_first_item_byte_budget() {
        let json = r#"["hello_world_long_string", "second_item"]"#;
        let opts = QueryOptions {
            limit: None,
            offset: None,
            max_bytes: Some(1),
        };
        let res = evaluate_path_on_str_bounded(json, "$[*]", &opts).unwrap();
        assert_eq!(res.total_matches, 2);
        assert_eq!(res.matches.len(), 0);
        assert_eq!(res.omitted, 2);
        assert!(res.is_truncated);
    }
}
