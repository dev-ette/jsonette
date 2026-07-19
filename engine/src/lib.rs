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
