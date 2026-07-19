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

//! Pretty printing and minification for the JSON AST.

use crate::json_node::JsonNode;
use crate::types::LineEnding;

/// Formats the JSON node.
/// Uses lossless number representations to preserve precision.
/// Retrieves configuration settings directly from the global singleton.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode` to format.
///
/// # Returns
///
/// The pretty-printed JSON string representation of the node.
pub fn format(node: &JsonNode) -> String {
    let opts = crate::settings::get_settings().format;
    let mut result = String::with_capacity(1024);
    format_impl(node, 0, &opts, &mut result);
    result
}

/// Minifies the JSON node, stripping all whitespace and formatting.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode` to minify.
///
/// # Returns
///
/// A single-line JSON string representation of the node without any unnecessary whitespace.
pub fn minify(node: &JsonNode) -> String {
    let mut result = String::with_capacity(1024);
    minify_impl(node, &mut result);
    result
}

/// Computes the indentation prefix string for a given level of nesting.
/// Supports both tab-based and space-based indentation.
/// Retrieves active preferences from the global settings manager.
///
/// # Arguments
///
/// * `level` - The current depth level of nesting.
///
/// # Returns
///
/// A `String` consisting of space or tab characters.
fn get_indent(level: usize, opts: &crate::types::FormatOptions) -> String {
    if level == 0 {
        return String::new();
    }
    if opts.use_tabs {
        "\t".repeat(level)
    } else {
        " ".repeat(level * opts.indent as usize)
    }
}

/// Returns the line ending character sequence corresponding to the configuration.
/// Retrieves active preferences from the global settings manager.
///
/// # Returns
///
/// A static string slice (`"\n"` or `"\r\n"`).
fn get_line_ending(opts: &crate::types::FormatOptions) -> &'static str {
    match opts.line_ending {
        LineEnding::LF => "\n",
        LineEnding::CRLF => "\r\n",
    }
}

/// Attempts to format a JSON node into a single-line string representation.
/// Used recursively to inline compact objects and arrays if they are empty or small.
/// Retrieves active preferences from the global settings manager.
///
/// # Arguments
///
/// * `node` - A reference to the `JsonNode` to inline-format.
///
/// # Returns
///
/// `Some(String)` if the node could be formatted inline within 80 characters, or `None` otherwise.
fn format_inline(node: &JsonNode, opts: &crate::types::FormatOptions) -> Option<String> {
    match node {
        JsonNode::Null(_) => Some("null".to_string()),
        JsonNode::Bool(b, _) => Some(if *b { "true" } else { "false" }.to_string()),
        JsonNode::Number(_, raw, _) => Some(raw.clone()),
        JsonNode::String(s, _) => Some(serde_json::to_string(s).unwrap()),
        JsonNode::Array(elements, _) => {
            if elements.is_empty() {
                return Some("[]".to_string());
            }
            let mut parts = Vec::with_capacity(elements.len());
            for elem in elements {
                let part = format_inline(elem, opts)?;
                parts.push(part);
            }
            let comma_space = if opts.space_after_comma { ", " } else { "," };
            let joined = parts.join(comma_space);
            let result = format!("[{}]", joined);
            // Limit to 80 characters for inline format
            if result.len() <= 80 {
                Some(result)
            } else {
                None
            }
        }
        JsonNode::Object(pairs, _) => {
            if pairs.is_empty() {
                return Some("{}".to_string());
            }
            let mut pairs = pairs.clone();
            if opts.sort_keys {
                pairs.sort_by(|a, b| a.key.cmp(&b.key));
            }
            let mut parts = Vec::with_capacity(pairs.len());
            for pair in &pairs {
                let val_str = format_inline(&pair.value, opts)?;
                let key_str = serde_json::to_string(&pair.key).unwrap();
                let colon_space = if opts.space_after_colon { ": " } else { ":" };
                parts.push(format!("{}{}{}", key_str, colon_space, val_str));
            }
            let comma_space = if opts.space_after_comma { ", " } else { "," };
            let joined = parts.join(comma_space);
            let result = format!("{{{}}}", joined);
            // Limit to 80 characters for inline format
            if result.len() <= 80 {
                Some(result)
            } else {
                None
            }
        }
    }
}

/// Recursively formats a JSON node into a pretty-printed string at a specific indent level.
/// Retrieves active preferences from the global settings manager.
///
/// # Arguments
///
/// * `node` - A reference to the `JsonNode` to format.
/// * `level` - The current indentation depth level.
/// * `opts` - Formatting options.
/// * `out` - Mutable string buffer to append formatted output.
fn format_impl(
    node: &JsonNode,
    level: usize,
    opts: &crate::types::FormatOptions,
    out: &mut String,
) {
    match node {
        JsonNode::Null(_) => out.push_str("null"),
        JsonNode::Bool(b, _) => out.push_str(if *b { "true" } else { "false" }),
        JsonNode::Number(_, raw, _) => out.push_str(raw),
        JsonNode::String(s, _) => out.push_str(&serde_json::to_string(s).unwrap()),
        JsonNode::Array(elements, _) => {
            if elements.is_empty() {
                out.push_str("[]");
                return;
            }
            if let crate::types::FoldingStyle::Compact = opts.folding_style
                && let Some(inline) = format_inline(node, opts)
            {
                out.push_str(&inline);
                return;
            }

            let line_ending = get_line_ending(opts);
            let next_indent = get_indent(level + 1, opts);
            let current_indent = get_indent(level, opts);

            out.push('[');
            out.push_str(line_ending);

            for (i, elem) in elements.iter().enumerate() {
                out.push_str(&next_indent);
                format_impl(elem, level + 1, opts, out);
                if i + 1 < elements.len() {
                    out.push(',');
                }
                out.push_str(line_ending);
            }

            out.push_str(&current_indent);
            out.push(']');
        }
        JsonNode::Object(pairs, _) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            if let crate::types::FoldingStyle::Compact = opts.folding_style
                && let Some(inline) = format_inline(node, opts)
            {
                out.push_str(&inline);
                return;
            }

            // Note: we might want to avoid cloning pairs just to sort, but let's keep it functionally equivalent.
            let mut pairs_ref: Vec<_> = pairs.iter().collect();
            if opts.sort_keys {
                pairs_ref.sort_by(|a, b| a.key.cmp(&b.key));
            }

            let line_ending = get_line_ending(opts);
            let next_indent = get_indent(level + 1, opts);
            let current_indent = get_indent(level, opts);
            let colon_str = if opts.space_after_colon { ": " } else { ":" };

            out.push('{');
            out.push_str(line_ending);

            for (i, pair) in pairs_ref.iter().enumerate() {
                out.push_str(&next_indent);
                out.push_str(&serde_json::to_string(&pair.key).unwrap());
                out.push_str(colon_str);
                format_impl(&pair.value, level + 1, opts, out);
                if i + 1 < pairs.len() {
                    out.push(',');
                }
                out.push_str(line_ending);
            }

            out.push_str(&current_indent);
            out.push('}');
        }
    }
}

/// Recursively minifies a JSON node, stripping all whitespace and formatting.
///
/// # Arguments
///
/// * `node` - A reference to the `JsonNode` to minify.
/// * `out` - Mutable string buffer to append minified output.
fn minify_impl(node: &JsonNode, out: &mut String) {
    match node {
        JsonNode::Null(_) => out.push_str("null"),
        JsonNode::Bool(b, _) => out.push_str(if *b { "true" } else { "false" }),
        JsonNode::Number(_, raw, _) => out.push_str(raw),
        JsonNode::String(s, _) => out.push_str(&serde_json::to_string(s).unwrap()),
        JsonNode::Array(elements, _) => {
            out.push('[');
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                minify_impl(elem, out);
            }
            out.push(']');
        }
        JsonNode::Object(pairs, _) => {
            out.push('{');
            for (i, pair) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(&pair.key).unwrap());
                out.push(':');
                minify_impl(&pair.value, out);
            }
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::types::{FoldingStyle, FormatOptions};
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_temporary_settings<F, R>(opts: FormatOptions, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let mut app_settings = crate::settings::get_settings();
        let original_format = app_settings.format;
        app_settings.format = opts;
        crate::settings::update_settings(app_settings).unwrap();
        let result = f();
        app_settings.format = original_format;
        crate::settings::update_settings(app_settings).unwrap();
        result
    }

    /// **Test Case**: Format primitives
    ///
    /// ### Description
    /// Validates base primitives are formatted correctly.
    ///
    /// ### Test Procedure
    /// 1. Parse and format `null`, `true`, `false`, `123`, and string primitives.
    ///
    /// ### Expected Result
    /// Standard formatting output equals the input identity.
    #[test]
    fn test_format_primitives() {
        let _guard = TEST_LOCK.lock().unwrap();
        let null_node = parse("null").unwrap();
        assert_eq!(format(&null_node), "null");
        assert_eq!(minify(&null_node), "null");

        let true_node = parse("true").unwrap();
        assert_eq!(format(&true_node), "true");
        assert_eq!(minify(&true_node), "true");

        let false_node = parse("false").unwrap();
        assert_eq!(format(&false_node), "false");
        assert_eq!(minify(&false_node), "false");

        let num_node = parse("123.45e-2").unwrap();
        assert_eq!(format(&num_node), "123.45e-2");
        assert_eq!(minify(&num_node), "123.45e-2");

        let str_node = parse("\"hello \\u263a world\"").unwrap();
        assert_eq!(format(&str_node), "\"hello ☺ world\"");
        assert_eq!(minify(&str_node), "\"hello ☺ world\"");
    }

    /// **Test Case**: Format empty nodes
    ///
    /// ### Description
    /// Validates empty objects and arrays do not collapse spacing incorrectly.
    ///
    /// ### Test Procedure
    /// 1. Parse and format `{}` and `[]`.
    ///
    /// ### Expected Result
    /// Result identically matches without newlines.
    #[test]
    fn test_format_empty() {
        let _guard = TEST_LOCK.lock().unwrap();
        let empty_arr = parse("[]").unwrap();
        assert_eq!(format(&empty_arr), "[]");
        assert_eq!(minify(&empty_arr), "[]");

        let empty_obj = parse("{}").unwrap();
        assert_eq!(format(&empty_obj), "{}");
        assert_eq!(minify(&empty_obj), "{}");
    }

    /// **Test Case**: Format expanded array
    ///
    /// ### Description
    /// Validates expanded line wrapping for array elements.
    ///
    /// ### Test Procedure
    /// 1. Provide an array with primitive elements.
    ///
    /// ### Expected Result
    /// Output contains line breaks per array property.
    #[test]
    fn test_format_expanded_array() {
        let _guard = TEST_LOCK.lock().unwrap();
        let input = "[1,true,null]";
        let node = parse(input).unwrap();
        let expected = "[\n  1,\n  true,\n  null\n]";
        assert_eq!(format(&node), expected);
        assert_eq!(minify(&node), "[1,true,null]");
    }

    /// **Test Case**: Format expanded object
    ///
    /// ### Description
    /// Validates expanded line wrapping for object properties.
    ///
    /// ### Test Procedure
    /// 1. Provide an object with primitive values.
    ///
    /// ### Expected Result
    /// Output contains line breaks and indentations per key-value pair.
    #[test]
    fn test_format_expanded_object() {
        let _guard = TEST_LOCK.lock().unwrap();
        let input = "{\"a\":1,\"b\":true}";
        let node = parse(input).unwrap();
        let expected = "{\n  \"a\": 1,\n  \"b\": true\n}";
        assert_eq!(format(&node), expected);
        assert_eq!(minify(&node), "{\"a\":1,\"b\":true}");
    }

    /// **Test Case**: Format compact folding
    ///
    /// ### Description
    /// Tests compact folding logic where children nodes collapse on the same line.
    ///
    /// ### Test Procedure
    /// 1. Enable `compact` folding and parse objects and arrays.
    ///
    /// ### Expected Result
    /// The formatted output contains no newlines.
    #[test]
    fn test_format_compact_folding() {
        let _guard = TEST_LOCK.lock().unwrap();
        let opts = FormatOptions {
            folding_style: FoldingStyle::Compact,
            ..Default::default()
        };
        let input = "[1,true,null]";
        let node = parse(input).unwrap();
        let res = with_temporary_settings(opts, || format(&node));
        assert_eq!(res, "[1, true, null]");

        let obj_input = "{\"a\":1,\"b\":true}";
        let obj_node = parse(obj_input).unwrap();
        let res_obj = with_temporary_settings(opts, || format(&obj_node));
        assert_eq!(res_obj, "{\"a\": 1, \"b\": true}");
    }

    /// **Test Case**: Format sort keys
    ///
    /// ### Description
    /// Tests key sorting during formatting.
    ///
    /// ### Test Procedure
    /// 1. Enable key sorting and format an object with unordered keys.
    ///
    /// ### Expected Result
    /// Keys are ordered alphabetically.
    #[test]
    fn test_format_sort_keys() {
        let _guard = TEST_LOCK.lock().unwrap();
        let opts = FormatOptions {
            sort_keys: true,
            ..Default::default()
        };
        let input = "{\"z\":1,\"a\":true}";
        let node = parse(input).unwrap();
        let expected = "{\n  \"a\": true,\n  \"z\": 1\n}";
        let res = with_temporary_settings(opts, || format(&node));
        assert_eq!(res, expected);
    }
}
