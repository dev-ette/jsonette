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

//! Shell owns rendering only. All logic lives here.

#![allow(unused_variables)]

pub mod completion;
pub mod converter;
pub mod formatter;
pub mod generator;
pub mod json_node;
pub mod parser;
pub mod query;
pub mod settings;
pub mod types;

pub use json_node::{JsonNode, KeyValuePair};
pub use types::{
    AppSettings, CompletionItem, Diagnostic, FoldingStyle, FormatOptions, LineEnding, LintOptions,
    ParserOptions, Severity, Span,
};

// Re-export public API functions
pub use completion::completions_at;
pub use formatter::{format, minify};
pub use parser::{diagnostics, parse, tolerant_parse};
pub use query::{diagnostics_for_path, evaluate_path, evaluate_path_on_str};
pub use settings::{Settings, get_settings, set_in_memory_settings, update_settings};

uniffi::setup_scaffolding!();

/// A trivial ping-pong function to verify the UniFFI FFI bridge is working end-to-end.
///
/// # Arguments
///
/// * `input` - A string parameter sent from the calling shell (Swift/Kotlin).
///
/// # Returns
///
/// A formatted string response showing success.
#[uniffi::export]
pub fn ping(input: String) -> String {
    format!("pong: {}", input)
}

/// A UniFFI-compatible diagnostic record containing error boundaries and a message.
#[derive(uniffi::Record)]
pub struct FFIDiagnostic {
    /// The starting byte offset in the string (inclusive).
    pub start: u64,
    /// The ending byte offset in the string (exclusive).
    pub end: u64,
    /// A human-readable message describing the syntax error.
    pub message: String,
}

/// Checks the syntax of a given JSON string and returns a list of diagnostics.
///
/// # Arguments
///
/// * `input` - The JSON string to validate.
///
/// # Returns
///
/// A `Vec<FFIDiagnostic>` representing all structural or syntax errors encountered.
/// If the input is valid, returns an empty vector.
#[uniffi::export]
pub fn check_syntax(input: String) -> Vec<FFIDiagnostic> {
    diagnostics(&input)
        .into_iter()
        .map(|d| FFIDiagnostic {
            start: d.span.start as u64,
            end: d.span.end as u64,
            message: d.message,
        })
        .collect()
}

/// Formats a given JSON string using the engine's parser and formatter.
/// If the JSON is invalid, it returns the original string unmodified.
#[uniffi::export]
pub fn format_json(input: String) -> String {
    match parse(&input) {
        Ok(node) => format(&node),
        Err(_) => input,
    }
}

/// Minifies a given JSON string using the engine's parser and formatter.
/// If the JSON is invalid, it returns the original string unmodified.
#[uniffi::export]
pub fn minify_json(input: String) -> String {
    match parse(&input) {
        Ok(node) => minify(&node),
        Err(_) => input,
    }
}

/// A UniFFI-compatible result containing either the evaluated output or an error message.
#[derive(uniffi::Record)]
pub struct FFIQueryResult {
    pub success: bool,
    pub output: String,
}

/// Evaluates a JSONPath query against a JSON string.
/// Returns a formatted array of results, or an error message if the query/JSON is invalid.
#[uniffi::export]
pub fn query_json(input: String, path: String) -> FFIQueryResult {
    match evaluate_path_on_str(&input, &path) {
        Ok(results) => {
            let mut out = String::new();
            out.push('[');
            for (i, node) in results.iter().enumerate() {
                out.push_str(&format(node));
                if i < results.len() - 1 {
                    out.push_str(", ");
                }
            }
            out.push(']');
            FFIQueryResult { success: true, output: out }
        },
        Err(e) => FFIQueryResult { success: false, output: e },
    }
}

/// A UniFFI-compatible recursive AST node for the UI symbols tree.
#[derive(uniffi::Record)]
pub struct FFISymbolNode {
    pub key: String,
    pub value_str: Option<String>,
    pub children: Option<Vec<FFISymbolNode>>,
}

/// Parses the JSON and returns the full AST symbols tree if valid.
#[uniffi::export]
pub fn get_ast_symbols(input: String) -> Option<FFISymbolNode> {
    let node = match parse(&input) {
        Ok(n) => n,
        Err(_) => return None,
    };
    Some(build_ffi_symbol("root".to_string(), &node))
}

fn build_ffi_symbol(key: String, node: &JsonNode) -> FFISymbolNode {
    match node {
        JsonNode::Object(pairs, _) => {
            let children = pairs.iter().map(|p| build_ffi_symbol(p.key.clone(), &p.value)).collect();
            FFISymbolNode { key, value_str: None, children: Some(children) }
        }
        JsonNode::Array(items, _) => {
            let children = items.iter().enumerate().map(|(i, n)| build_ffi_symbol(format!("[{}]", i), n)).collect();
            FFISymbolNode { key, value_str: None, children: Some(children) }
        }
        JsonNode::String(s, _) => FFISymbolNode { key, value_str: Some(format!("\"{}\"", s)), children: None },
        JsonNode::Number(_, raw, _) => FFISymbolNode { key, value_str: Some(raw.clone()), children: None },
        JsonNode::Bool(b, _) => FFISymbolNode { key, value_str: Some(b.to_string()), children: None },
        JsonNode::Null(_) => FFISymbolNode { key, value_str: Some("null".to_string()), children: None },
    }
}

/// A UniFFI-compatible autocomplete suggestion item.
#[derive(uniffi::Record)]
pub struct FFICompletionItem {
    pub key: String,
    pub path: String,
}

/// Provides autocomplete suggestions for a JSONPath prefix given the JSON document.
#[uniffi::export]
pub fn get_completions(input: String, path_prefix: String) -> Vec<FFICompletionItem> {
    match parse(&input) {
        Ok(node) => completions_at(&node, &path_prefix)
            .into_iter()
            .map(|c| FFICompletionItem {
                key: c.key,
                path: c.path,
            })
            .collect(),
        Err(_) => vec![],
    }
}


/// Generates dummy JSON data from a given schema string.
///
/// # Arguments
///
/// * `schema_input` - The JSON schema string.
/// * `target_size_kb` - Optional target size in kilobytes.
/// * `target_count` - Optional target count for generated array items.
///
/// # Returns
///
/// A `FFIQueryResult` containing the generated JSON string or an error message.
#[uniffi::export]
pub fn generate_dummy_data(
    schema_input: String,
    target_size_kb: Option<u32>,
    target_count: Option<u32>,
) -> FFIQueryResult {
    match parse(&schema_input) {
        Ok(schema_node) => {
            let opts = crate::generator::GeneratorOptions {
                target_size_bytes: target_size_kb.map(|kb| (kb * 1024) as usize),
                target_count: target_count.map(|c| c as usize),
            };
            match crate::generator::generate_from_schema(&schema_node, &opts) {
                Ok(generated) => FFIQueryResult {
                    success: true,
                    output: format(&generated),
                },
                Err(diags) => FFIQueryResult {
                    success: false,
                    output: format!("Generation failed with {} errors", diags.len()),
                },
            }
        }
        Err(diags) => FFIQueryResult {
            success: false,
            output: "Invalid schema JSON".to_string(),
        },
    }
}

/// Converts a JSON string to another data format.
///
/// # Arguments
///
/// * `input` - The JSON string to convert.
/// * `target_format` - The target format (e.g., "yaml", "toml", "xml").
///
/// # Returns
///
/// A `FFIQueryResult` containing the converted string or an error message.
#[uniffi::export]
pub fn convert_json(input: String, target_format: String) -> FFIQueryResult {
    use std::str::FromStr;
    match crate::converter::DataFormat::from_str(&target_format) {
        Ok(format) => match crate::converter::convert(&input, crate::converter::DataFormat::Json, format) {
            Ok(output) => FFIQueryResult { success: true, output },
            Err(e) => FFIQueryResult { success: false, output: e },
        },
        Err(e) => FFIQueryResult { success: false, output: e },
    }
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    /// **Test Case**: Ping returns pong prefix
    ///
    /// ### Description
    /// Validates the FFI bindings return a formatted ping response.
    ///
    /// ### Test Procedure
    /// 1. Execute `ping("world")`.
    ///
    /// ### Expected Result
    /// Result is `pong: world`.
    #[test]
    fn test_ping_returns_pong_prefix() {
        assert_eq!(ping("world".to_string()), "pong: world");
    }

    /// **Test Case**: Ping empty string
    ///
    /// ### Description
    /// Validates the FFI bindings process an empty payload safely.
    ///
    /// ### Test Procedure
    /// 1. Execute `ping("")`.
    ///
    /// ### Expected Result
    /// Result is `pong: `.
    #[test]
    fn test_ping_empty_string() {
        assert_eq!(ping(String::new()), "pong: ");
    }

    /// **Test Case**: Check syntax returns empty on valid JSON
    ///
    /// ### Description
    /// Validates that a correctly formatted JSON string produces no diagnostics.
    ///
    /// ### Test Procedure
    /// 1. Execute `check_syntax("{}")`.
    ///
    /// ### Expected Result
    /// Result is an empty vector.
    #[test]
    fn test_check_syntax_valid() {
        assert!(check_syntax("{}".to_string()).is_empty());
    }

    /// **Test Case**: Convert JSON FFI
    ///
    /// ### Description
    /// Validates the FFI bindings can convert JSON to YAML.
    ///
    /// ### Test Procedure
    /// 1. Execute `convert_json` with a JSON payload and target "yaml".
    ///
    /// ### Expected Result
    /// Result indicates success and contains YAML formatting.
    #[test]
    fn test_convert_json_ffi() {
        let json = r#"{"key": "value"}"#;
        let result = convert_json(json.to_string(), "yaml".to_string());
        assert!(result.success);
        assert!(result.output.contains("key: value"));
    }

    /// **Test Case**: Generate Dummy FFI
    ///
    /// ### Description
    /// Validates the FFI bindings can generate dummy JSON data from a schema.
    ///
    /// ### Test Procedure
    /// 1. Execute `generate_dummy_data` with a valid JSON schema.
    ///
    /// ### Expected Result
    /// Result indicates success and contains a JSON array string.
    #[test]
    fn test_generate_dummy_ffi() {
        let schema = r#"{ "id": { "@@type": "uuid" }, "age": { "@type": "integer" } }"#;
        let result = generate_dummy_data(schema.to_string(), None, Some(2));
        assert!(result.success);
        assert!(result.output.contains("["));
    }

    /// **Test Case**: Check syntax returns diagnostic on invalid JSON
    ///
    /// ### Description
    /// Validates that an incorrectly formatted JSON string produces diagnostic errors.
    ///
    /// ### Test Procedure
    /// 1. Execute `check_syntax("{")`.
    ///
    /// ### Expected Result
    /// Result contains at least one `FFIDiagnostic`.
    #[test]
    fn test_check_syntax_invalid() {
        let diags = check_syntax("{".to_string());
        assert!(!diags.is_empty());
    }
}
