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

//! Structural schema and shape extraction for JSON documents.
//!
//! Provides token-bounded inspection of JSON documents, producing concise
//! schemas of key hierarchies, types, and array dimensions without exposing
//! massive data payloads to AI agents or logging pipelines.

use crate::json_node::JsonNode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Configuration options for structural schema inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaOptions {
    /// Maximum depth to recurse into nested objects and arrays.
    /// `None` indicates an unbounded traversal up to a safe system limit (32).
    pub max_depth: Option<usize>,
    /// Number of sample array items to inspect when inferring array element schema.
    pub sample_array_items: usize,
    /// Whether to capture sample primitive scalar values.
    pub include_samples: bool,
}

impl Default for SchemaOptions {
    fn default() -> Self {
        Self {
            max_depth: Some(10),
            sample_array_items: 5,
            include_samples: true,
        }
    }
}

/// A node in the extracted structural schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SchemaNode {
    /// A JSON null value.
    Null,
    /// A JSON boolean value.
    Bool {
        /// An optional sample boolean value observed in the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        sample: Option<bool>,
    },
    /// A JSON number.
    Number {
        /// An optional sample numeric value observed in the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        sample: Option<serde_json::Number>,
    },
    /// A JSON string.
    String {
        /// An optional sample string (truncated if long) observed in the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        sample: Option<String>,
    },
    /// A JSON array value, showing element count and inferred item schema.
    Array {
        /// Number of elements present in the inspected array.
        length: usize,
        /// Inferred structural schema of elements, if the array is non-empty.
        #[serde(skip_serializing_if = "Option::is_none")]
        item_schema: Option<Box<SchemaNode>>,
    },
    /// A JSON object value, showing key names and associated sub-schemas.
    Object {
        /// Map of object keys to their respective structural schemas.
        keys: BTreeMap<String, SchemaNode>,
    },
    /// Indicates that traversal halted because the configured depth limit was reached.
    DepthLimitReached,
}

impl SchemaNode {
    /// Generates a human- and LLM-friendly structural outline string.
    ///
    /// # Returns
    ///
    /// An indented outline describing types, array lengths, and sample values.
    pub fn to_summary_string(&self) -> String {
        let mut out = String::new();
        format_summary(self, 0, &mut out);
        out
    }

    /// Serializes the schema node as a JSON string.
    ///
    /// # Arguments
    ///
    /// * `pretty` - Whether to format with indentation and line breaks.
    ///
    /// # Returns
    ///
    /// A serialized JSON string, or a `serde_json::Error` on serialization failure.
    pub fn to_json_string(&self, pretty: bool) -> Result<String, serde_json::Error> {
        if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }
    }
}

fn format_summary(node: &SchemaNode, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match node {
        SchemaNode::Null => out.push_str("null\n"),
        SchemaNode::Bool { sample } => {
            if let Some(s) = sample {
                out.push_str(&format!("bool (sample: {s})\n"));
            } else {
                out.push_str("bool\n");
            }
        }
        SchemaNode::Number { sample } => {
            if let Some(s) = sample {
                out.push_str(&format!("number (sample: {s})\n"));
            } else {
                out.push_str("number\n");
            }
        }
        SchemaNode::String { sample } => {
            if let Some(s) = sample {
                out.push_str(&format!("string (sample: {:?})\n", s));
            } else {
                out.push_str("string\n");
            }
        }
        SchemaNode::Array {
            length,
            item_schema,
        } => {
            out.push_str(&format!("array[{length}]"));
            if let Some(item) = item_schema {
                out.push_str(" of ");
                format_summary(item, indent, out);
            } else {
                out.push('\n');
            }
        }
        SchemaNode::Object { keys } => {
            out.push_str("object\n");
            for (k, v) in keys {
                out.push_str(&format!("{pad}  .{k}: "));
                format_summary(v, indent + 1, out);
            }
        }
        SchemaNode::DepthLimitReached => out.push_str("<max depth reached>\n"),
    }
}

/// Inspects the structural schema of a parsed `JsonNode`.
///
/// # Arguments
///
/// * `node` - The root `JsonNode` to inspect.
/// * `options` - Traversal options (max depth, array samples, etc.).
///
/// # Returns
///
/// The extracted `SchemaNode` hierarchy.
pub fn inspect_schema(node: &JsonNode, options: &SchemaOptions) -> SchemaNode {
    inspect_impl(node, 0, options)
}

fn inspect_impl(node: &JsonNode, depth: usize, options: &SchemaOptions) -> SchemaNode {
    let max = options.max_depth.unwrap_or(32);
    if depth >= max {
        return SchemaNode::DepthLimitReached;
    }

    match node {
        JsonNode::Null(_) => SchemaNode::Null,
        JsonNode::Bool(b, _) => SchemaNode::Bool {
            sample: if options.include_samples {
                Some(*b)
            } else {
                None
            },
        },
        JsonNode::Number(num, raw, _) => SchemaNode::Number {
            sample: if options.include_samples {
                raw.parse::<serde_json::Number>()
                    .ok()
                    .or_else(|| serde_json::Number::from_f64(*num))
            } else {
                None
            },
        },
        JsonNode::String(s, _) => SchemaNode::String {
            sample: if options.include_samples {
                let sample_str = if s.chars().count() > 60 {
                    let prefix: String = s.chars().take(57).collect();
                    format!("{prefix}...")
                } else {
                    s.clone()
                };
                Some(sample_str)
            } else {
                None
            },
        },
        JsonNode::Array(items, _) => {
            let length = items.len();
            let samples: Vec<&JsonNode> = items.iter().take(options.sample_array_items).collect();

            let item_schema = if samples.is_empty() {
                None
            } else if samples.len() == 1 {
                Some(Box::new(inspect_impl(samples[0], depth + 1, options)))
            } else {
                Some(Box::new(merge_schemas(samples, depth + 1, options)))
            };

            SchemaNode::Array {
                length,
                item_schema,
            }
        }
        JsonNode::Object(pairs, _) => {
            let mut keys = BTreeMap::new();
            for kv in pairs {
                keys.insert(kv.key.clone(), inspect_impl(&kv.value, depth + 1, options));
            }
            SchemaNode::Object { keys }
        }
    }
}

fn merge_schemas(nodes: Vec<&JsonNode>, depth: usize, options: &SchemaOptions) -> SchemaNode {
    if nodes.is_empty() {
        return SchemaNode::Null;
    }

    let all_objects = nodes.iter().all(|n| matches!(n, JsonNode::Object(_, _)));
    if all_objects {
        let mut merged_keys: BTreeMap<String, Vec<&JsonNode>> = BTreeMap::new();
        for node in &nodes {
            if let JsonNode::Object(pairs, _) = node {
                for kv in pairs {
                    merged_keys
                        .entry(kv.key.clone())
                        .or_default()
                        .push(&kv.value);
                }
            }
        }
        let mut keys = BTreeMap::new();
        for (key, values) in merged_keys {
            keys.insert(key, merge_schemas(values, depth + 1, options));
        }
        return SchemaNode::Object { keys };
    }

    let first = nodes
        .iter()
        .find(|n| !matches!(n, JsonNode::Null(_)))
        .unwrap_or(&nodes[0]);
    inspect_impl(first, depth, options)
}

/// Inspects the structural schema directly from a raw JSON string.
///
/// # Arguments
///
/// * `json_str` - The JSON text to parse and inspect.
/// * `options` - Inspection options.
///
/// # Returns
///
/// * `Ok(SchemaNode)` on successful inspection.
/// * `Err(String)` if the JSON syntax is invalid.
pub fn inspect_schema_from_str(
    json_str: &str,
    options: &SchemaOptions,
) -> Result<SchemaNode, String> {
    let node = crate::parser::parse(json_str)
        .map_err(|diags| format!("Failed to parse JSON: {:?}", diags))?;
    Ok(inspect_schema(&node, options))
}

/// Summarizes the schema of a raw JSON string into a compact outline.
///
/// # Arguments
///
/// * `json_str` - The JSON text to inspect.
/// * `options` - Inspection options.
///
/// # Returns
///
/// * `Ok(String)` formatted outline.
/// * `Err(String)` if parsing fails.
pub fn summarize_schema_from_str(
    json_str: &str,
    options: &SchemaOptions,
) -> Result<String, String> {
    let schema = inspect_schema_from_str(json_str, options)?;
    Ok(schema.to_summary_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Test Case**: Inspect Primitive JSON Roots
    ///
    /// ### Description
    /// Verifies that inspecting primitive scalar values produces the matching `SchemaNode`
    /// variants with samples attached.
    ///
    /// ### Test Procedure
    /// 1. Inspect `"42"`, `"\"hello\""`, `"true"`, and `"null"`.
    ///
    /// ### Expected Result
    /// Result matches `Number`, `String`, `Bool`, and `Null` respectively.
    #[test]
    fn test_inspect_primitives() {
        let opts = SchemaOptions::default();
        let res = inspect_schema_from_str("42", &opts).unwrap();
        assert_eq!(
            res,
            SchemaNode::Number {
                sample: Some(serde_json::Number::from(42))
            }
        );

        let res = inspect_schema_from_str("\"hello\"", &opts).unwrap();
        assert_eq!(
            res,
            SchemaNode::String {
                sample: Some("hello".to_string())
            }
        );

        let res = inspect_schema_from_str("true", &opts).unwrap();
        assert_eq!(res, SchemaNode::Bool { sample: Some(true) });

        let res = inspect_schema_from_str("null", &opts).unwrap();
        assert_eq!(res, SchemaNode::Null);
    }

    /// **Test Case**: Inspect Object and Array Structure
    ///
    /// ### Description
    /// Verifies hierarchical object and array schema extraction and summary generation.
    ///
    /// ### Test Procedure
    /// 1. Inspect a JSON object containing nested primitives, arrays, and objects.
    /// 2. Generate summary outline string.
    ///
    /// ### Expected Result
    /// Schema correctly extracts all keys, nested array item schema, and summary matches.
    #[test]
    fn test_inspect_object_and_array() {
        let json = r#"{
            "id": 1,
            "name": "Widget",
            "tags": ["alpha", "beta"],
            "details": {
                "in_stock": true
            }
        }"#;

        let opts = SchemaOptions::default();
        let schema = inspect_schema_from_str(json, &opts).unwrap();

        if let SchemaNode::Object { keys } = &schema {
            assert!(keys.contains_key("id"));
            assert!(keys.contains_key("name"));
            assert!(keys.contains_key("tags"));
            assert!(keys.contains_key("details"));

            if let Some(SchemaNode::Array {
                length,
                item_schema,
            }) = keys.get("tags")
            {
                assert_eq!(*length, 2);
                assert_eq!(
                    item_schema.as_deref(),
                    Some(&SchemaNode::String {
                        sample: Some("alpha".to_string())
                    })
                );
            } else {
                panic!("Expected tags to be an array");
            }
        } else {
            panic!("Expected root to be an object");
        }

        let summary = schema.to_summary_string();
        assert!(summary.contains(".id: number"));
        assert!(summary.contains(".name: string"));
        assert!(summary.contains(".tags: array[2] of string"));
        assert!(summary.contains(".details: object"));
        assert!(summary.contains(".in_stock: bool"));
    }

    /// **Test Case**: Traversal Respects Configured Maximum Depth
    ///
    /// ### Description
    /// Verifies that recursing beyond `max_depth` replaces deeper sub-trees with `DepthLimitReached`.
    ///
    /// ### Test Procedure
    /// 1. Inspect nested object `{"a": {"b": {"c": 1}}}` with `max_depth: Some(2)`.
    ///
    /// ### Expected Result
    /// Key `"b"` contains `SchemaNode::DepthLimitReached`.
    #[test]
    fn test_inspect_max_depth() {
        let json = r#"{"a": {"b": {"c": 1}}}"#;
        let opts = SchemaOptions {
            max_depth: Some(2),
            ..Default::default()
        };
        let schema = inspect_schema_from_str(json, &opts).unwrap();
        if let SchemaNode::Object { keys } = &schema {
            if let Some(SchemaNode::Object { keys: inner }) = keys.get("a") {
                assert_eq!(inner.get("b"), Some(&SchemaNode::DepthLimitReached));
            } else {
                panic!("Expected 'a' to be an object");
            }
        } else {
            panic!("Expected root to be an object");
        }
    }

    /// **Test Case**: Maximum Depth Set to Zero
    ///
    /// ### Description
    /// Verifies that configuring `max_depth: Some(0)` halts traversal immediately at root.
    ///
    /// ### Test Procedure
    /// 1. Inspect `{"a": 1}` with `max_depth: Some(0)`.
    ///
    /// ### Expected Result
    /// Root node is `SchemaNode::DepthLimitReached`.
    #[test]
    fn test_inspect_max_depth_zero() {
        let json = r#"{"a": 1}"#;
        let opts = SchemaOptions {
            max_depth: Some(0),
            ..Default::default()
        };
        let schema = inspect_schema_from_str(json, &opts).unwrap();
        assert_eq!(schema, SchemaNode::DepthLimitReached);
        assert_eq!(schema.to_summary_string(), "<max depth reached>\n");
    }

    /// **Test Case**: Heterogeneous Array Merges Object Keys
    ///
    /// ### Description
    /// Verifies that arrays containing objects with different schemas merge all observed keys.
    ///
    /// ### Test Procedure
    /// 1. Inspect an array with two objects containing distinct and overlapping keys.
    ///
    /// ### Expected Result
    /// Inferred item schema contains all three union keys: `id`, `name`, and `email`.
    #[test]
    fn test_inspect_heterogeneous_array_merges_object_keys() {
        let json = r#"[
            {"id": 1, "name": "A"},
            {"id": 2, "email": "b@example.com"}
        ]"#;
        let opts = SchemaOptions::default();
        let schema = inspect_schema_from_str(json, &opts).unwrap();
        if let SchemaNode::Array {
            length,
            item_schema,
        } = &schema
        {
            assert_eq!(*length, 2);
            if let Some(box_item) = item_schema {
                if let SchemaNode::Object { keys } = box_item.as_ref() {
                    assert!(keys.contains_key("id"));
                    assert!(keys.contains_key("name"));
                    assert!(keys.contains_key("email"));
                } else {
                    panic!("Expected merged object item schema");
                }
            } else {
                panic!("Expected item schema");
            }
        } else {
            panic!("Expected array root");
        }
    }

    /// **Test Case**: Empty Documents, Objects, and Arrays
    ///
    /// ### Description
    /// Verifies edge cases for empty documents, empty objects, and empty arrays.
    ///
    /// ### Test Procedure
    /// 1. Inspect empty string `""`.
    /// 2. Inspect empty object `"{}"`.
    /// 3. Inspect empty array `"[]"`.
    ///
    /// ### Expected Result
    /// Empty string fails with a parse error; empty object and array return correct empty structures.
    #[test]
    fn test_inspect_empty_cases() {
        let opts = SchemaOptions::default();

        // Empty string
        let res = inspect_schema_from_str("", &opts);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Failed to parse JSON"));

        // Empty object
        let res = inspect_schema_from_str("{}", &opts).unwrap();
        assert_eq!(
            res,
            SchemaNode::Object {
                keys: BTreeMap::new()
            }
        );
        assert_eq!(res.to_summary_string(), "object\n");

        // Empty array
        let res = inspect_schema_from_str("[]", &opts).unwrap();
        assert_eq!(
            res,
            SchemaNode::Array {
                length: 0,
                item_schema: None,
            }
        );
        assert_eq!(res.to_summary_string(), "array[0]\n");
    }

    /// **Test Case**: Multibyte UTF-8 String Sample Truncation
    ///
    /// ### Description
    /// Verifies that strings exceeding 60 characters with multi-byte unicode sequences
    /// are truncated safely without slicing through code point byte boundaries.
    ///
    /// ### Test Procedure
    /// 1. Inspect a string containing 70 multi-byte Arabic / unicode characters.
    ///
    /// ### Expected Result
    /// Truncation succeeds without panicking, leaving exactly 57 characters plus ellipsis.
    #[test]
    fn test_inspect_long_multibyte_string_truncation() {
        let opts = SchemaOptions::default();
        // 70 Arabic characters (2 bytes each)
        let arabic = "مرحبا بكم في عالم البرمجة السريع والممتع مرحبا بكم في عالم البرمجة السريع";
        let json = format!("\"{}\"", arabic);
        let schema = inspect_schema_from_str(&json, &opts).unwrap();

        if let SchemaNode::String {
            sample: Some(sample),
        } = schema
        {
            assert!(sample.ends_with("..."));
            assert_eq!(sample.chars().count(), 60);
        } else {
            panic!("Expected string schema node with sample");
        }
    }

    /// **Test Case**: Schema Inspection with Samples Disabled
    ///
    /// ### Description
    /// Verifies that `include_samples: false` suppresses scalar samples from `SchemaNode`.
    ///
    /// ### Test Procedure
    /// 1. Inspect `{"active": true, "count": 100, "label": "test"}` with `include_samples: false`.
    ///
    /// ### Expected Result
    /// All scalar samples are `None`, and summary string renders type names without sample values.
    #[test]
    fn test_inspect_samples_disabled() {
        let opts = SchemaOptions {
            include_samples: false,
            ..Default::default()
        };
        let json = r#"{"active": true, "count": 100, "label": "test"}"#;
        let schema = inspect_schema_from_str(json, &opts).unwrap();

        if let SchemaNode::Object { keys } = &schema {
            assert_eq!(keys.get("active"), Some(&SchemaNode::Bool { sample: None }));
            assert_eq!(
                keys.get("count"),
                Some(&SchemaNode::Number { sample: None })
            );
            assert_eq!(
                keys.get("label"),
                Some(&SchemaNode::String { sample: None })
            );
        } else {
            panic!("Expected object");
        }

        let summary = schema.to_summary_string();
        assert!(!summary.contains("sample:"));
        assert!(summary.contains(".active: bool"));
        assert!(summary.contains(".count: number"));
        assert!(summary.contains(".label: string"));
    }

    /// **Test Case**: JSON Serialization of SchemaNode
    ///
    /// ### Description
    /// Verifies compact and pretty JSON serialization of `SchemaNode`.
    ///
    /// ### Test Procedure
    /// 1. Inspect a simple JSON document.
    /// 2. Serialize schema to compact JSON and pretty JSON via `to_json_string`.
    ///
    /// ### Expected Result
    /// Both serialize successfully; deserialization yields matching schema structure.
    #[test]
    fn test_schema_to_json_string() {
        let opts = SchemaOptions::default();
        let schema = inspect_schema_from_str(r#"{"key": "value"}"#, &opts).unwrap();

        let compact = schema.to_json_string(false).unwrap();
        assert!(compact.contains("\"type\":\"object\""));
        assert!(!compact.contains('\n'));

        let pretty = schema.to_json_string(true).unwrap();
        assert!(pretty.contains('\n'));

        let roundtrip: SchemaNode = serde_json::from_str(&compact).unwrap();
        assert_eq!(roundtrip, schema);
    }

    /// **Test Case**: Summarize Schema From String Helper
    ///
    /// ### Description
    /// Verifies that `summarize_schema_from_str` generates the formatted outline string or error.
    ///
    /// ### Test Procedure
    /// 1. Call `summarize_schema_from_str` on valid JSON and invalid JSON.
    ///
    /// ### Expected Result
    /// Valid JSON returns outline string; invalid JSON returns descriptive error.
    #[test]
    fn test_summarize_schema_from_str_helper() {
        let opts = SchemaOptions::default();
        let summary = summarize_schema_from_str(r#"[1, 2, 3]"#, &opts).unwrap();
        assert!(summary.contains("array[3] of number"));

        let err = summarize_schema_from_str("INVALID", &opts);
        assert!(err.is_err());
    }

    /// **Test Case**: Lossless Integer Precision in Numeric Schema Samples
    ///
    /// ### Description
    /// Verifies that integer samples larger than 2^53 (e.g. 9007199254740993) preserve exact
    /// precision from the raw lexeme rather than losing precision via f64.
    ///
    /// ### Expected Result
    /// Sample retains exact value 9007199254740993.
    #[test]
    fn test_inspect_large_integer_lossless() {
        let opts = SchemaOptions::default();
        let json = r#"{"big": 9007199254740993}"#;
        let schema = inspect_schema_from_str(json, &opts).unwrap();

        if let SchemaNode::Object { keys } = &schema {
            let num_node = keys.get("big").unwrap();
            if let SchemaNode::Number { sample } = num_node {
                let sample_num = sample.as_ref().unwrap();
                assert_eq!(sample_num.to_string(), "9007199254740993");
            } else {
                panic!("Expected number node");
            }
        } else {
            panic!("Expected object");
        }

        let summary = schema.to_summary_string();
        assert!(summary.contains("sample: 9007199254740993"));
        assert!(!summary.contains("9007199254740992"));
    }

    /// **Test Case**: Array Inspection With sample_array_items Zero
    ///
    /// ### Description
    /// Verifies that when `sample_array_items` is explicitly 0, array sampling is disabled
    /// and `item_schema` is None, while the array length is still reported.
    ///
    /// ### Expected Result
    /// `length` is 3, `item_schema` is None.
    #[test]
    fn test_inspect_array_sample_zero() {
        let opts = SchemaOptions {
            sample_array_items: 0,
            ..Default::default()
        };
        let json = r#"[1, 2, 3]"#;
        let schema = inspect_schema_from_str(json, &opts).unwrap();
        assert_eq!(
            schema,
            SchemaNode::Array {
                length: 3,
                item_schema: None,
            }
        );

        let summary = schema.to_summary_string();
        assert_eq!(summary.trim(), "array[3]");
    }
}
