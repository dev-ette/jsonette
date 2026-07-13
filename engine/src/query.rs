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

//! JSONPath expression evaluation and validation.

use crate::json_node::JsonNode;
use crate::types::Diagnostic;

/// Evaluates a JSONPath query against the AST, returning all matching nodes.
///
/// # Arguments
///
/// * `node` - A reference to the root `JsonNode` to query.
/// * `path` - The JSONPath expression string slice (e.g. `$.store.book[*].author`).
///
/// # Returns
///
/// * `Ok(Vec<JsonNode>)` - A list of JSON nodes that match the query.
/// * `Err(String)` - An error message indicating the JSONPath syntax is invalid.
pub fn evaluate_path(node: &JsonNode, path: &str) -> Result<Vec<JsonNode>, String> {
    todo!("JSONPath evaluation logic will be implemented in subsequent issues")
}

/// Validates the syntax of a JSONPath expression without evaluating it against a document.
///
/// # Arguments
///
/// * `path` - The JSONPath expression string slice to validate.
///
/// # Returns
///
/// A list of `Diagnostic` errors found in the path syntax. If the path is valid, this is empty.
pub fn diagnostics_for_path(path: &str) -> Vec<Diagnostic> {
    todo!("JSONPath syntax validation will be implemented in subsequent issues")
}

#[cfg(test)]
mod stub_tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_evaluate_path_is_stub() {
        let node = parse("{}").unwrap();
        evaluate_path(&node, "$.foo").unwrap();
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_diagnostics_for_path_is_stub() {
        diagnostics_for_path("$.foo");
    }
}
