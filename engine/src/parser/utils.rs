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

//! Internal parser utilities, including mapping line/column errors back to byte offsets.

/// Helper to convert a 1-indexed line and column to a 0-indexed byte offset in the input string.
///
/// # Arguments
///
/// * `input` - The raw JSON string slice.
/// * `line` - The 1-indexed line number where the error occurred.
/// * `col` - The 1-indexed column number where the error occurred.
///
/// # Returns
///
/// The 0-indexed byte offset of the character in the input string. If `line` is `0`, `0` is returned.
/// If `line` or `col` exceeds the boundaries of the string, the length of the string is returned.
pub fn line_col_to_byte_offset(input: &str, line: usize, col: usize) -> usize {
    if line == 0 {
        return 0;
    }
    let mut current_line = 1;
    let mut current_col = 1;
    for (offset, c) in input.char_indices() {
        if current_line == line && current_col == col {
            return offset;
        }
        if c == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    input.len()
}
