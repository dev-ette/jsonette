use jsonette::{FoldingStyle, FormatOptions, JsonNode, LineEnding, Span, parse};

#[test]
fn test_format_options_default() {
    let opts = FormatOptions::default();
    assert_eq!(opts.use_tabs, false);
    assert_eq!(opts.indent, 2);
    assert_eq!(opts.line_ending, LineEnding::LF);
    assert_eq!(opts.folding_style, FoldingStyle::Expanded);
    assert_eq!(opts.sort_keys, false);
    assert_eq!(opts.space_after_colon, true);
    assert_eq!(opts.space_after_comma, true);
}

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

#[test]
fn test_parse_null() {
    let input = "null";
    let res = parse(input).unwrap();
    assert_eq!(res, JsonNode::Null(Span { start: 0, end: 4 }));
}

#[test]
fn test_parse_bool() {
    let input = "true";
    let res = parse(input).unwrap();
    assert_eq!(res, JsonNode::Bool(true, Span { start: 0, end: 4 }));

    let input_false = "false";
    let res_false = parse(input_false).unwrap();
    assert_eq!(res_false, JsonNode::Bool(false, Span { start: 0, end: 5 }));
}

#[test]
fn test_parse_number() {
    let input = "123.45e-2";
    let res = parse(input).unwrap();
    assert_eq!(
        res,
        JsonNode::Number(1.2345, "123.45e-2".to_string(), Span { start: 0, end: 9 })
    );
}

#[test]
fn test_parse_string() {
    let input = "\"hello \\u263a world\"";
    let res = parse(input).unwrap();
    assert_eq!(
        res,
        JsonNode::String("hello ☺ world".to_string(), Span { start: 0, end: 20 })
    );
}

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

#[test]
fn test_parse_deeply_nested() {
    let input = "{\"x\": {\"y\": {\"z\": [1]}}}";
    let res = parse(input).unwrap();
    assert_eq!(res.node_type(), "object");
}

#[test]
fn test_parse_invalid_json() {
    let input = "{\"key\": [1, 2, ]}";
    let res = parse(input);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(errs[0].span.start > 0);
}

#[test]
fn test_parse_empty_string() {
    let input = "   ";
    let res = parse(input);
    assert!(res.is_err());
}
