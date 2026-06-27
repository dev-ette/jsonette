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

//! Strict JSON parser implementation carrying byte-accurate spans for AST nodes.

use super::utils::line_col_to_byte_offset;
use crate::json_node::{JsonNode, KeyValuePair};
use crate::types::{Diagnostic, Span};

/// Strict parsing: Fails entirely if the JSON is invalid.
/// Returns the parsed tree or a list of diagnostic errors.
/// Primarily used for final validation.
///
/// # Arguments
///
/// * `input` - The raw JSON string slice to parse.
///
/// # Returns
///
/// * `Ok(JsonNode)` - The parsed JSON abstract syntax tree (AST) on successful parse.
/// * `Err(Vec<Diagnostic>)` - A list of syntax or structural errors found during parsing.
pub fn parse(input: &str) -> Result<JsonNode, Vec<Diagnostic>> {
    // 1. Validate with serde_json to ensure standard compliance
    if let Err(err) = serde_json::from_str::<serde_json::Value>(input) {
        let line = err.line();
        let col = err.column();
        let offset = line_col_to_byte_offset(input, line, col);
        let diag = Diagnostic {
            span: Span {
                start: offset,
                end: (offset + 1).min(input.len()),
            },
            message: err.to_string(),
        };
        return Err(vec![diag]);
    }

    // 2. Parse with our hand-rolled parser to build the AST with correct spans
    let mut parser = Parser::new(input);
    match parser.parse_value() {
        Ok(node) => {
            parser.skip_whitespace();
            if parser.cursor < parser.input.len() {
                Err(vec![parser.error(
                    parser.cursor,
                    "Unexpected trailing characters after JSON value",
                )])
            } else {
                Ok(node)
            }
        }
        Err(diag) => Err(vec![diag]),
    }
}

/// A stateful recursive-descent parser for strict JSON documents.
/// Keeps track of byte offset locations to generate AST nodes with accurate `Span` info.
struct Parser<'a> {
    /// The input bytes slice of the JSON document.
    input: &'a [u8],
    /// The original input string slice for number parsing and error reporting.
    input_str: &'a str,
    /// The current byte offset cursor in the input.
    cursor: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new Parser instance for the given JSON input.
    fn new(input: &'a str) -> Self {
        Parser {
            input: input.as_bytes(),
            input_str: input,
            cursor: 0,
        }
    }

    /// Returns the character byte at the current cursor, or `None` if EOF is reached.
    fn peek(&self) -> Option<u8> {
        if self.cursor < self.input.len() {
            Some(self.input[self.cursor])
        } else {
            None
        }
    }

    /// Advances the cursor by one byte.
    fn advance(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Skips any ASCII whitespace characters (spaces, tabs, newlines, carriage returns).
    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Helper to create a single-character `Diagnostic` error starting at the given position.
    fn error(&self, pos: usize, message: impl Into<String>) -> Diagnostic {
        let end = (pos + 1).min(self.input.len());
        Diagnostic {
            span: Span { start: pos, end },
            message: message.into(),
        }
    }

    /// Main entry point to parse a JSON value (null, bool, number, string, array, object).
    fn parse_value(&mut self) -> Result<JsonNode, Diagnostic> {
        self.skip_whitespace();
        let start = self.cursor;
        let b = match self.peek() {
            Some(b) => b,
            None => return Err(self.error(start, "Unexpected end of input")),
        };

        match b {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'"' => self.parse_string_node(),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(self.error(start, format!("Unexpected character '{}'", b as char))),
        }
    }

    /// Parses a JSON null value.
    fn parse_null(&mut self) -> Result<JsonNode, Diagnostic> {
        let start = self.cursor;
        if self.cursor + 4 <= self.input.len()
            && &self.input[self.cursor..self.cursor + 4] == b"null"
        {
            self.cursor += 4;
            Ok(JsonNode::Null(Span {
                start,
                end: self.cursor,
            }))
        } else {
            Err(self.error(start, "Expected 'null'"))
        }
    }

    /// Parses a JSON boolean value (true or false).
    fn parse_bool(&mut self) -> Result<JsonNode, Diagnostic> {
        let start = self.cursor;
        if self.cursor + 4 <= self.input.len()
            && &self.input[self.cursor..self.cursor + 4] == b"true"
        {
            self.cursor += 4;
            Ok(JsonNode::Bool(
                true,
                Span {
                    start,
                    end: self.cursor,
                },
            ))
        } else if self.cursor + 5 <= self.input.len()
            && &self.input[self.cursor..self.cursor + 5] == b"false"
        {
            self.cursor += 5;
            Ok(JsonNode::Bool(
                false,
                Span {
                    start,
                    end: self.cursor,
                },
            ))
        } else {
            Err(self.error(start, "Expected boolean value"))
        }
    }

    /// Parses a raw string value, decoding escape characters and surrogate pairs,
    /// and returns the decoded string and its source span.
    fn parse_string_raw(&mut self) -> Result<(String, Span), Diagnostic> {
        let start = self.cursor;
        if self.peek() != Some(b'"') {
            return Err(self.error(start, "Expected opening quote for string"));
        }
        self.advance(); // consume opening quote

        let mut s = String::new();
        while let Some(b) = self.peek() {
            match b {
                b'"' => {
                    self.advance(); // consume closing quote
                    return Ok((
                        s,
                        Span {
                            start,
                            end: self.cursor,
                        },
                    ));
                }
                b'\\' => {
                    self.advance(); // consume backslash
                    let esc = match self.peek() {
                        Some(esc) => esc,
                        None => return Err(self.error(self.cursor, "Unterminated string escape")),
                    };
                    self.advance(); // consume escape char
                    match esc {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'b' => s.push('\x08'),
                        b'f' => s.push('\x0c'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'u' => {
                            if self.cursor + 4 > self.input.len() {
                                return Err(
                                    self.error(self.cursor, "Invalid unicode escape sequence")
                                );
                            }
                            let hex_str =
                                std::str::from_utf8(&self.input[self.cursor..self.cursor + 4])
                                    .map_err(|_| {
                                        self.error(self.cursor, "Invalid utf-8 in unicode escape")
                                    })?;
                            let code_point = u16::from_str_radix(hex_str, 16).map_err(|_| {
                                self.error(self.cursor, "Invalid hex in unicode escape")
                            })?;
                            self.cursor += 4;

                            if (0xD800..=0xDBFF).contains(&code_point) {
                                if self.cursor + 6 <= self.input.len()
                                    && &self.input[self.cursor..self.cursor + 2] == b"\\u"
                                {
                                    self.cursor += 2;
                                    let low_hex_str = std::str::from_utf8(
                                        &self.input[self.cursor..self.cursor + 4],
                                    )
                                    .map_err(|_| {
                                        self.error(self.cursor, "Invalid utf-8 in low surrogate")
                                    })?;
                                    let low_code_point = u16::from_str_radix(low_hex_str, 16)
                                        .map_err(|_| {
                                            self.error(self.cursor, "Invalid hex in low surrogate")
                                        })?;
                                    self.cursor += 4;
                                    if (0xDC00..=0xDFFF).contains(&low_code_point) {
                                        let utf32 = (((code_point - 0xD800) as u32) << 10)
                                            + (low_code_point - 0xDC00) as u32
                                            + 0x10000;
                                        if let Some(c) = std::char::from_u32(utf32) {
                                            s.push(c);
                                        } else {
                                            return Err(self.error(
                                                self.cursor - 12,
                                                "Invalid surrogate pair",
                                            ));
                                        }
                                    } else {
                                        return Err(self.error(
                                            self.cursor - 6,
                                            "Expected low surrogate after high surrogate",
                                        ));
                                    }
                                } else {
                                    return Err(self.error(
                                        self.cursor,
                                        "Expected low surrogate after high surrogate",
                                    ));
                                }
                            } else if (0xDC00..=0xDFFF).contains(&code_point) {
                                return Err(self.error(
                                    self.cursor - 6,
                                    "Unexpected low surrogate without high surrogate",
                                ));
                            } else {
                                if let Some(c) = std::char::from_u32(code_point as u32) {
                                    s.push(c);
                                } else {
                                    return Err(
                                        self.error(self.cursor - 6, "Invalid unicode code point")
                                    );
                                }
                            }
                        }
                        _ => {
                            return Err(self.error(
                                self.cursor - 1,
                                format!("Invalid escape character '{}'", esc as char),
                            ));
                        }
                    }
                }
                b @ 0..=0x1f => {
                    return Err(self.error(self.cursor, "Control characters must be escaped"));
                }
                _ => {
                    let tail = match std::str::from_utf8(&self.input[self.cursor..]) {
                        Ok(t) => t,
                        Err(_) => return Err(self.error(self.cursor, "Invalid UTF-8 sequence")),
                    };
                    let c = match tail.chars().next() {
                        Some(ch) => ch,
                        None => return Err(self.error(self.cursor, "Unexpected EOF")),
                    };
                    self.cursor += c.len_utf8();
                    s.push(c);
                }
            }
        }
        Err(self.error(start, "Unterminated string"))
    }

    /// Parses a JSON string value.
    fn parse_string_node(&mut self) -> Result<JsonNode, Diagnostic> {
        let (s, span) = self.parse_string_raw()?;
        Ok(JsonNode::String(s, span))
    }

    /// Parses a JSON number value.
    fn parse_number(&mut self) -> Result<JsonNode, Diagnostic> {
        let start = self.cursor;

        if self.peek() == Some(b'-') {
            self.advance();
        }

        match self.peek() {
            Some(b'0') => {
                self.advance();
            }
            Some(b) if b.is_ascii_digit() => {
                while let Some(next_b) = self.peek() {
                    if next_b.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(self.error(start, "Expected digit for number")),
        }

        if self.peek() == Some(b'.') {
            self.advance();
            while let Some(next_b) = self.peek() {
                if next_b.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if let Some(b'e' | b'E') = self.peek() {
            self.advance();
            if let Some(b'+' | b'-') = self.peek() {
                self.advance();
            }
            while let Some(next_b) = self.peek() {
                if next_b.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let end = self.cursor;
        let span = Span { start, end };
        let raw_str = self.input_str[start..end].to_string();

        let val: f64 = raw_str.parse().unwrap_or(0.0);

        Ok(JsonNode::Number(val, raw_str, span))
    }

    /// Parses a JSON array value.
    fn parse_array(&mut self) -> Result<JsonNode, Diagnostic> {
        let start = self.cursor;
        if self.peek() != Some(b'[') {
            return Err(self.error(start, "Expected '['"));
        }
        self.advance(); // consume '['

        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.advance(); // consume ']'
            return Ok(JsonNode::Array(
                vec![],
                Span {
                    start,
                    end: self.cursor,
                },
            ));
        }

        let mut elements = Vec::new();
        loop {
            let val = self.parse_value()?;
            elements.push(val);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek() == Some(b']') {
                        return Err(
                            self.error(self.cursor, "Trailing commas are not allowed in JSON")
                        );
                    }
                }
                Some(b']') => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    return Err(self.error(
                        self.cursor,
                        format!(
                            "Expected ',' or ']' after array element, found '{}'",
                            b as char
                        ),
                    ));
                }
                None => {
                    return Err(self.error(self.cursor, "Unterminated array"));
                }
            }
        }

        Ok(JsonNode::Array(
            elements,
            Span {
                start,
                end: self.cursor,
            },
        ))
    }

    /// Parses a JSON object value.
    fn parse_object(&mut self) -> Result<JsonNode, Diagnostic> {
        let start = self.cursor;
        if self.peek() != Some(b'{') {
            return Err(self.error(start, "Expected '{'"));
        }
        self.advance(); // consume '{'

        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.advance(); // consume '}'
            return Ok(JsonNode::Object(
                vec![],
                Span {
                    start,
                    end: self.cursor,
                },
            ));
        }

        let mut pairs = Vec::new();
        loop {
            self.skip_whitespace();
            let key_start = self.cursor;
            if self.peek() != Some(b'"') {
                return Err(self.error(key_start, "Expected string key in object"));
            }
            let (key, _) = self.parse_string_raw()?;

            self.skip_whitespace();
            let colon_pos = self.cursor;
            if self.peek() != Some(b':') {
                return Err(self.error(colon_pos, "Expected ':' after key"));
            }
            self.advance(); // consume ':'

            let val = self.parse_value()?;
            pairs.push(KeyValuePair { key, value: val });

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek() == Some(b'}') {
                        return Err(
                            self.error(self.cursor, "Trailing commas are not allowed in JSON")
                        );
                    }
                }
                Some(b'}') => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    return Err(self.error(
                        self.cursor,
                        format!(
                            "Expected ',' or '}}' after object member, found '{}'",
                            b as char
                        ),
                    ));
                }
                None => {
                    return Err(self.error(self.cursor, "Unterminated object"));
                }
            }
        }

        Ok(JsonNode::Object(
            pairs,
            Span {
                start,
                end: self.cursor,
            },
        ))
    }
}

#[cfg(test)]
mod private_tests {
    use super::*;

    /// **Test Case**: Trailing Characters Check
    ///
    /// ### Description
    /// Verifies that the parser parses a valid JSON value but detects trailing unparsed characters.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with `"123 abc"`.
    /// 2. Call `parse_value()` to parse the number `123`.
    /// 3. Assert that the cursor has not reached the end of the input (trailing characters exist).
    ///
    /// ### Expected Result
    /// The parser parses the number `123` successfully and reports remaining characters at the end.
    #[test]
    fn test_parser_trailing_characters() {
        let mut parser = Parser::new("123 abc");
        let res = parser.parse_value();
        assert!(res.is_ok());
        parser.skip_whitespace();
        assert!(parser.cursor < parser.input.len());
    }

    /// **Test Case**: Unexpected End of Input Error
    ///
    /// ### Description
    /// Verifies that parsing an empty input string produces an "Unexpected end of input" error.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with an empty string `""`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `Err` with the message "Unexpected end of input".
    #[test]
    fn test_parser_unexpected_eof() {
        let mut parser = Parser::new("");
        let res = parser.parse_value();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message, "Unexpected end of input");
    }

    /// **Test Case**: Unexpected Character Error
    ///
    /// ### Description
    /// Verifies that an invalid JSON value starting character produces an "Unexpected character" error.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with `"x"`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `Err` with the message "Unexpected character 'x'".
    #[test]
    fn test_parser_unexpected_char() {
        let mut parser = Parser::new("x");
        let res = parser.parse_value();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message, "Unexpected character 'x'");
    }

    /// **Test Case**: Null Literal Parsing Error
    ///
    /// ### Description
    /// Verifies that a malformed `null` literal results in a parsing error.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with `"nula"`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `Err` with the message "Expected 'null'".
    #[test]
    fn test_parser_null_error() {
        let mut parser = Parser::new("nula");
        let res = parser.parse_value();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message, "Expected 'null'");
    }

    /// **Test Case**: Boolean Literal Parsing Error
    ///
    /// ### Description
    /// Verifies that malformed boolean literals result in parsing errors.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with `"truf"` and `"falz"`.
    /// 2. Call `parse_value()` on both.
    ///
    /// ### Expected Result
    /// Both return `Err` with the message "Expected boolean value".
    #[test]
    fn test_parser_bool_error() {
        let mut parser = Parser::new("truf");
        let res = parser.parse_value();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message, "Expected boolean value");

        let mut parser = Parser::new("falz");
        let res = parser.parse_value();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message, "Expected boolean value");
    }

    /// **Test Case**: String Literal Parsing Errors
    ///
    /// ### Description
    /// Verifies that various malformed string literals (unterminated, unescaped controls, invalid escape sequences) produce correct error messages.
    ///
    /// ### Test Procedure
    /// 1. Test unterminated string `"\"hello"`.
    /// 2. Test unescaped control character `"\u{08}"`.
    /// 3. Test invalid escape character `"\x"`.
    /// 4. Test unterminated string escape `"\`.
    /// 5. Test invalid unicode escape length `"\u1"`.
    /// 6. Test invalid hex character in unicode escape `"\u123g"`.
    /// 7. Test missing low surrogate after high surrogate `"\uD800"`.
    /// 8. Test invalid low surrogate token after high surrogate `"\uD800\u1234"`.
    /// 9. Test low surrogate without preceding high surrogate `"\uDC00"`.
    ///
    /// ### Expected Result
    /// All cases return `Err` with their respective parsing/syntax error messages.
    #[test]
    fn test_parser_string_errors() {
        let mut parser = Parser::new("\"hello");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Unterminated string"
        );

        let mut parser = Parser::new("\"\u{08}\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Control characters must be escaped"
        );

        let mut parser = Parser::new("\"\\x\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Invalid escape character 'x'"
        );

        let mut parser = Parser::new("\"\\");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Unterminated string escape"
        );

        let mut parser = Parser::new("\"\\u1\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Invalid unicode escape sequence"
        );

        let mut parser = Parser::new("\"\\u123g\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Invalid hex in unicode escape"
        );

        let mut parser = Parser::new("\"\\uD800\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected low surrogate after high surrogate"
        );

        let mut parser = Parser::new("\"\\uD800\\u1234\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected low surrogate after high surrogate"
        );

        let mut parser = Parser::new("\"\\uDC00\"");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Unexpected low surrogate without high surrogate"
        );
    }

    /// **Test Case**: Number Parsing Errors
    ///
    /// ### Description
    /// Verifies that invalid number formats (e.g., negative sign with no digits) result in a parsing error.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with `"-"`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `Err` with the message "Expected digit for number".
    #[test]
    fn test_parser_number_errors() {
        let mut parser = Parser::new("-");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected digit for number"
        );
    }

    /// **Test Case**: Array Parsing Errors
    ///
    /// ### Description
    /// Verifies error detection for malformed array declarations (unterminated arrays, trailing commas, missing separators).
    ///
    /// ### Test Procedure
    /// 1. Test unterminated array `"[1"`.
    /// 2. Test trailing comma `"[1, ]"`.
    /// 3. Test missing separator `"[1 2]"`.
    ///
    /// ### Expected Result
    /// All cases return `Err` with their respective parsing/syntax error messages.
    #[test]
    fn test_parser_array_errors() {
        let mut parser = Parser::new("[1");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Unterminated array"
        );

        let mut parser = Parser::new("[1, ]");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Trailing commas are not allowed in JSON"
        );

        let mut parser = Parser::new("[1 2]");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected ',' or ']' after array element, found '2'"
        );
    }

    /// **Test Case**: Object Parsing Errors
    ///
    /// ### Description
    /// Verifies error detection for malformed object declarations (non-string keys, missing colons, trailing commas, missing separators, unterminated objects).
    ///
    /// ### Test Procedure
    /// 1. Test non-string key `"{1: 2}"`.
    /// 2. Test missing colon `{"key" 1}`.
    /// 3. Test trailing comma `{"key": 1, }`.
    /// 4. Test missing separator `{"key": 1 "other": 2}`.
    /// 5. Test unterminated object `{"key": 1`.
    ///
    /// ### Expected Result
    /// All cases return `Err` with their respective parsing/syntax error messages.
    #[test]
    fn test_parser_object_errors() {
        let mut parser = Parser::new("{1: 2}");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected string key in object"
        );

        let mut parser = Parser::new("{\"key\" 1}");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected ':' after key"
        );

        let mut parser = Parser::new("{\"key\": 1, }");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Trailing commas are not allowed in JSON"
        );

        let mut parser = Parser::new("{\"key\": 1 \"other\": 2}");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Expected ',' or '}' after object member, found '\"'"
        );

        let mut parser = Parser::new("{\"key\": 1");
        assert_eq!(
            parser.parse_value().unwrap_err().message,
            "Unterminated object"
        );
    }

    /// **Test Case**: Offset Mapping Edge Cases
    ///
    /// ### Description
    /// Validates the robustness of the coordinate-to-byte-offset mapping utility
    /// under common boundary inputs.
    ///
    /// ### Test Procedure
    /// 1. Query an empty input string with line 0, column 0.
    /// 2. Query a valid multiline string with coordinates referencing the character 'e'.
    /// 3. Query an out-of-bounds line and column number.
    ///
    /// ### Expected Result
    /// 1. Line 0 returns 0.
    /// 2. Valid coordinates return the exact byte offset of the character 'e' (5).
    /// 3. Out-of-bounds queries fall back gracefully to the total input string length.
    #[test]
    fn test_line_col_to_byte_offset_edge_cases() {
        assert_eq!(line_col_to_byte_offset("", 0, 0), 0);
        assert_eq!(line_col_to_byte_offset("abc\ndef\n", 2, 2), 5); // 'e' is at index 5
        assert_eq!(line_col_to_byte_offset("abc", 5, 5), 3); // out of bounds
    }

    /// **Test Case**: String Escape Decoding
    ///
    /// ### Description
    /// Verifies that all standard character escapes (quote, backslash, slash, backspace, formfeed, newline, carriage return, tab) are correctly decoded.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with a string containing all escaped control characters: `\"\\\"\\\\\\/\\b\\f\\n\\r\\t\"`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `JsonNode::String` containing the correct unescaped string and span.
    #[test]
    fn test_parser_valid_escapes() {
        let mut parser = Parser::new("\"\\\"\\\\\\/\\b\\f\\n\\r\\t\"");
        let res = parser.parse_value().unwrap();
        assert_eq!(
            res,
            JsonNode::String(
                "\"\\/\x08\x0c\n\r\t".to_string(),
                Span { start: 0, end: 18 }
            )
        );
    }

    /// **Test Case**: Unicode Surrogate Pair Decoding
    ///
    /// ### Description
    /// Verifies that Unicode surrogate pairs (e.g., `\uD83D\uDE00` representing 😀) are successfully parsed and decoded into a UTF-8 character.
    ///
    /// ### Test Procedure
    /// 1. Initialize `Parser` with high and low surrogates: `\"\\uD83D\\uDE00\"`.
    /// 2. Call `parse_value()`.
    ///
    /// ### Expected Result
    /// Returns `JsonNode::String` containing "😀" and span `0..14`.
    #[test]
    fn test_parser_surrogate_pair() {
        let mut parser = Parser::new("\"\\uD83D\\uDE00\"");
        let res = parser.parse_value().unwrap();
        assert_eq!(
            res,
            JsonNode::String("😀".to_string(), Span { start: 0, end: 14 })
        );
    }

    /// **Test Case**: Number Formats Parsing
    ///
    /// ### Description
    /// Verifies successful parsing of various valid numeric formats (integers, decimals, and scientific exponents).
    ///
    /// ### Test Procedure
    /// 1. Test parsing `"0"`.
    /// 2. Test parsing `"0.1"`.
    /// 3. Test parsing `"123e4"`.
    ///
    /// ### Expected Result
    /// All cases return correct `JsonNode::Number` containing the correct float value, raw text representation, and span.
    #[test]
    fn test_parser_numbers() {
        let mut parser = Parser::new("0");
        assert_eq!(
            parser.parse_value().unwrap(),
            JsonNode::Number(0.0, "0".to_string(), Span { start: 0, end: 1 })
        );

        let mut parser = Parser::new("0.1");
        assert_eq!(
            parser.parse_value().unwrap(),
            JsonNode::Number(0.1, "0.1".to_string(), Span { start: 0, end: 3 })
        );

        let mut parser = Parser::new("123e4 ");
        assert_eq!(
            parser.parse_value().unwrap(),
            JsonNode::Number(1230000.0, "123e4".to_string(), Span { start: 0, end: 5 })
        );
    }

    /// **Test Case**: Empty Array and Object Parsing
    ///
    /// ### Description
    /// Verifies that empty arrays `[]` and empty objects `{}` are correctly parsed with precise spans.
    ///
    /// ### Test Procedure
    /// 1. Test parsing `"[]"`.
    /// 2. Test parsing `"{}"`.
    ///
    /// ### Expected Result
    /// Returns correct empty container nodes with spans starting at 0 and ending at 2.
    #[test]
    fn test_parser_empty_array_and_object() {
        let mut parser = Parser::new("[]");
        assert_eq!(
            parser.parse_value().unwrap(),
            JsonNode::Array(vec![], Span { start: 0, end: 2 })
        );

        let mut parser = Parser::new("{}");
        assert_eq!(
            parser.parse_value().unwrap(),
            JsonNode::Object(vec![], Span { start: 0, end: 2 })
        );
    }
}
