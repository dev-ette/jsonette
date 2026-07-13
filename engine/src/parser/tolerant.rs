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

use crate::json_node::{JsonNode, KeyValuePair};
use crate::types::{Diagnostic, Span};

macro_rules! t_err {
    ($self:expr, $pos:expr, $msg:expr $(,)?) => {{
        let d = $self.error($pos, $msg);
        $self.diagnostics.push(d);
    }};
}

/// Tolerant parsing: Fails entirely if the JSON is invalid.
/// Returns the parsed tree or a list of diagnostic errors.
/// Primarily used for final validation.
///
/// # Arguments
///
/// * `input` - The raw JSON string slice to parse.
///
/// # Returns
///
/// * `Some(JsonNode)` - The parsed JSON abstract syntax tree (AST) on successful parse.
/// * `Err(Vec<Diagnostic>)` - A list of syntax or structural errors found during parsing.
pub fn parse(input: &str) -> (Option<JsonNode>, Vec<Diagnostic>) {
    let mut parser = Parser::new(input);
    let node = parser.parse_value();
    parser.skip_whitespace();
    if parser.cursor < parser.input.len() {
        let err = parser.error(
            parser.cursor,
            "Unexpected trailing characters after JSON value",
        );
        parser.diagnostics.push(err);
    }
    (node, parser.diagnostics)
}

struct Parser<'a> {
    pub diagnostics: Vec<Diagnostic>,
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
            diagnostics: Vec::new(),
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

    /// Returns the character byte at one position ahead of the current cursor, or `None` if EOF is reached.
    fn peek_next(&self) -> Option<u8> {
        if self.cursor + 1 < self.input.len() {
            Some(self.input[self.cursor + 1])
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

    /// Skips any ASCII whitespace characters (spaces, tabs, newlines, carriage returns)
    /// and single-line/multi-line comments if they are allowed in configuration.
    fn skip_whitespace(&mut self) {
        let allow_comments = crate::settings::get_settings().parser.allow_comments;
        loop {
            let start = self.cursor;
            // 1. Skip standard whitespace
            while let Some(b) = self.peek() {
                if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                    self.advance();
                } else {
                    break;
                }
            }
            // 2. Skip comments if enabled
            if allow_comments && self.peek() == Some(b'/') {
                match self.peek_next() {
                    Some(b'/') => {
                        // Line comment: skip until newline or EOF
                        self.advance(); // skip '/'
                        self.advance(); // skip '/'
                        while let Some(c) = self.peek() {
                            if c == b'\n' {
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }
                    Some(b'*') => {
                        // Block comment: skip until '*/' or EOF
                        self.advance(); // skip '/'
                        self.advance(); // skip '*'
                        while let Some(c) = self.peek() {
                            if c == b'*' && self.peek_next() == Some(b'/') {
                                self.advance(); // skip '*'
                                self.advance(); // skip '/'
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            if self.cursor == start {
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
    fn parse_value(&mut self) -> Option<JsonNode> {
        self.skip_whitespace();
        let start = self.cursor;
        let b = match self.peek() {
            Some(b) => b,
            None => {
                return {
                    t_err!(self, start, "Unexpected end of input");
                    None
                };
            }
        };

        match b {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'"' => self.parse_string_node(),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => {
                t_err!(self, start, format!("Unexpected character '{}'", b as char));
                None
            }
        }
    }

    /// Parses a JSON null value.
    fn parse_null(&mut self) -> Option<JsonNode> {
        let start = self.cursor;
        if self.cursor + 4 <= self.input.len()
            && &self.input[self.cursor..self.cursor + 4] == b"null"
        {
            self.cursor += 4;
            Some(JsonNode::Null(Span {
                start,
                end: self.cursor,
            }))
        } else {
            {
                t_err!(self, start, "Expected 'null'");
                None
            }
        }
    }

    /// Parses a JSON boolean value (true or false).
    fn parse_bool(&mut self) -> Option<JsonNode> {
        let start = self.cursor;
        if self.cursor + 4 <= self.input.len()
            && &self.input[self.cursor..self.cursor + 4] == b"true"
        {
            self.cursor += 4;
            Some(JsonNode::Bool(
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
            Some(JsonNode::Bool(
                false,
                Span {
                    start,
                    end: self.cursor,
                },
            ))
        } else {
            {
                t_err!(self, start, "Expected boolean value");
                None
            }
        }
    }

    /// Parses a raw string value, decoding escape characters and surrogate pairs,
    /// and returns the decoded string and its source span.
    fn parse_string_raw(&mut self) -> Option<(String, Span)> {
        let start = self.cursor;
        if self.peek() != Some(b'"') {
            return {
                t_err!(self, start, "Expected opening quote for string");
                None
            };
        }
        self.advance(); // consume opening quote

        let mut s = String::new();
        while let Some(b) = self.peek() {
            match b {
                b'"' => {
                    self.advance(); // consume closing quote
                    return Some((
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
                        None => {
                            return {
                                t_err!(self, self.cursor, "Unterminated string escape");
                                None
                            };
                        }
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
                                return {
                                    t_err!(self, self.cursor, "Invalid unicode escape sequence");
                                    None
                                };
                            }
                            let hex_str =
                                std::str::from_utf8(&self.input[self.cursor..self.cursor + 4])
                                    .map_err(|_| {
                                        let err = self
                                            .error(self.cursor, "Invalid utf-8 in unicode escape");
                                        self.diagnostics.push(err);
                                    })
                                    .ok()?;
                            let code_point = u16::from_str_radix(hex_str, 16)
                                .map_err(|_| {
                                    let err =
                                        self.error(self.cursor, "Invalid hex in unicode escape");
                                    self.diagnostics.push(err);
                                })
                                .ok()?;
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
                                        let err = self
                                            .error(self.cursor, "Invalid utf-8 in low surrogate");
                                        self.diagnostics.push(err);
                                    })
                                    .ok()?;
                                    let low_code_point = u16::from_str_radix(low_hex_str, 16)
                                        .map_err(|_| {
                                            let err = self
                                                .error(self.cursor, "Invalid hex in low surrogate");
                                            self.diagnostics.push(err);
                                        })
                                        .ok()?;
                                    self.cursor += 4;
                                    if (0xDC00..=0xDFFF).contains(&low_code_point) {
                                        let utf32 = (((code_point - 0xD800) as u32) << 10)
                                            + (low_code_point - 0xDC00) as u32
                                            + 0x10000;
                                        if let Some(c) = std::char::from_u32(utf32) {
                                            s.push(c);
                                        } else {
                                            return {
                                                t_err!(
                                                    self,
                                                    self.cursor - 12,
                                                    "Invalid surrogate pair",
                                                );
                                                None
                                            };
                                        }
                                    } else {
                                        return {
                                            t_err!(
                                                self,
                                                self.cursor - 6,
                                                "Expected low surrogate after high surrogate",
                                            );
                                            None
                                        };
                                    }
                                } else {
                                    return {
                                        t_err!(
                                            self,
                                            self.cursor,
                                            "Expected low surrogate after high surrogate",
                                        );
                                        None
                                    };
                                }
                            } else if (0xDC00..=0xDFFF).contains(&code_point) {
                                return {
                                    t_err!(
                                        self,
                                        self.cursor - 6,
                                        "Unexpected low surrogate without high surrogate",
                                    );
                                    None
                                };
                            } else {
                                if let Some(c) = std::char::from_u32(code_point as u32) {
                                    s.push(c);
                                } else {
                                    return {
                                        t_err!(self, self.cursor - 6, "Invalid unicode code point");
                                        None
                                    };
                                }
                            }
                        }
                        _ => {
                            return {
                                t_err!(
                                    self,
                                    self.cursor - 1,
                                    format!("Invalid escape character '{}'", esc as char),
                                );
                                None
                            };
                        }
                    }
                }
                b @ 0..=0x1f => {
                    return {
                        t_err!(self, self.cursor, "Control characters must be escaped");
                        None
                    };
                }
                _ => {
                    let tail = match std::str::from_utf8(&self.input[self.cursor..]) {
                        Ok(t) => t,
                        Err(_) => {
                            return {
                                t_err!(self, self.cursor, "Invalid UTF-8 sequence");
                                None
                            };
                        }
                    };
                    let c = match tail.chars().next() {
                        Some(ch) => ch,
                        None => {
                            return {
                                t_err!(self, self.cursor, "Unexpected EOF");
                                None
                            };
                        }
                    };
                    self.cursor += c.len_utf8();
                    s.push(c);
                }
            }
        }
        {
            t_err!(self, start, "Unterminated string");
            Some((
                s,
                Span {
                    start,
                    end: self.cursor,
                },
            ))
        }
    }

    /// Parses a JSON string value.
    fn parse_string_node(&mut self) -> Option<JsonNode> {
        let (s, span) = self.parse_string_raw()?;
        Some(JsonNode::String(s, span))
    }

    /// Parses a JSON number value.
    fn parse_number(&mut self) -> Option<JsonNode> {
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
            _ => {
                return {
                    t_err!(self, start, "Expected digit for number");
                    None
                };
            }
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

        Some(JsonNode::Number(val, raw_str, span))
    }

    /// Parses a JSON array value.
    fn parse_array(&mut self) -> Option<JsonNode> {
        let start = self.cursor;
        if self.peek() != Some(b'[') {
            {
                t_err!(self, start, "Expected '['");
                return None;
            }
        }
        self.advance(); // consume '['

        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.advance(); // consume ']'
            return Some(JsonNode::Array(
                vec![],
                Span {
                    start,
                    end: self.cursor,
                },
            ));
        }

        let mut elements = Vec::new();
        #[allow(clippy::while_let_loop)]
        loop {
            let val = if let Some(v) = self.parse_value() {
                v
            } else {
                break;
            };
            elements.push(val);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek() == Some(b']') {
                        if !crate::settings::get_settings().parser.allow_trailing_commas {
                            t_err!(self, self.cursor, "Trailing commas are not allowed in JSON");
                        }
                        self.advance();
                        break;
                    }
                }
                Some(b']') => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    t_err!(
                        self,
                        self.cursor,
                        format!(
                            "Expected ',' or ']' after array element, found '{}'",
                            b as char
                        )
                    );
                    break;
                }
                None => {
                    t_err!(self, self.cursor, "Unterminated array");
                    break;
                }
            }
        }

        Some(JsonNode::Array(
            elements,
            Span {
                start,
                end: self.cursor,
            },
        ))
    }

    /// Parses a JSON object value.
    fn parse_object(&mut self) -> Option<JsonNode> {
        let start = self.cursor;
        if self.peek() != Some(b'{') {
            {
                t_err!(self, start, "Expected '{'");
                return None;
            }
        }
        self.advance(); // consume '{'

        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.advance(); // consume '}'
            return Some(JsonNode::Object(
                vec![],
                Span {
                    start,
                    end: self.cursor,
                },
            ));
        }

        let mut pairs = Vec::new();
        #[allow(clippy::while_let_loop)]
        loop {
            self.skip_whitespace();
            let key_start = self.cursor;
            if self.peek() != Some(b'"') {
                {
                    t_err!(self, key_start, "Expected string key in object");
                    break;
                }
            }
            let (key, _) = if let Some(k) = self.parse_string_raw() {
                k
            } else {
                break;
            };

            self.skip_whitespace();
            let colon_pos = self.cursor;
            if self.peek() != Some(b':') {
                {
                    t_err!(self, colon_pos, "Expected ':' after key");
                    break;
                }
            }
            self.advance(); // consume ':'

            let val = if let Some(v) = self.parse_value() {
                v
            } else {
                break;
            };
            pairs.push(KeyValuePair { key, value: val });

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek() == Some(b'}') {
                        if !crate::settings::get_settings().parser.allow_trailing_commas {
                            t_err!(self, self.cursor, "Trailing commas are not allowed in JSON");
                        }
                        self.advance();
                        break;
                    }
                }
                Some(b'}') => {
                    self.advance();
                    break;
                }
                Some(b) => {
                    t_err!(
                        self,
                        self.cursor,
                        format!(
                            "Expected ',' or '}}' after object member, found '{}'",
                            b as char
                        )
                    );
                    break;
                }
                None => {
                    t_err!(self, self.cursor, "Unterminated object");
                    break;
                }
            }
        }

        Some(JsonNode::Object(
            pairs,
            Span {
                start,
                end: self.cursor,
            },
        ))
    }
}

#[cfg(test)]
mod tolerant_tests {
    use super::*;

    #[test]
    fn test_tolerant_dangling_value() {
        let (node, diagnostics) = parse(r#"{"a":"#);
        assert!(node.is_some());
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_tolerant_trailing_comma() {
        let (node, diagnostics) = parse(r#"{"a": 1,"#);
        assert!(node.is_some());
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_tolerant_unclosed_string() {
        let (node, diagnostics) = parse(r#"{"a": "unclosed"#);
        assert!(node.is_some());
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_tolerant_array_trailing_comma() {
        let (node, diagnostics) = parse(r#"[1, 2,"#);
        assert!(node.is_some());
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_tolerant_just_opening_brace() {
        let (node, diagnostics) = parse(r#"{"#);
        assert!(node.is_some());
        assert!(!diagnostics.is_empty());
    }
}
