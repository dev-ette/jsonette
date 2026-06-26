use jsonette::{FoldingStyle, FormatOptions, JsonNode, LineEnding, Span};

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
