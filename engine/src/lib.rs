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
pub mod formatter;
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
pub use query::{diagnostics_for_path, evaluate_path};
pub use settings::{Settings, get_settings, set_in_memory_settings, update_settings};
