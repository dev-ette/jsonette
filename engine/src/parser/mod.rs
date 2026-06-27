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
    todo!("Tolerant JSON parsing logic will be implemented in subsequent issues")
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
