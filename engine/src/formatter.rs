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
    format_impl(node, 0, &opts)
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
    minify_impl(node)
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
///
/// # Returns
///
/// The pretty-printed string representation of the sub-tree.
fn format_impl(node: &JsonNode, level: usize, opts: &crate::types::FormatOptions) -> String {
    match node {
        JsonNode::Null(_) => "null".to_string(),
        JsonNode::Bool(b, _) => if *b { "true" } else { "false" }.to_string(),
        JsonNode::Number(_, raw, _) => raw.clone(),
        JsonNode::String(s, _) => serde_json::to_string(s).unwrap(),
        JsonNode::Array(elements, _) => {
            if elements.is_empty() {
                return "[]".to_string();
            }
            if let (crate::types::FoldingStyle::Compact, Some(inline)) =
                (&opts.folding_style, format_inline(node, opts))
            {
                return inline;
            }

            let line_ending = get_line_ending(opts);
            let next_indent = get_indent(level + 1, opts);
            let current_indent = get_indent(level, opts);

            let mut result = String::new();
            result.push('[');
            result.push_str(line_ending);

            for (i, elem) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format_impl(elem, level + 1, opts));
                if i + 1 < elements.len() {
                    result.push(',');
                }
                result.push_str(line_ending);
            }

            result.push_str(&current_indent);
            result.push(']');
            result
        }
        JsonNode::Object(pairs, _) => {
            if pairs.is_empty() {
                return "{}".to_string();
            }
            if let (crate::types::FoldingStyle::Compact, Some(inline)) =
                (&opts.folding_style, format_inline(node, opts))
            {
                return inline;
            }

            let mut pairs = pairs.clone();
            if opts.sort_keys {
                pairs.sort_by(|a, b| a.key.cmp(&b.key));
            }

            let line_ending = get_line_ending(opts);
            let next_indent = get_indent(level + 1, opts);
            let current_indent = get_indent(level, opts);
            let colon_str = if opts.space_after_colon { ": " } else { ":" };

            let mut result = String::new();
            result.push('{');
            result.push_str(line_ending);

            for (i, pair) in pairs.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serde_json::to_string(&pair.key).unwrap());
                result.push_str(colon_str);
                result.push_str(&format_impl(&pair.value, level + 1, opts));
                if i + 1 < pairs.len() {
                    result.push(',');
                }
                result.push_str(line_ending);
            }

            result.push_str(&current_indent);
            result.push('}');
            result
        }
    }
}

/// Recursively minifies a JSON node, stripping all whitespace and formatting.
///
/// # Arguments
///
/// * `node` - A reference to the `JsonNode` to minify.
///
/// # Returns
///
/// The minified string representation of the sub-tree.
fn minify_impl(node: &JsonNode) -> String {
    match node {
        JsonNode::Null(_) => "null".to_string(),
        JsonNode::Bool(b, _) => if *b { "true" } else { "false" }.to_string(),
        JsonNode::Number(_, raw, _) => raw.clone(),
        JsonNode::String(s, _) => serde_json::to_string(s).unwrap(),
        JsonNode::Array(elements, _) => {
            let mut parts = Vec::with_capacity(elements.len());
            for elem in elements {
                parts.push(minify_impl(elem));
            }
            format!("[{}]", parts.join(","))
        }
        JsonNode::Object(pairs, _) => {
            let mut parts = Vec::with_capacity(pairs.len());
            for pair in pairs {
                let val_str = minify_impl(&pair.value);
                let key_str = serde_json::to_string(&pair.key).unwrap();
                parts.push(format!("{}:{}", key_str, val_str));
            }
            format!("{{{}}}", parts.join(","))
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

    #[test]
    fn test_format_expanded_array() {
        let _guard = TEST_LOCK.lock().unwrap();
        let input = "[1,true,null]";
        let node = parse(input).unwrap();
        let expected = "[\n  1,\n  true,\n  null\n]";
        assert_eq!(format(&node), expected);
        assert_eq!(minify(&node), "[1,true,null]");
    }

    #[test]
    fn test_format_expanded_object() {
        let _guard = TEST_LOCK.lock().unwrap();
        let input = "{\"a\":1,\"b\":true}";
        let node = parse(input).unwrap();
        let expected = "{\n  \"a\": 1,\n  \"b\": true\n}";
        assert_eq!(format(&node), expected);
        assert_eq!(minify(&node), "{\"a\":1,\"b\":true}");
    }

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
