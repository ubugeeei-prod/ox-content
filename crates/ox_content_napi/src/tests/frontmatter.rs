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
