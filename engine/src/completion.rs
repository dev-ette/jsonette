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

//! JSONPath auto-completion and suggestion engine.

use crate::json_node::JsonNode;
use crate::types::CompletionItem;

/// Provides autocomplete suggestions for a JSONPath prefix given the current document state.
///
/// # Arguments
///
/// * `node` - A reference to the parsed root `JsonNode` context.
/// * `path_prefix` - The incomplete JSONPath prefix string slice typed by the user.
///
/// # Returns
///
/// A list of `CompletionItem` candidates suitable for autocomplete suggestions.
pub fn completions_at(node: &JsonNode, path_prefix: &str) -> Vec<CompletionItem> {
    let (parent_path, prefix) = match path_prefix.rfind('.') {
        Some(idx) => {
            let parent = &path_prefix[..idx];
            let pref = &path_prefix[idx + 1..];
            let parent = if parent.is_empty() { "$" } else { parent };
            (parent, pref)
        }
        None => {
            if path_prefix == "$" {
                ("$", "")
            } else {
                return vec![];
            }
        }
    };

    let mut completions = Vec::new();

    if let Ok(results) = crate::query::evaluate_path(node, parent_path) {
        for result in results {
            if let JsonNode::Object(pairs, _) = result {
                for kv in pairs {
                    if kv.key.starts_with(prefix) {
                        let new_path = if parent_path == "$" {
                            format!("$.{}", kv.key)
                        } else {
                            format!("{}.{}", parent_path, kv.key)
                        };
                        completions.push(CompletionItem {
                            key: kv.key.clone(),
                            path: new_path,
                        });
                    }
                }
            }
        }
    }

    completions.sort_by(|a, b| a.key.cmp(&b.key));
    completions.dedup_by(|a, b| a.key == b.key);

    completions
}

#[cfg(test)]
mod stub_tests {
    use super::*;
    use crate::parser::parse;

    /// **Test Case**: Autocomplete at Root Object
    ///
    /// ### Description
    /// Verifies that an empty JSONPath prefix `$.` returns all top-level keys.
    ///
    /// ### Test Procedure
    /// 1. Parse a valid JSON object.
    /// 2. Request completions at prefix `$.`.
    ///
    /// ### Expected Result
    /// Returns all top-level keys sorted alphabetically.
    #[test]
    fn test_completions_at_root() {
        let node = parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let comps = completions_at(&node, "$.");
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].key, "age");
        assert_eq!(comps[1].key, "name");
    }

    /// **Test Case**: Autocomplete with Partial Key Prefix
    ///
    /// ### Description
    /// Verifies that typing a partial key `$.na` filters completions to only keys starting with `na`.
    ///
    /// ### Test Procedure
    /// 1. Parse an object with multiple keys.
    /// 2. Request completions at prefix `$.na`.
    ///
    /// ### Expected Result
    /// Returns only keys matching the `na` prefix.
    #[test]
    fn test_completions_at_prefix() {
        let node = parse(r#"{"name": "Alice", "age": 30, "nested": {"nav": 1}}"#).unwrap();
        let comps = completions_at(&node, "$.na");
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].key, "name");
        assert_eq!(comps[0].path, "$.name");
    }

    /// **Test Case**: Autocomplete within Nested Object
    ///
    /// ### Description
    /// Verifies that autocomplete successfully evaluates a nested JSONPath prefix
    /// and correctly filters the nested object's keys.
    ///
    /// ### Test Procedure
    /// 1. Parse an object with a nested object.
    /// 2. Request completions at prefix `$.nested.na`.
    ///
    /// ### Expected Result
    /// Returns keys inside `nested` that start with `na`, with the fully resolved paths.
    #[test]
    fn test_completions_nested() {
        let node = parse(r#"{"nested": {"name": "Alice", "nav": 1}}"#).unwrap();
        let comps = completions_at(&node, "$.nested.na");
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].key, "name");
        assert_eq!(comps[0].path, "$.nested.name");
        assert_eq!(comps[1].key, "nav");
        assert_eq!(comps[1].path, "$.nested.nav");
    }
}
