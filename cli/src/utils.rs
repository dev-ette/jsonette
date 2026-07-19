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
 * See the License for the licenses.
 */

//! Command-line helper utilities.
//!
//! Includes helpers to read standard input or file paths, convert byte offsets
//! to line/column numbers, and print diagnostic syntax errors.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Reads data from a file path or falls back to reading from standard input.
///
/// # Arguments
///
/// * `file` - An optional `PathBuf` pointing to a file.
///
/// # Returns
///
/// A tuple containing:
/// * `String` - The read content.
/// * `String` - A descriptive label of the input source (file path or `<stdin>`).
///
/// # Errors
///
/// Returns an `io::Error` if reading standard input or the file fails.
pub fn read_input(file: &Option<PathBuf>) -> io::Result<(String, String)> {
    match file {
        Some(path) => {
            let content = fs::read_to_string(path)?;
            let label = path.to_string_lossy().to_string();
            Ok((content, label))
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok((buffer, "<stdin>".to_string()))
        }
    }
}

/// Converts a 0-indexed byte offset into 1-indexed line and column coordinates.
///
/// # Arguments
///
/// * `input` - The raw source string.
/// * `offset` - The byte offset index.
///
/// # Returns
///
/// A tuple `(line, column)` containing 1-indexed coordinates.
pub fn byte_offset_to_line_col(input: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    let limit = offset.min(input.len());
    for (i, c) in input.char_indices() {
        if i >= limit {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Prints a list of diagnostic errors to `stderr` with compiler-style visual carets.
///
/// # Arguments
///
/// * `input` - The source text containing errors.
/// * `diagnostics` - A slice of `jsonette_core::Diagnostic` instances representing the errors.
/// * `file_label` - A string label representing the source input (file path or `<stdin>`).
///
/// # Returns
///
/// Nothing.
pub fn print_diagnostics(input: &str, diagnostics: &[jsonette_core::Diagnostic], file_label: &str) {
    for diag in diagnostics {
        let (line, col) = byte_offset_to_line_col(input, diag.span.start);
        eprintln!("Error in {}:{}:{}: {}", file_label, line, col, diag.message);

        // Print the line containing the error with a caret
        let lines: Vec<&str> = input.lines().collect();
        if !lines.is_empty() {
            let visual_line = line.min(lines.len());
            let error_line = lines[visual_line - 1];
            eprintln!("  |");
            eprintln!("{:>3} | {}", visual_line, error_line);

            let caret_col = if line > lines.len() {
                error_line.len() + 1
            } else {
                col
            };
            let span_len = (diag.span.end.saturating_sub(diag.span.start)).max(1);
            let indent = " ".repeat(caret_col.saturating_sub(1));
            let caret = "^".repeat(span_len);
            eprintln!("  | {}{}", indent, caret);
        }
    }
}
