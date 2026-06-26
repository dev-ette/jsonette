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

use crate::types::Span;

/// A key-value pair in a JSON object. Used to represent objects as lists of pairs
/// to preserve key ordering and stay FFI-friendly.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValuePair {
    /// The key of the property.
    pub key: String,
    /// The associated JSON node value.
    pub value: JsonNode,
}

/// The JSON value tree returned by a successful parse.
/// Each variant carries the byte-offset range (Span) from the original source code
/// for accurate syntax highlighting, navigation, and error reporting.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonNode {
    /// A JSON null value.
    Null(Span),
    /// A JSON boolean value (true or false).
    Bool(bool, Span),
    /// A JSON number, storing both its evaluated `f64` representation and
    /// its original raw string for lossless round-trip formatting.
    Number(f64, String, Span),
    /// A JSON string value.
    String(String, Span),
    /// A JSON array value.
    Array(Vec<JsonNode>, Span),
    /// A JSON object value, represented as a list of key-value pairs to preserve insertion order.
    Object(Vec<KeyValuePair>, Span),
}

impl JsonNode {
    /// Returns the type of the JSON node as a string.
    pub fn node_type(&self) -> String {
        match self {
            JsonNode::Null(_) => "null".to_string(),
            JsonNode::Bool(_, _) => "bool".to_string(),
            JsonNode::Number(_, _, _) => "number".to_string(),
            JsonNode::String(_, _) => "string".to_string(),
            JsonNode::Array(_, _) => "array".to_string(),
            JsonNode::Object(_, _) => "object".to_string(),
        }
    }

    /// Returns the span of the JSON node.
    pub fn span(&self) -> Span {
        match self {
            JsonNode::Null(span) => span.clone(),
            JsonNode::Bool(_, span) => span.clone(),
            JsonNode::Number(_, _, span) => span.clone(),
            JsonNode::String(_, span) => span.clone(),
            JsonNode::Array(_, span) => span.clone(),
            JsonNode::Object(_, span) => span.clone(),
        }
    }
}
