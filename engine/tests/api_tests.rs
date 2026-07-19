use jsonette_core::{
    FoldingStyle, FormatOptions, JsonNode, LineEnding, Span, format, minify, parse,
};
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

/// **Test Case**: Default Format Options
///
/// ### Description
/// Verifies that `FormatOptions` initializes with the correct default JSON formatting choices.
///
/// ### Test Procedure
/// 1. Instantiate `FormatOptions::default()`.
/// 2. Assert values of each setting (tabs, indent size, line ending, folding style, key sorting, spacing).
///
/// ### Expected Result
/// Default settings are: 2-space indentation, no tabs, LF line endings, expanded folding style, no key sorting, and spaces enabled after colons and commas.
#[test]
fn test_format_options_default() {
    let opts = FormatOptions::default();
    assert!(!opts.use_tabs);
    assert_eq!(opts.indent, 2);
    assert_eq!(opts.line_ending, LineEnding::LF);
    assert_eq!(opts.folding_style, FoldingStyle::Expanded);
    assert!(!opts.sort_keys);
    assert!(opts.space_after_colon);
    assert!(opts.space_after_comma);
}

/// **Test Case**: JSON AST Node Helpers
///
/// ### Description
/// Verifies the helper functions and variants of the `JsonNode` AST node struct.
///
/// ### Test Procedure
/// 1. Construct `JsonNode` variants (`Null`, `Bool`, `Number`, `String`, `Array`, `Object`) with a dummy `Span`.
/// 2. Query each node's `node_type()` name and its `span()`.
///
/// ### Expected Result
/// Each helper returns the correct type name string and matched span coordinates.
#[test]
fn test_json_node_helpers() {
    let span = Span { start: 5, end: 10 };

    let node_null = JsonNode::Null(span.clone());
    assert_eq!(node_null.node_type(), "null");
    assert_eq!(node_null.span(), span);

    let node_bool = JsonNode::Bool(true, span.clone());
    assert_eq!(node_bool.node_type(), "bool");
    assert_eq!(node_bool.span(), span);

    let node_num = JsonNode::Number(42.0, "42".to_string(), span.clone());
    assert_eq!(node_num.node_type(), "number");
    assert_eq!(node_num.span(), span);

    let node_str = JsonNode::String("hello".to_string(), span.clone());
    assert_eq!(node_str.node_type(), "string");
    assert_eq!(node_str.span(), span);

    let node_arr = JsonNode::Array(vec![], span.clone());
    assert_eq!(node_arr.node_type(), "array");
    assert_eq!(node_arr.span(), span);

    let node_obj = JsonNode::Object(vec![], span.clone());
    assert_eq!(node_obj.node_type(), "object");
    assert_eq!(node_obj.span(), span);
}

/// **Test Case**: Parse Null Literal
///
/// ### Description
/// Verifies parsing a standard JSON `null` literal.
///
/// ### Test Procedure
/// 1. Parse the string `"null"`.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Null)` with a span of `0..4`.
#[test]
fn test_parse_null() {
    let input = "null";
    let res = parse(input).unwrap();
    assert_eq!(res, JsonNode::Null(Span { start: 0, end: 4 }));
}

/// **Test Case**: Parse Boolean Literals
///
/// ### Description
/// Verifies parsing standard JSON `true` and `false` boolean literals.
///
/// ### Test Procedure
/// 1. Parse string `"true"`.
/// 2. Parse string `"false"`.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Bool)` containing matching boolean state and span coordinates.
#[test]
fn test_parse_bool() {
    let input = "true";
    let res = parse(input).unwrap();
    assert_eq!(res, JsonNode::Bool(true, Span { start: 0, end: 4 }));

    let input_false = "false";
    let res_false = parse(input_false).unwrap();
    assert_eq!(res_false, JsonNode::Bool(false, Span { start: 0, end: 5 }));
}

/// **Test Case**: Parse Number Literals
///
/// ### Description
/// Verifies parsing numbers with floating points and exponents.
///
/// ### Test Procedure
/// 1. Parse string `"123.45e-2"`.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Number)` representing the evaluated float value `1.2345`, keeping the original raw text.
#[test]
fn test_parse_number() {
    let input = "123.45e-2";
    let res = parse(input).unwrap();
    assert_eq!(
        res,
        JsonNode::Number(1.2345, "123.45e-2".to_string(), Span { start: 0, end: 9 })
    );
}

/// **Test Case**: Parse String Literals
///
/// ### Description
/// Verifies parsing strings with unicode escape codes (`\u263a` representing `☺`).
///
/// ### Test Procedure
/// 1. Parse string `"\"hello \\u263a world\""`.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::String)` containing the unescaped UTF-8 string `"hello ☺ world"`.
#[test]
fn test_parse_string() {
    let input = "\"hello \\u263a world\"";
    let res = parse(input).unwrap();
    assert_eq!(
        res,
        JsonNode::String("hello ☺ world".to_string(), Span { start: 0, end: 20 })
    );
}

/// **Test Case**: Parse Array Literals
///
/// ### Description
/// Verifies parsing an array containing mixed types of child elements.
///
/// ### Test Procedure
/// 1. Parse array `"[1, true, null, \"hello\"]"`.
/// 2. Assert child types, float values, boolean states, and nested spans.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Array)` containing the 4 elements at exact inner offset spans.
#[test]
fn test_parse_array() {
    let input = "[1, true, null, \"hello\"]";
    let res = parse(input).unwrap();
    if let JsonNode::Array(elements, span) = res {
        assert_eq!(span, Span { start: 0, end: 24 });
        assert_eq!(elements.len(), 4);
        assert_eq!(
            elements[0],
            JsonNode::Number(1.0, "1".to_string(), Span { start: 1, end: 2 })
        );
        assert_eq!(elements[1], JsonNode::Bool(true, Span { start: 4, end: 8 }));
        assert_eq!(elements[2], JsonNode::Null(Span { start: 10, end: 14 }));
        assert_eq!(
            elements[3],
            JsonNode::String("hello".to_string(), Span { start: 16, end: 23 })
        );
    } else {
        panic!("expected array");
    }
}

/// **Test Case**: Parse Object Literals
///
/// ### Description
/// Verifies parsing a standard key-value object containing primitive and array values.
///
/// ### Test Procedure
/// 1. Parse object `{"a": 1, "b": [false]}`.
/// 2. Query keys, values, and inner array item states.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Object)` with matching keys, values, and child span positions.
#[test]
fn test_parse_object() {
    let input = "{\"a\": 1, \"b\": [false]}";
    let res = parse(input).unwrap();
    if let JsonNode::Object(pairs, span) = res {
        assert_eq!(span, Span { start: 0, end: 22 });
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].key, "a");
        assert_eq!(
            pairs[0].value,
            JsonNode::Number(1.0, "1".to_string(), Span { start: 6, end: 7 })
        );
        assert_eq!(pairs[1].key, "b");
        if let JsonNode::Array(arr, _) = &pairs[1].value {
            assert_eq!(arr[0], JsonNode::Bool(false, Span { start: 15, end: 20 }));
        } else {
            panic!("expected array");
        }
    } else {
        panic!("expected object");
    }
}

/// **Test Case**: Parse Deeply Nested JSON
///
/// ### Description
/// Verifies that recursive-descent stack parsing can handle deeply nested objects and arrays.
///
/// ### Test Procedure
/// 1. Parse nesting string `{"x": {"y": {"z": [1]}}}`.
///
/// ### Expected Result
/// Returns `Ok(JsonNode::Object)` containing parsed nested sub-structures.
#[test]
fn test_parse_deeply_nested() {
    let input = "{\"x\": {\"y\": {\"z\": [1]}}}";
    let res = parse(input).unwrap();
    assert_eq!(res.node_type(), "object");
}

/// **Test Case**: Parse Invalid JSON
///
/// ### Description
/// Verifies correct detection and reporting of structural errors (like a trailing comma in array).
///
/// ### Test Procedure
/// 1. Parse invalid array representation `{"key": [1, 2, ]}`.
///
/// ### Expected Result
/// Returns `Err` with a diagnostic pointing to the incorrect position inside the input.
#[test]
fn test_parse_invalid_json() {
    let input = "{\"key\": [1, 2, ]}";
    let res = parse(input);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(errs[0].span.start > 0);
}

/// **Test Case**: Parse Empty Input String
///
/// ### Description
/// Verifies parsing behavior when only whitespace is supplied.
///
/// ### Test Procedure
/// 1. Parse string `"   "`.
///
/// ### Expected Result
/// Returns `Err` indicating unexpected EOF/end of input.
#[test]
fn test_parse_empty_string() {
    let input = "   ";
    let res = parse(input);
    assert!(res.is_err());
}

/// **Test Case**: Multibyte Error Span
///
/// ### Description
/// Verifies that standard parser diagnostics correctly calculate the 0-indexed byte span
/// of a trailing character error immediately following a multi-byte UTF-8 character on the same line.
///
/// ### Test Procedure
/// 1. Parse raw JSON string `"\"é\"x"`.
/// 2. Extract returned diagnostic list.
///
/// ### Expected Result
/// Returns a single error diagnostic whose span exactly targets `4..5` (the character `x`).
#[test]
fn test_parse_multibyte_error_span() {
    let input = "\"é\"x";
    let res = parse(input);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].span.start, 4);
    assert_eq!(errs[0].span.end, 5);
}

/// **Test Case**: Multibyte Multiline Error Span
///
/// ### Description
/// Verifies that standard parser diagnostics correctly track byte offsets across multiple lines
/// when preceding lines contain multi-byte characters (emojis).
///
/// ### Test Procedure
/// 1. Parse multi-value JSON string containing `{"name": "😀"}\n[1, 2, ]`.
/// 2. Extract returned diagnostic list.
///
/// ### Expected Result
/// Returns an error diagnostic indicating trailing characters at the start of the second line (byte index 17).
#[test]
fn test_parse_multibyte_multiline_error_span() {
    let input = "{\"name\": \"😀\"}\n[1, 2, ]";
    let res = parse(input);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    // Line 1: {"name": "😀"}\n is 17 bytes:
    // '{' (1), '"name"' (6), ':' (1), ' ' (1), '"😀"' (6, with 4-byte emoji), '}' (1), '\n' (1).
    // Line 2: [1, 2, ]
    // Since there are multiple top-level values, serde_json reports trailing characters at line 2, column 1.
    // Line 2 starts at byte index 17.
    assert_eq!(errs[0].span.start, 17);
}

/// **Test Case**: Multibyte Error Inside Object
///
/// ### Description
/// Verifies that standard parser diagnostics correctly calculate the 0-indexed byte offset
/// when a syntax error (missing value for a key) occurs inside an object that contains multi-byte values.
///
/// ### Test Procedure
/// 1. Parse JSON string containing an object with a multibyte value on the previous line and a key with no value on the next.
/// 2. Extract returned diagnostic list.
///
/// ### Expected Result
/// Returns an error diagnostic indicating a missing value, pointing to the closing brace '}' at byte offset 26.
#[test]
fn test_parse_multibyte_error_inside_object() {
    // Error is key without value on line 3
    let input = "{\n  \"name\": \"😀\",\n  \"x\"\n}";
    let res = parse(input);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);

    // Line 1: {\n (2 bytes)
    // Line 2:   "name": "😀",\n (18 bytes: 2 spaces, '"name"' (6), ':' (1), ' ' (1), '"😀"' (6), ',' (1), '\n' (1))
    // Line 3 starts at byte index 20.
    // The closing brace '}' begins at byte index 26.
    // serde_json reports the error at line 3, column 7 (mapping to index 26).
    assert_eq!(errs[0].span.start, 26);
}

/// **Test Case**: Format and Minify Integration
///
/// ### Description
/// Verifies public API format and minify functions, including round-trip parity,
/// idempotence, and nested structure preservation.
#[test]
fn test_api_format_and_minify() {
    let _guard = TEST_LOCK.lock().unwrap();
    fn eq_ignore_span(a: &JsonNode, b: &JsonNode) -> bool {
        match (a, b) {
            (JsonNode::Null(_), JsonNode::Null(_)) => true,
            (JsonNode::Bool(x, _), JsonNode::Bool(y, _)) => x == y,
            (JsonNode::Number(x, rx, _), JsonNode::Number(y, ry, _)) => x == y && rx == ry,
            (JsonNode::String(x, _), JsonNode::String(y, _)) => x == y,
            (JsonNode::Array(x, _), JsonNode::Array(y, _)) => {
                if x.len() != y.len() {
                    return false;
                }
                for (elem_a, elem_b) in x.iter().zip(y.iter()) {
                    if !eq_ignore_span(elem_a, elem_b) {
                        return false;
                    }
                }
                true
            }
            (JsonNode::Object(x, _), JsonNode::Object(y, _)) => {
                if x.len() != y.len() {
                    return false;
                }
                for (pair_a, pair_b) in x.iter().zip(y.iter()) {
                    if pair_a.key != pair_b.key || !eq_ignore_span(&pair_a.value, &pair_b.value) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    let input = r#"{
  "name": "DevEtte",
  "version": 1.0,
  "unicode": "☺",
  "nested": {
    "list": [true, false, null]
  }
}"#;

    let parsed = parse(input).expect("parse input");

    // Preserve original global settings and ensure default settings are active
    let original_settings = jsonette_core::get_settings();
    let mut default_settings = original_settings;
    default_settings.format = FormatOptions::default();
    jsonette_core::update_settings(default_settings).expect("reset global settings");

    // Test default formatting using the global settings
    let formatted = format(&parsed);
    let parsed_formatted = parse(&formatted).expect("parse formatted");

    // Assert round-trip parsed structure matches exactly (ignoring spans)
    assert!(eq_ignore_span(&parsed, &parsed_formatted));

    // Test idempotence: format(format(x)) == format(x)
    let formatted_twice = format(&parsed_formatted);
    assert_eq!(formatted, formatted_twice);

    // Test minify round-trip
    let minified = minify(&parsed);
    assert_eq!(
        minified,
        r#"{"name":"DevEtte","version":1.0,"unicode":"☺","nested":{"list":[true,false,null]}}"#
    );

    // Test global settings manager singleton integration
    // 1. Current format output matches formatted_global
    let formatted_global = format(&parsed);
    assert_eq!(formatted, formatted_global);

    // 2. Change global settings via update_settings to use Compact folding
    let mut new_settings = default_settings;
    new_settings.format.folding_style = FoldingStyle::Compact;
    jsonette_core::update_settings(new_settings).expect("update global settings");

    // 3. Format -> should now format using the new Compact folding style
    let formatted_compact = format(&parsed);
    assert!(formatted_compact.contains("\"list\": [true, false, null]"));

    // Restore original global settings
    jsonette_core::update_settings(original_settings).expect("restore global settings");
}

/// **Test Case**: Parser Options Integration
///
/// ### Description
/// Verifies that parser settings for comments and trailing commas dynamically
/// affect strict parsing behavior.
#[test]
fn test_api_parser_options() {
    let _guard = TEST_LOCK.lock().unwrap();
    let original_settings = jsonette_core::get_settings();

    // --- Part 1: Comments ---
    let comment_json = r#"{
        // This is a line comment
        "key": "value", /* and a block comment */
        "number": 42
    }"#;

    // 1. Comments disabled by default -> should fail to parse
    let mut settings_no_comments = original_settings;
    settings_no_comments.parser.allow_comments = false;
    jsonette_core::update_settings(settings_no_comments).unwrap();
    assert!(parse(comment_json).is_err());

    // 2. Enable comments -> should parse successfully
    let mut settings_comments = original_settings;
    settings_comments.parser.allow_comments = true;
    jsonette_core::update_settings(settings_comments).unwrap();
    let parsed_comments = parse(comment_json).expect("should parse comments");
    assert_eq!(parsed_comments.node_type(), "object");

    // --- Part 2: Trailing Commas ---
    let trailing_comma_json = r#"{
        "array": [1, 2, 3,],
        "nested": {
            "key": "val",
        },
    }"#;

    // 1. Trailing commas disabled by default -> should fail to parse
    let mut settings_no_trailing = original_settings;
    settings_no_trailing.parser.allow_trailing_commas = false;
    jsonette_core::update_settings(settings_no_trailing).unwrap();
    assert!(parse(trailing_comma_json).is_err());

    // 2. Enable trailing commas -> should parse successfully
    let mut settings_trailing = original_settings;
    settings_trailing.parser.allow_trailing_commas = true;
    jsonette_core::update_settings(settings_trailing).unwrap();
    let parsed_trailing = parse(trailing_comma_json).expect("should parse trailing commas");
    assert_eq!(parsed_trailing.node_type(), "object");

    // Restore original settings
    jsonette_core::update_settings(original_settings).unwrap();
}
