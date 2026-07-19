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
            let mut reader = quick_xml::Reader::from_str(input);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();

            // Skip to root element
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Start(ref e)) => {
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        let inner = parse_xml_element(&mut reader, &mut buf)
                            .map_err(|err| format!("XML Parse Error: {}", err))?;

                        if name == "root" {
                            if let serde_json::Value::Object(ref map) = inner
                                && map.len() == 1
                                && map.contains_key("item")
                                && let Some(serde_json::Value::Array(arr)) = map.get("item")
                            {
                                break serde_json::Value::Array(arr.clone());
                            }
                            break inner;
                        } else {
                            let mut root = serde_json::Map::new();
                            root.insert(name, inner);
                            break serde_json::Value::Object(root);
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => {
                        return Err("XML Parse Error: Unexpected EOF".to_string());
                    }
                    Err(e) => return Err(format!("XML Parse Error: {}", e)),
                    _ => (), // Skip prolog, DTD, etc.
                }
            }
        }
    };

    // 2. Serialize value to target format
    match to {
        DataFormat::Json => {
            let raw_json = serde_json::to_string(&value)
                .map_err(|e| format!("JSON Serialize Error: {}", e))?;
            // Reparse and format using our engine to respect configuration rules
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
            let mut writer = quick_xml::Writer::new_with_indent(Vec::new(), b' ', 4);

            match &value {
                serde_json::Value::Object(map) if map.len() == 1 => {
                    let (k, v) = map.iter().next().unwrap();
                    if v.is_array() {
                        let start = quick_xml::events::BytesStart::new("root");
                        writer
                            .write_event(quick_xml::events::Event::Start(start.clone()))
                            .unwrap();
                        write_json_to_xml(&mut writer, k, v)
                            .map_err(|e| format!("XML Serialize Error: {}", e))?;
                        writer
                            .write_event(quick_xml::events::Event::End(start.to_end()))
                            .unwrap();
                    } else {
                        write_json_to_xml(&mut writer, k, v)
                            .map_err(|e| format!("XML Serialize Error: {}", e))?;
                    }
                }
                serde_json::Value::Array(_) => {
                    // Wrap arrays in a root element so we don't output multiple root elements
                    let start = quick_xml::events::BytesStart::new("root");
                    writer
                        .write_event(quick_xml::events::Event::Start(start.clone()))
                        .unwrap();
                    write_json_to_xml(&mut writer, "item", &value)
                        .map_err(|e| format!("XML Serialize Error: {}", e))?;
                    writer
                        .write_event(quick_xml::events::Event::End(start.to_end()))
                        .unwrap();
                }
                _ => {
                    write_json_to_xml(&mut writer, "root", &value)
                        .map_err(|e| format!("XML Serialize Error: {}", e))?;
                }
            }

            let pretty_xml = String::from_utf8(writer.into_inner()).unwrap();
            Ok(format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}",
                pretty_xml
            ))
        }
    }
}

/// Parses an XML element into a serde_json::Value.
///
/// # Arguments
///
/// * `reader` - The XML reader instance.
/// * `buf` - A mutable byte buffer used for reading events.
///
/// # Returns
///
/// The parsed JSON Value representing the XML element.
fn parse_xml_element(
    reader: &mut quick_xml::Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<serde_json::Value, String> {
    let mut map = serde_json::Map::new();
    let mut text_content = String::new();
    let mut has_children = false;

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                has_children = true;
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_val = parse_xml_element(reader, buf)?;

                if let Some(existing) = map.remove(&name) {
                    if let serde_json::Value::Array(mut arr) = existing {
                        arr.push(child_val);
                        map.insert(name, serde_json::Value::Array(arr));
                    } else {
                        map.insert(name, serde_json::Value::Array(vec![existing, child_val]));
                    }
                } else {
                    map.insert(name, child_val);
                }
            }
            Ok(quick_xml::events::Event::Empty(ref e)) => {
                has_children = true;
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some(existing) = map.remove(&name) {
                    if let serde_json::Value::Array(mut arr) = existing {
                        arr.push(serde_json::Value::Null);
                        map.insert(name, serde_json::Value::Array(arr));
                    } else {
                        map.insert(
                            name,
                            serde_json::Value::Array(vec![existing, serde_json::Value::Null]),
                        );
                    }
                } else {
                    map.insert(name, serde_json::Value::Null);
                }
            }
            Ok(quick_xml::events::Event::Text(e)) => {
                text_content.push_str(&String::from_utf8_lossy(e.as_ref()));
            }
            Ok(quick_xml::events::Event::End(_)) => {
                if !has_children {
                    if text_content.is_empty() {
                        return Ok(serde_json::Value::Null);
                    }
                    if let Ok(b) = text_content.parse::<bool>() {
                        return Ok(serde_json::Value::Bool(b));
                    }
                    if let Ok(n) = text_content.parse::<serde_json::Number>() {
                        return Ok(serde_json::Value::Number(n));
                    }
                    return Ok(serde_json::Value::String(text_content));
                }
                return Ok(serde_json::Value::Object(map));
            }
            Ok(quick_xml::events::Event::Eof) => return Err("Unexpected EOF".to_string()),
            Err(e) => return Err(e.to_string()),
            _ => (),
        }
    }
}

/// Writes a serde_json::Value as XML elements.
///
/// # Arguments
///
/// * `writer` - The XML writer instance.
/// * `name` - The tag name for the current element.
/// * `value` - The JSON Value to serialize.
///
/// # Returns
///
/// Empty result or a string error message on failure.
fn write_json_to_xml(
    writer: &mut quick_xml::Writer<Vec<u8>>,
    name: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    use quick_xml::events::{BytesStart, BytesText, Event};

    match value {
        serde_json::Value::Object(map) => {
            let start = BytesStart::new(name);
            writer
                .write_event(Event::Start(start.clone()))
                .map_err(|e| e.to_string())?;
            for (k, v) in map {
                write_json_to_xml(writer, k, v)?;
            }
            writer
                .write_event(Event::End(start.to_end()))
                .map_err(|e| e.to_string())?;
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                write_json_to_xml(writer, name, item)?;
            }
        }
        serde_json::Value::String(s) => {
            let start = BytesStart::new(name);
            writer
                .write_event(Event::Start(start.clone()))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(s)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(start.to_end()))
                .map_err(|e| e.to_string())?;
        }
        serde_json::Value::Number(n) => {
            let start = BytesStart::new(name);
            writer
                .write_event(Event::Start(start.clone()))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(&n.to_string())))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(start.to_end()))
                .map_err(|e| e.to_string())?;
        }
        serde_json::Value::Bool(b) => {
            let start = BytesStart::new(name);
            writer
                .write_event(Event::Start(start.clone()))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(&b.to_string())))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(start.to_end()))
                .map_err(|e| e.to_string())?;
        }
        serde_json::Value::Null => {
            let start = BytesStart::new(name);
            writer
                .write_event(Event::Empty(start))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
