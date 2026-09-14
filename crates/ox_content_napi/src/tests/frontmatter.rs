use super::*;

#[test]
fn parses_nested_yaml_frontmatter() {
    let (content, frontmatter) = parse_frontmatter(
        "---\ntitle: Guide\nmeta:\n  tags:\n    - rust\n    - napi\n  draft: false\n---\n# Body",
    );

    assert_eq!(content, "# Body");
    assert_eq!(frontmatter.get("title"), Some(&json!("Guide")));
    assert_eq!(frontmatter.get("meta"), Some(&json!({"tags": ["rust", "napi"], "draft": false})));
}

#[test]
fn frontmatter_preserves_yaml_scalars_and_quoted_colons() {
    let (_, frontmatter) = parse_frontmatter(
        "---\ncount: 3\nratio: 1.5\ncanonical: \"https://example.com/a:b\"\n---\n",
    );

    assert_eq!(frontmatter.get("count"), Some(&json!(3)));
    assert_eq!(frontmatter.get("ratio"), Some(&json!(1.5)));
    assert_eq!(frontmatter.get("canonical"), Some(&json!("https://example.com/a:b")));
}

#[test]
fn malformed_yaml_strips_block_and_returns_empty_frontmatter() {
    let (content, frontmatter) = parse_frontmatter("---\ntitle: [broken\n---\nBody");

    assert_eq!(content, "Body");
    assert!(frontmatter.is_empty());
}

#[test]
fn parse_frontmatter_napi_returns_body_and_frontmatter() {
    let result = crate::parse_frontmatter_napi(
        "---\ntitle: Guide\nmeta:\n  draft: false\n---\n# Body".to_string(),
    );

    assert_eq!(result.content, "# Body");
    assert_eq!(result.frontmatter.get("title"), Some(&json!("Guide")));
    assert_eq!(result.frontmatter.get("meta"), Some(&json!({"draft": false})));
}

#[test]
fn parse_frontmatter_napi_preserves_yaml_key_order() {
    let result = crate::parse_frontmatter_napi(
        "---\npermalink: /blog/post\ntitle: Post\ndate: 2026-09-13\nisPublished: false\nlang: en\n---\n"
            .to_string(),
    );
    let keys = result.frontmatter.keys().map(String::as_str).collect::<Vec<_>>();

    assert_eq!(keys, ["permalink", "title", "date", "isPublished", "lang"]);
}

#[test]
fn stringify_frontmatter_napi_roundtrips_with_leading_blank_body_line() {
    let mut frontmatter = serde_json::Map::new();
    frontmatter.insert("permalink".to_string(), json!("/blog/post"));
    frontmatter.insert("title".to_string(), json!("Post"));

    let document = crate::stringify_frontmatter_napi(frontmatter, "\n# Post\n".to_string())
        .expect("frontmatter serialization should succeed");
    let result = crate::parse_frontmatter_napi(document);

    assert_eq!(result.content, "\n# Post\n");
    assert_eq!(result.frontmatter.get("permalink"), Some(&json!("/blog/post")));
    assert_eq!(result.frontmatter.get("title"), Some(&json!("Post")));
}

#[test]
fn stringify_frontmatter_napi_preserves_js_key_order() {
    let mut frontmatter = serde_json::Map::new();
    frontmatter.insert("permalink".to_string(), json!("/blog/post"));
    frontmatter.insert("title".to_string(), json!("Post"));
    frontmatter.insert("date".to_string(), json!("2026-09-13"));
    frontmatter.insert("isPublished".to_string(), json!(false));
    frontmatter.insert("lang".to_string(), json!("en"));

    let document = crate::stringify_frontmatter_napi(frontmatter, String::new())
        .expect("frontmatter serialization should succeed");

    assert_eq!(
        document,
        "---\npermalink: /blog/post\ntitle: Post\ndate: 2026-09-13\nisPublished: false\nlang: en\n---\n"
    );
}

#[test]
fn stringify_frontmatter_napi_preserves_body_without_frontmatter() {
    let document =
        crate::stringify_frontmatter_napi(serde_json::Map::new(), "\n# Body".to_string()).unwrap();

    assert_eq!(document, "\n# Body");
}
