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

//! The parsing module contains the strict and tolerant JSON parsers,
//! as well as fast diagnostics syntax checking and parsing helper utilities.

pub mod strict;
pub mod tolerant;
pub mod utils;

pub use strict::parse;

use crate::json_node::JsonNode;
use crate::types::Diagnostic;

/// Tolerant parsing: Attempts to build a partial AST even if errors are present.
/// Useful for live IDE feedback while typing, preserving as much of the tree as possible.
///
/// # Arguments
///
/// * `input` - The raw JSON string slice to parse (potentially incomplete or invalid).
///
/// # Returns
///
/// A tuple containing:
/// * `Option<JsonNode>` - The partial AST tree if any structure could be recovered.
/// * `Vec<Diagnostic>` - A list of errors encountered during parsing.
pub fn tolerant_parse(input: &str) -> (Option<JsonNode>, Vec<Diagnostic>) {
    // We opted for a hand-rolled tolerant parser (Option B) instead of tree-sitter.
    // While tree-sitter provides excellent error recovery, it introduces a C dependency
    // which complicates the cross-platform UniFFI build (especially for iOS/Android targets),
    // and slightly increases binary size. A minimal hand-rolled parser is sufficient for
    // JSON and gives us full control over `JsonNode` span generation without an intermediate AST.
    tolerant::parse(input)
}

/// Fast validation path: parses and returns only syntax or structural errors
/// without fully allocating the resulting AST tree.
///
/// # Arguments
///
/// * `input` - The raw JSON string slice to validate.
///
/// # Returns
///
/// A list of `Diagnostic` errors found in the input. If the input is valid JSON, this is empty.
pub fn diagnostics(input: &str) -> Vec<Diagnostic> {
    todo!("Fast diagnostics check will be implemented in subsequent issues")
}

#[cfg(test)]
mod stub_tests {
    use super::*;

    #[test]
    fn test_tolerant_parse_is_implemented() {
        let (node, _) = tolerant_parse("{}");
        assert!(node.is_some());
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn test_diagnostics_is_stub() {
        diagnostics("{}");
    }
}
