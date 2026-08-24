#[test]
fn mdx_flag_reaches_parse_and_transform_bindings() {
    let parsed = crate::parse(
        "import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n".to_string(),
        Some(crate::JsParserOptions { mdx: Some(true), ..Default::default() }),
    );
    let ast: serde_json::Value = serde_json::from_str(&parsed.ast).unwrap();
    assert_eq!(ast["children"][0]["type"], "mdxjsEsm");
    assert_eq!(ast["children"][1]["type"], "mdxJsxFlowElement");

    let transformed = crate::transform(
        "import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n".to_string(),
        Some(crate::JsTransformOptions { mdx: Some(true), ..Default::default() }),
    );
    assert!(transformed.errors.is_empty());
    assert!(!transformed.html.contains("import Alert"));
    assert!(transformed.html.contains("data-ox-island=\"Alert\""));
}
