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

//! Schema-based Dummy Data JSON Generator.
//!
//! Evaluates a template `JsonNode` schema and generates a new `JsonNode`
//! containing dummy data according to the schema rules.

use crate::json_node::{JsonNode, KeyValuePair};
use crate::types::{Diagnostic, Span};
use std::collections::HashMap;

/// State for stateful generators like auto-incrementing integers.
#[derive(Default)]
struct GeneratorState {
    variables: HashMap<String, f64>,
}

/// Options for the generation process.
pub struct GeneratorOptions {
    /// If provided, the generator will wrap the schema in an array and repeat
    /// it until the approximate byte size is reached.
    pub target_size_bytes: Option<usize>,
    /// If provided, the generator will wrap the schema in an array and repeat
    /// it exactly `target_count` times.
    pub target_count: Option<usize>,
}

/// Evaluates a schema and returns the generated dummy data.
/// Returns a list of diagnostics if the schema contains structural errors.
///
/// # Arguments
///
/// * `schema` - A reference to the `JsonNode` representing the generation template.
/// * `options` - A reference to `GeneratorOptions` controlling target size or count.
///
/// # Returns
///
/// A `Result` containing the generated `JsonNode` AST on success, or a `Vec<Diagnostic>`
/// representing all schema evaluation errors.
pub fn generate_from_schema(
    schema: &JsonNode,
    options: &GeneratorOptions,
) -> Result<JsonNode, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut state = GeneratorState::default();

    if let Some(target_size) = options.target_size_bytes {
        let mut items = Vec::new();
        let mut current_size = 0;

        while current_size < target_size {
            let item = evaluate_node(schema, &mut state, &mut diagnostics);
            current_size += estimate_size(&item);
            items.push(item);

            if items.len() > 1000000 {
                break;
            }
        }

        if !diagnostics.is_empty() {
            Err(diagnostics)
        } else {
            Ok(JsonNode::Array(items, Span::default()))
        }
    } else if let Some(target_count) = options.target_count {
        let mut items = Vec::with_capacity(target_count);
        for _ in 0..target_count {
            items.push(evaluate_node(schema, &mut state, &mut diagnostics));
        }

        if !diagnostics.is_empty() {
            Err(diagnostics)
        } else {
            Ok(JsonNode::Array(items, Span::default()))
        }
    } else {
        let result = evaluate_node(schema, &mut state, &mut diagnostics);
        if !diagnostics.is_empty() {
            Err(diagnostics)
        } else {
            Ok(result)
        }
    }
}

/// Recursively evaluates a schema node.
///
/// # Arguments
///
/// * `node` - The current `JsonNode` in the schema being evaluated.
/// * `state` - The mutable `GeneratorState` maintaining variables across evaluations.
/// * `diagnostics` - A mutable vector accumulating any structural or evaluation errors.
///
/// # Returns
///
/// The evaluated `JsonNode` instance with resolved generator instructions.
fn evaluate_node(
    node: &JsonNode,
    state: &mut GeneratorState,
    diagnostics: &mut Vec<Diagnostic>,
) -> JsonNode {
    match node {
        JsonNode::Object(pairs, span) => {
            // Check if this is a generator instruction
            if let Some(type_pair) = pairs.iter().find(|p| p.key == "@type") {
                #[allow(clippy::collapsible_if)]
                if let JsonNode::String(t, _) = &type_pair.value {
                    return evaluate_instruction(t, pairs, state, diagnostics, span.clone());
                }
            }

            // Otherwise, it's a regular nested object. Evaluate its children.
            let mut new_pairs = Vec::with_capacity(pairs.len());
            for pair in pairs {
                // Handle @@ escape hatch
                let key = if pair.key.starts_with("@@") {
                    pair.key[1..].to_string()
                } else {
                    pair.key.clone()
                };

                new_pairs.push(KeyValuePair {
                    key,
                    value: evaluate_node(&pair.value, state, diagnostics),
                });
            }
            JsonNode::Object(new_pairs, span.clone())
        }
        JsonNode::Array(items, span) => {
            let new_items = items
                .iter()
                .map(|item| evaluate_node(item, state, diagnostics))
                .collect();
            JsonNode::Array(new_items, span.clone())
        }
        // Primitives pass through unchanged
        _ => node.clone(),
    }
}

/// Evaluates a specific generator instruction based on its `@type`.
///
/// # Arguments
///
/// * `instruction_type` - The string identifying the instruction (e.g., `uuid`, `integer`).
/// * `pairs` - The key-value pairs of the object containing the instruction.
/// * `state` - The mutable `GeneratorState` maintaining variables across evaluations.
/// * `diagnostics` - A mutable vector accumulating any structural or evaluation errors.
/// * `span` - The byte span of the instruction object.
///
/// # Returns
///
/// The generated `JsonNode` value for this instruction.
fn evaluate_instruction(
    instruction_type: &str,
    pairs: &[KeyValuePair],
    state: &mut GeneratorState,
    diagnostics: &mut Vec<Diagnostic>,
    span: Span,
) -> JsonNode {
    match instruction_type {
        "uuid" => {
            // Simple deterministic UUID for M0 to avoid rand dependency, or use a basic LCG.
            // In a real app we'd use `uuid` or `rand`.
            let lcg = state.variables.entry("lcg".to_string()).or_insert(1.0);
            *lcg = (*lcg * 1664525.0 + 1013904223.0) % 4294967296.0;
            let val = *lcg as u32;
            JsonNode::String(
                format!("uuid-{:08x}-1234-5678-abcd-123456789012", val),
                span,
            )
        }
        "integer" => {
            let mut start = 0.0;
            let mut step = 1.0;
            for pair in pairs {
                #[allow(clippy::collapsible_if)]
                if pair.key == "@start" {
                    if let JsonNode::Number(n, _, _) = pair.value {
                        start = n;
                    }
                } else if pair.key == "@step" {
                    if let JsonNode::Number(n, _, _) = pair.value {
                        step = n;
                    }
                }
            }
            let state_key = format!("int_{}", span.start);
            let current = state.variables.entry(state_key).or_insert(start);
            let val = *current;
            *current += step;
            JsonNode::Number(val, val.to_string(), span)
        }
        "float" => {
            let mut min = 0.0;
            let mut max = 1.0;
            for pair in pairs {
                #[allow(clippy::collapsible_if)]
                if pair.key == "@min" {
                    if let JsonNode::Number(n, _, _) = pair.value {
                        min = n;
                    }
                } else if pair.key == "@max" {
                    if let JsonNode::Number(n, _, _) = pair.value {
                        max = n;
                    }
                }
            }
            let lcg = state.variables.entry("lcg".to_string()).or_insert(1.0);
            *lcg = (*lcg * 1664525.0 + 1013904223.0) % 4294967296.0;
            let val = min + (*lcg / 4294967296.0) * (max - min);
            JsonNode::Number(val, format!("{:.4}", val), span)
        }
        "bool" => {
            let lcg = state.variables.entry("lcg".to_string()).or_insert(1.0);
            *lcg = (*lcg * 1664525.0 + 1013904223.0) % 4294967296.0;
            JsonNode::Bool(*lcg > 2147483648.0, span)
        }
        "string" => {
            let mut template = "".to_string();
            let mut pool = Vec::new();
            let mut vars_schema = None;

            for pair in pairs {
                #[allow(clippy::collapsible_if)]
                if pair.key == "@template" {
                    if let JsonNode::String(s, _) = &pair.value {
                        template = s.clone();
                    }
                } else if pair.key == "@pool" {
                    if let JsonNode::Array(items, _) = &pair.value {
                        for item in items {
                            if let JsonNode::String(s, _) = item {
                                pool.push(s.clone());
                            }
                        }
                    }
                } else if pair.key == "@vars" {
                    vars_schema = Some(&pair.value);
                }
            }

            if !pool.is_empty() {
                let lcg = state.variables.entry("lcg".to_string()).or_insert(1.0);
                *lcg = (*lcg * 1664525.0 + 1013904223.0) % 4294967296.0;
                let idx = (*lcg as usize) % pool.len();
                JsonNode::String(pool[idx].clone(), span)
            } else if !template.is_empty() {
                let mut output = template;
                if let Some(JsonNode::Object(var_pairs, _)) = vars_schema {
                    for var_pair in var_pairs {
                        let var_val = evaluate_node(&var_pair.value, state, diagnostics);
                        let var_str = match var_val {
                            JsonNode::String(s, _) => s,
                            JsonNode::Number(_, raw, _) => raw,
                            JsonNode::Bool(b, _) => b.to_string(),
                            _ => "".to_string(),
                        };
                        let placeholder = format!("{{{}}}", var_pair.key);
                        output = output.replace(&placeholder, &var_str);
                    }
                }
                JsonNode::String(output, span)
            } else {
                JsonNode::String("dummy".to_string(), span)
            }
        }
        "array" => {
            let mut count = 1;
            let mut item_schema = &JsonNode::Null(span.clone());
            for pair in pairs {
                if pair.key == "@count" {
                    if let JsonNode::Number(n, _, _) = pair.value {
                        count = n as usize;
                    }
                } else if pair.key == "@item" {
                    item_schema = &pair.value;
                }
            }

            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(evaluate_node(item_schema, state, diagnostics));
            }
            JsonNode::Array(items, span)
        }
        _ => {
            diagnostics.push(Diagnostic {
                span: span.clone(),
                message: format!("Unknown generator instruction: {}", instruction_type),
            });
            JsonNode::Null(span)
        }
    }
}

/// Very rough byte size estimator for loop termination.
///
/// # Arguments
///
/// * `node` - The `JsonNode` to estimate the serialized size for.
///
/// # Returns
///
/// An `usize` representing the approximate byte size of the node.
fn estimate_size(node: &JsonNode) -> usize {
    match node {
        JsonNode::Null(_) => 4,
        JsonNode::Bool(_, _) => 4,
        JsonNode::Number(_, raw, _) => raw.len(),
        JsonNode::String(s, _) => s.len() + 2,
        JsonNode::Array(items, _) => {
            items.iter().map(estimate_size).sum::<usize>() + items.len() + 2
        }
        JsonNode::Object(pairs, _) => {
            pairs
                .iter()
                .map(|p| p.key.len() + 4 + estimate_size(&p.value))
                .sum::<usize>()
                + 2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    /// **Test Case**: Generator Evaluates Integer and UUID Instructions
    ///
    /// ### Description
    /// Verifies that the generator correctly processes an `@type: integer` and `@type: uuid` schema.
    ///
    /// ### Test Procedure
    /// 1. Provide a schema containing `integer` and `uuid` instructions.
    /// 2. Generate exactly 2 items.
    ///
    /// ### Expected Result
    /// The returned AST is an array of two objects with incrementing integers and string UUIDs.
    #[test]
    fn test_generator_evaluates_integer_and_uuid() {
        let schema_str = r#"{
            "id": { "@type": "uuid" },
            "index": { "@type": "integer", "@start": 5, "@step": 2 }
        }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: Some(2),
        };

        let result = generate_from_schema(&schema, &opts).unwrap();
        if let JsonNode::Array(items, _) = result {
            assert_eq!(items.len(), 2);
            if let JsonNode::Object(pairs, _) = &items[0] {
                let index_node = pairs.iter().find(|p| p.key == "index").unwrap();
                if let JsonNode::Number(val, _, _) = index_node.value {
                    assert_eq!(val, 5.0);
                } else {
                    panic!("Expected Number");
                }
            }
            if let JsonNode::Object(pairs, _) = &items[1] {
                let index_node = pairs.iter().find(|p| p.key == "index").unwrap();
                if let JsonNode::Number(val, _, _) = index_node.value {
                    assert_eq!(val, 7.0);
                } else {
                    panic!("Expected Number");
                }
            }
        } else {
            panic!("Expected array");
        }
    }

    /// **Test Case**: Generator Returns Diagnostics for Unknown Instructions
    ///
    /// ### Description
    /// Verifies that providing an invalid `@type` instruction captures a diagnostic.
    ///
    /// ### Test Procedure
    /// 1. Provide a schema with `@type: fake_type`.
    ///
    /// ### Expected Result
    /// The generator returns an `Err(Vec<Diagnostic>)`.
    #[test]
    fn test_generator_unknown_instruction_returns_diagnostic() {
        let schema_str = r#"{ "field": { "@type": "fake_type" } }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: Some(1),
        };

        let result = generate_from_schema(&schema, &opts);
        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Unknown generator instruction"));
    }

    /// **Test Case**: Generator Handles Escaped Keys
    ///
    /// ### Description
    /// Verifies that `@@type` is unescaped to `@type` correctly without triggering an instruction.
    ///
    /// ### Test Procedure
    /// 1. Provide a schema containing `@@type`.
    ///
    /// ### Expected Result
    /// The AST outputs a literal `@type` key.
    #[test]
    fn test_generator_escaped_keys() {
        let schema_str = r#"{ "@@type": "User" }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: None,
        };

        let result = generate_from_schema(&schema, &opts).unwrap();
        if let JsonNode::Object(pairs, _) = result {
            assert_eq!(pairs[0].key, "@type");
            assert!(matches!(pairs[0].value, JsonNode::String(_, _)));
        } else {
            panic!("Expected object");
        }
    }

    /// **Test Case**: Generator Float And Bool
    ///
    /// ### Description
    /// Validates generator float and bool functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_generator_float_and_bool`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_generator_float_and_bool() {
        let schema_str = r#"{
            "f": { "@type": "float", "@min": 10, "@max": 20 },
            "b": { "@type": "bool" }
        }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: None,
        };
        let result = generate_from_schema(&schema, &opts).unwrap();

        if let JsonNode::Object(pairs, _) = result {
            let f_node = pairs.iter().find(|p| p.key == "f").unwrap();
            let b_node = pairs.iter().find(|p| p.key == "b").unwrap();

            if let JsonNode::Number(val, _, _) = f_node.value {
                assert!(val >= 10.0 && val <= 20.0);
            } else {
                panic!("Expected Number for float");
            }

            assert!(matches!(b_node.value, JsonNode::Bool(_, _)));
        } else {
            panic!("Expected Object");
        }
    }

    /// **Test Case**: Generator String Pool And Template
    ///
    /// ### Description
    /// Validates generator string pool and template functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_generator_string_pool_and_template`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_generator_string_pool_and_template() {
        let schema_str = r#"{
            "p": { "@type": "string", "@pool": ["A", "B"] },
            "t": { 
                "@type": "string", 
                "@template": "Hello {name} {age} {flag}", 
                "@vars": { 
                    "name": "World",
                    "age": { "@type": "integer", "@start": 42, "@step": 0 },
                    "flag": { "@type": "bool" }
                }
            },
            "dummy": { "@type": "string" }
        }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: None,
        };
        let result = generate_from_schema(&schema, &opts).unwrap();

        if let JsonNode::Object(pairs, _) = result {
            let p_node = pairs.iter().find(|p| p.key == "p").unwrap();
            let t_node = pairs.iter().find(|p| p.key == "t").unwrap();
            let dummy_node = pairs.iter().find(|p| p.key == "dummy").unwrap();

            if let JsonNode::String(s, _) = &p_node.value {
                assert!(s == "A" || s == "B");
            } else {
                panic!("Expected String");
            }

            if let JsonNode::String(s, _) = &t_node.value {
                assert!(s.starts_with("Hello World 42"));
            } else {
                panic!("Expected String");
            }

            if let JsonNode::String(s, _) = &dummy_node.value {
                assert_eq!(s, "dummy");
            } else {
                panic!("Expected String");
            }
        } else {
            panic!("Expected Object");
        }
    }

    /// **Test Case**: Generator Array Instruction
    ///
    /// ### Description
    /// Validates generator array instruction functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_generator_array_instruction`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_generator_array_instruction() {
        let schema_str = r#"{
            "arr": { 
                "@type": "array", 
                "@count": 3, 
                "@item": { "@type": "integer", "@start": 1, "@step": 1 } 
            }
        }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: None,
            target_count: None,
        };
        let result = generate_from_schema(&schema, &opts).unwrap();

        if let JsonNode::Object(pairs, _) = result {
            let arr_node = &pairs[0].value;
            if let JsonNode::Array(items, _) = arr_node {
                assert_eq!(items.len(), 3);
            } else {
                panic!("Expected Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    /// **Test Case**: Generator Target Size
    ///
    /// ### Description
    /// Validates generator target size functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_generator_target_size`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_generator_target_size() {
        let schema_str = r#"{ "a": 1 }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: Some(100),
            target_count: None,
        };
        let result = generate_from_schema(&schema, &opts).unwrap();

        if let JsonNode::Array(items, _) = result {
            // estimate_size({"a": 1}) = 1 + 4 + 1(for 1) + 2 = 8 bytes
            // So to get 100 bytes, we need around 100 / 8 = 12 items
            assert!(items.len() >= 10);
        } else {
            panic!("Expected Array");
        }
    }

    /// **Test Case**: Generator Target Size With Error
    ///
    /// ### Description
    /// Validates generator target size with error functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_generator_target_size_with_error`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_generator_target_size_with_error() {
        let schema_str = r#"{ "a": { "@type": "unknown" } }"#;
        let schema = parse(schema_str).unwrap();
        let opts = GeneratorOptions {
            target_size_bytes: Some(100),
            target_count: None,
        };
        let result = generate_from_schema(&schema, &opts);

        assert!(result.is_err());
    }

    /// **Test Case**: Estimate Size All Nodes
    ///
    /// ### Description
    /// Validates estimate size all nodes functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_estimate_size_all_nodes`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_estimate_size_all_nodes() {
        use crate::types::Span;
        let span = Span::default();
        assert_eq!(estimate_size(&JsonNode::Null(span.clone())), 4);
        assert_eq!(estimate_size(&JsonNode::Bool(true, span.clone())), 4);
        assert_eq!(
            estimate_size(&JsonNode::Number(1.0, "1.0".to_string(), span.clone())),
            3
        );
        assert_eq!(
            estimate_size(&JsonNode::String("abc".to_string(), span.clone())),
            5
        );
        assert_eq!(estimate_size(&JsonNode::Array(vec![], span.clone())), 2);
    }
}
