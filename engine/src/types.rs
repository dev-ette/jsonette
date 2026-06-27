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

//! Public API types definitions

/// A byte-offset range in the source text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    /// The starting byte offset (inclusive).
    pub start: usize,
    /// The ending byte offset (exclusive).
    pub end: usize,
}

/// A single diagnostic (error or warning) from the engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    /// The span of the input text where this diagnostic applies.
    pub span: Span,
    /// The diagnostic message explaining the error or warning.
    pub message: String,
}

/// The line ending style to use when formatting text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    /// Unix line ending (Line Feed: `\n`).
    LF,
    /// Windows line ending (Carriage Return + Line Feed: `\r\n`).
    CRLF,
}

/// The folding (wrapping) style to use for JSON arrays and objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldingStyle {
    /// Always expand elements and key-value pairs onto new lines.
    Expanded,
    /// Keep elements/keys inline on a single line if they are empty or small.
    Compact,
}

/// Options for the formatter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormatOptions {
    /// Whether to use tab characters (`\t`) for indentation
    pub use_tabs: bool,
    /// The number of spaces to use for indentation.
    pub indent: u8,
    /// The line ending character sequence to use.
    pub line_ending: LineEnding,
    /// The wrapping/folding strategy for objects and arrays.
    pub folding_style: FoldingStyle,
    /// Whether to sort objects keys alphabetically.
    pub sort_keys: bool,
    /// Whether to insert a space after colons (e.g. `{"key": "value"}` vs `{"key":"value"}`).
    pub space_after_colon: bool,
    /// Whether to insert a space after commas (e.g. `[1, 2]` vs `[1,2]`).
    pub space_after_comma: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            use_tabs: false,
            indent: 2,
            line_ending: LineEnding::LF,
            folding_style: FoldingStyle::Expanded,
            sort_keys: false,
            space_after_colon: true,
            space_after_comma: true,
        }
    }
}

/// A single autocomplete suggestion.
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    /// The key/value candidate suggested for autocompletion.
    pub key: String,
    /// The JSONPath of the parent node context where this key applies.
    pub path: String,
}
