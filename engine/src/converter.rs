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

//! Data conversion module for JSON ↔ YAML, TOML, XML.
//!
//! Uses `serde_json::Value` as the intermediate data model to translate between
//! various structured data formats. When converting TO JSON, the output is formatted
//! using the core engine's `format` implementation to respect user settings.

use std::str::FromStr;

/// The supported data formats for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// XML format
    Xml,
}

impl FromStr for DataFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(DataFormat::Json),
            "yaml" | "yml" => Ok(DataFormat::Yaml),
            "toml" => Ok(DataFormat::Toml),
            "xml" => Ok(DataFormat::Xml),
            _ => Err(format!("Unsupported format: {}", s)),
        }
    }
}

/// Converts a string payload from one data format to another.
///
/// # Arguments
///
/// * `input` - The input string payload.
/// * `from` - The source format.
/// * `to` - The target format.
///
/// # Returns
///
/// The converted string or an error message.
pub fn convert(input: &str, from: DataFormat, to: DataFormat) -> Result<String, String> {
    // 1. Parse input to intermediate serde_json::Value
    let value: serde_json::Value = match from {
        DataFormat::Json => {
            serde_json::from_str(input).map_err(|e| format!("JSON Parse Error: {}", e))?
        }
        DataFormat::Yaml => {
            serde_yml::from_str(input).map_err(|e| format!("YAML Parse Error: {}", e))?
        }
        DataFormat::Toml => {
            toml::from_str(input).map_err(|e| format!("TOML Parse Error: {}", e))?
        }
        DataFormat::Xml => {
            quick_xml::de::from_str(input).map_err(|e| format!("XML Parse Error: {}", e))?
        }
    };

    // 2. Serialize value to target format
    match to {
        DataFormat::Json => {
            let raw_json = serde_json::to_string(&value)
                .map_err(|e| format!("JSON Serialize Error: {}", e))?;
            // Re-parse and format using our engine to respect configuration rules
            match crate::parse(&raw_json) {
                Ok(node) => Ok(crate::format(&node)),
                Err(_) => Ok(raw_json), // fallback to raw
            }
        }
        DataFormat::Yaml => {
            serde_yml::to_string(&value).map_err(|e| format!("YAML Serialize Error: {}", e))
        }
        DataFormat::Toml => {
            let toml_value = if value.is_array() {
                let mut map = serde_json::Map::new();
                map.insert("data".to_string(), value.clone());
                serde_json::Value::Object(map)
            } else {
                value.clone()
            };
            toml::to_string_pretty(&toml_value).map_err(|e| format!("TOML Serialize Error: {}", e))
        }
        DataFormat::Xml => {
            // Wrap the value in a root element so quick-xml can serialize it
            #[derive(serde::Serialize)]
            struct XmlRoot<'a> {
                #[serde(rename = "$value")]
                value: &'a serde_json::Value,
            }
            quick_xml::se::to_string_with_root("root", &value)
                .or_else(|_| quick_xml::se::to_string(&XmlRoot { value: &value }))
                .map_err(|e| format!("XML Serialize Error: {}", e))
        }
    }
}
