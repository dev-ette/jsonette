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

//! Property-based testing suite for the `jsonette` parser and formatter engine.
//!
//! Generates random, recursive valid JSON strings to verify core invariants
//! such as parser correctness, formatter idempotency, and minification equivalence.

use jsonette_core::{format, minify, parse};
use proptest::prelude::*;

/// Generates a strategy that produces arbitrary valid JSON strings.
///
/// Recursively builds objects, arrays, strings, numbers, booleans, and nulls
/// up to 8 levels deep with a maximum of 256 total nodes.
fn arb_json() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("null".to_string()),
        any::<bool>().prop_map(|b| b.to_string()),
        any::<i32>().prop_map(|n| n.to_string()),
        "[0-9a-zA-Z]{1,10}".prop_map(|s| format!("\"{}\"", s)),
    ];
    leaf.prop_recursive(
        8,   // 8 levels deep
        256, // max 256 nodes
        10,  // max 10 elements per array/object
        |inner| {
            prop_oneof![
                // Strategy for JSON Arrays
                prop::collection::vec(inner.clone(), 0..5)
                    .prop_map(|elements| format!("[{}]", elements.join(","))),
                // Strategy for JSON Objects
                prop::collection::vec(("[a-z]{1,5}", inner), 0..5).prop_map(|pairs| {
                    let kvs: Vec<String> = pairs
                        .into_iter()
                        .map(|(k, v)| format!("\"{}\":{}", k, v))
                        .collect();
                    format!("{{{}}}", kvs.join(","))
                })
            ]
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// **Test Case**: Parse Format Roundtrip
    ///
    /// ### Description
    /// For any valid JSON structure, parsing the formatted output results in the exact same AST structure (except spans).
    ///
    /// ### Test Procedure
    /// 1. Parse an arbitrary valid JSON document.
    /// 2. Format it.
    /// 3. Parse it again and format again.
    ///
    /// ### Expected Result
    /// The formatted strings identically match, proving the structure was preserved.
    #[test]
    fn test_parse_format_roundtrip(input in arb_json()) {
        let parsed = parse(&input).expect("Generated JSON must be valid");
        let formatted = format(&parsed);
        let parsed_formatted = parse(&formatted).expect("Formatted JSON must be valid");
        // Spans will differ between `parsed` and `parsed_formatted` because whitespace changes,
        // so we format them both and assert string equality, which is equivalent to tree structure equality
        assert_eq!(format(&parsed), format(&parsed_formatted));
    }

    /// **Test Case**: Formatter Idempotence
    ///
    /// ### Description
    /// For any valid JSON structure, `format(format(x)) == format(x)`.
    ///
    /// ### Test Procedure
    /// 1. Format a parsed JSON document.
    /// 2. Parse the output and format it again.
    ///
    /// ### Expected Result
    /// The output of the second format is byte-for-byte identical to the first.
    #[test]
    fn test_formatter_idempotence(input in arb_json()) {
        let parsed = parse(&input).expect("Generated JSON must be valid");
        let formatted_1 = format(&parsed);
        let parsed_1 = parse(&formatted_1).expect("Formatted JSON must be valid");
        let formatted_2 = format(&parsed_1);
        assert_eq!(formatted_1, formatted_2);
    }

    /// **Test Case**: Minify Parsability
    ///
    /// ### Description
    /// For any valid JSON structure, parsing minified output and minifying it again results in the exact same minified string.
    ///
    /// ### Test Procedure
    /// 1. Minify a parsed JSON document.
    /// 2. Parse the minified document and minify it again.
    ///
    /// ### Expected Result
    /// Both minified strings are identically matched.
    #[test]
    fn test_minify_parsability(input in arb_json()) {
        let parsed = parse(&input).expect("Generated JSON must be valid");
        let minified = minify(&parsed);
        let parsed_minified = parse(&minified).expect("Minified JSON must be parseable");
        assert_eq!(minify(&parsed_minified), minified);
    }
}
