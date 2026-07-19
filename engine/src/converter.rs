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

    /// Parses a string slice into a `DataFormat`.
    ///
    /// # Arguments
    ///
    /// * `s` - The string payload.
    ///
    /// # Returns
    ///
    /// Result containing the resolved format or an error.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **Test Case**: Data Format From Str
    ///
    /// ### Description
    /// Validates data format from str functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_data_format_from_str`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_data_format_from_str() {
        assert_eq!("json".parse::<DataFormat>().unwrap(), DataFormat::Json);
        assert_eq!("JSON".parse::<DataFormat>().unwrap(), DataFormat::Json);
        assert_eq!("yaml".parse::<DataFormat>().unwrap(), DataFormat::Yaml);
        assert_eq!("YML".parse::<DataFormat>().unwrap(), DataFormat::Yaml);
        assert_eq!("toml".parse::<DataFormat>().unwrap(), DataFormat::Toml);
        assert_eq!("xml".parse::<DataFormat>().unwrap(), DataFormat::Xml);
        assert!("invalid".parse::<DataFormat>().is_err());
    }

    /// **Test Case**: Convert Json To Others
    ///
    /// ### Description
    /// Validates convert json to others functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_convert_json_to_others`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_convert_json_to_others() {
        let json = r#"{"key": "value", "arr": [1, 2], "b": true, "n": null}"#;

        let yaml = convert(json, DataFormat::Json, DataFormat::Yaml).unwrap();
        assert!(yaml.contains("key: value"));

        let toml_json = r#"{"key": "value", "arr": [1, 2], "b": true}"#;
        let toml = convert(toml_json, DataFormat::Json, DataFormat::Toml).unwrap();
        assert!(toml.contains("key = \"value\""));

        let xml = convert(json, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<key>value</key>"));

        // json to json (formatting)
        let formatted = convert(json, DataFormat::Json, DataFormat::Json).unwrap();
        assert!(formatted.contains("\"key\": \"value\""));
    }

    /// **Test Case**: Convert Yaml To Others
    ///
    /// ### Description
    /// Validates convert yaml to others functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_convert_yaml_to_others`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_convert_yaml_to_others() {
        let yaml = "key: value\narr:\n  - 1\n  - 2";
        let json = convert(yaml, DataFormat::Yaml, DataFormat::Json).unwrap();
        assert!(json.contains("\"key\""));
    }

    /// **Test Case**: Convert Toml To Others
    ///
    /// ### Description
    /// Validates convert toml to others functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_convert_toml_to_others`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_convert_toml_to_others() {
        let toml = "key = \"value\"\narr = [1, 2]";
        let json = convert(toml, DataFormat::Toml, DataFormat::Json).unwrap();
        assert!(json.contains("\"key\""));

        // toml from array value
        let json_arr = r#"[1, 2]"#;
        let toml_out = convert(json_arr, DataFormat::Json, DataFormat::Toml).unwrap();
        assert!(toml_out.contains("data = ["));
    }

    /// **Test Case**: Convert Xml To Others
    ///
    /// ### Description
    /// Validates convert xml to others functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_convert_xml_to_others`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_convert_xml_to_others() {
        let xml = "<root><key>value</key><arr>1</arr><arr>2</arr><empty></empty></root>";
        let json = convert(xml, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json.contains("\"key\""));

        // array wrapped in root
        let json_arr = r#"[1, 2]"#;
        let xml_out = convert(json_arr, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml_out.contains("<item>1</item>"));

        // test parse error xml eof
        let xml_err = "<root><key>value</key>";
        assert!(convert(xml_err, DataFormat::Xml, DataFormat::Json).is_err());
    }

    /// **Test Case**: Convert Parse Errors
    ///
    /// ### Description
    /// Validates convert parse errors functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_convert_parse_errors`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_convert_parse_errors() {
        assert!(convert("invalid json", DataFormat::Json, DataFormat::Json).is_err());
        assert!(convert("{ invalid yaml", DataFormat::Yaml, DataFormat::Json).is_err());
        assert!(convert("invalid toml", DataFormat::Toml, DataFormat::Json).is_err());
    }

    /// **Test Case**: Write Json To Xml Cases
    ///
    /// ### Description
    /// Validates write json to xml cases functionality.
    ///
    /// ### Test Procedure
    /// 1. Execute `test_write_json_to_xml_cases`.
    ///
    /// ### Expected Result
    /// Completes successfully meeting all assertions.
    #[test]
    fn test_write_json_to_xml_cases() {
        // Test single object
        let json_obj = r#"{"a": 1}"#;
        let xml = convert(json_obj, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<a>1</a>"));

        // Test primitive string
        let json_str = r#""string""#;
        let xml = convert(json_str, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<root>string</root>"));

        // Test primitive number
        let json_num = r#"42"#;
        let xml = convert(json_num, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<root>42</root>"));

        // Test bool
        let json_bool = r#"true"#;
        let xml = convert(json_bool, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<root>true</root>"));

        // Test null
        let json_null = r#"null"#;
        let xml = convert(json_null, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<root/>"));

        // Test array with nested
        let json_complex = r#"{"item": [1, {"b": 2}]}"#;
        let xml = convert(json_complex, DataFormat::Json, DataFormat::Xml).unwrap();
        assert!(xml.contains("<item>1</item>"));
        assert!(xml.contains("<b>2</b>"));
    }

    /// **Test Case**: Parse XML Edge Cases
    ///
    /// ### Description
    /// Verifies the correct handling of complex or edge-case XML structures during parsing.
    ///
    /// ### Test Procedure
    /// 1. Convert an XML string with an empty element (`<empty/>`).
    /// 2. Convert an XML string with adjacent child tags of the same name.
    /// 3. Convert an XML string containing multiple root elements.
    /// 4. Convert XML sequences causing empty arrays and unexpected EOF errors.
    ///
    /// ### Expected Result
    /// The XML structures correctly evaluate to their respective fallback JSON forms (`null`, populated arrays/objects, or errors).
    #[test]
    fn test_parse_xml_edge_cases() {
        // Testing empty element conversion
        let xml = "<root><empty/></root>";
        let json = convert(xml, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json.contains("null"));

        let xml2 = "<root><k>1</k><k><empty/></k></root>";
        let json2 = convert(xml2, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json2.contains("\"k\""));

        // Testing unwrap single object
        let xml3 = "<item>1</item><item>2</item>";
        let json3 = convert(
            &format!("<root>{}</root>", xml3),
            DataFormat::Xml,
            DataFormat::Json,
        )
        .unwrap();
        assert!(json3.contains("["));

        // Testing numbers and bools in text
        let xml4 = "<root><n>42</n><b>true</b><f>false</f></root>";
        let json4 = convert(xml4, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json4.contains("42"));
        assert!(json4.contains("true"));
        assert!(json4.contains("false"));

        // Testing multiple root elements which hits lines 99-101
        let xml_multiple = "<root><a>1</a><b>2</b></root>";
        let json_multiple = convert(xml_multiple, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json_multiple.contains("\"a\": 1"));
        assert!(json_multiple.contains("\"b\": 2"));

        // Testing array existing handling in Start and Empty
        let xml_array_start = "<root><a>1</a><a>2</a><a>3</a></root>";
        let json_arr = convert(xml_array_start, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json_arr.contains("["));

        let xml_array_empty = "<root><a/><a/><a/></root>";
        let json_arr_empty = convert(xml_array_empty, DataFormat::Xml, DataFormat::Json).unwrap();
        assert!(json_arr_empty.contains("null"));

        // Testing EOF inside parse_xml_element
        let xml_eof = "<root><a>";
        assert!(convert(xml_eof, DataFormat::Xml, DataFormat::Json).is_err());

        // Testing Unexpected EOF at start
        let xml_empty = "";
        assert!(convert(xml_empty, DataFormat::Xml, DataFormat::Json).is_err());
    }
}
