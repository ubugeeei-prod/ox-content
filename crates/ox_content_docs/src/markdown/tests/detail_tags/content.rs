use super::*;

#[test]
fn detail_tags_resolve_links_in_prose_but_not_in_code() {
    let mut entry = greet();
    entry.tags = remarks(concat!(
        "See [Target] and `[Target]`.\n\n",
        "```ts\nconst a = [Target]\n// {@link Target}\n```\n\n",
        "~~~\n[Target]\n```\n[Target]\n~~~\n\n",
        "> Quoted [Target].\n>\n> ```ts\n> const q = [Target]\n> ```\n\n",
        "Indented:\n\n    indented [Target]\n\n",
        "- item [Target]\n\n  ```ts\n  const nested = [Target]\n  ```\n\n",
        "Then {@link Target} and {@link Target | `label`}.\n\n",
        "````md\n```\n[Target]\n```\n[Target]\n````\n\n",
        "```ts\nconst unclosed = [Target]",
    ));
    let target = test_entry("Target", "interface", "/repo/src/target.ts", "Target.");
    let page = page(&module(vec![entry, target]), &markdown_typedoc_options(), "greet");

    assert!(page.contains("See [Target](../interfaces/Target.md) and `[Target]`."));
    assert!(page.contains("const a = [Target]\n// {@link Target}\n"));
    assert!(page.contains("~~~\n[Target]\n```\n[Target]\n~~~"));
    assert!(page.contains("> Quoted [Target](../interfaces/Target.md)."));
    assert!(page.contains("> const q = [Target]\n"));
    assert!(page.contains("    indented [Target]\n"));
    assert!(page.contains("- item [Target](../interfaces/Target.md)"));
    assert!(page.contains("  const nested = [Target]\n"));
    assert!(page.contains("Then [Target](../interfaces/Target.md) and"));
    assert!(page.contains("[`label`](../interfaces/Target.md)."));
    assert!(page.contains("````md\n```\n[Target]\n```\n[Target]\n````"));
    assert!(page.contains("const unclosed = [Target]"));
    assert_eq!(page.matches("(../interfaces/Target.md)").count(), 5);
}

#[test]
fn an_unclosed_inline_tag_does_not_expose_later_code() {
    let mut entry = greet();
    entry.tags = remarks("- {@link Target\n\n`[Target]`");
    let target = test_entry("Target", "interface", "/repo/src/target.ts", "Target.");
    let page = page(&module(vec![entry, target]), &markdown_typedoc_options(), "greet");

    assert!(page.contains("`[Target]`"));
    assert!(!page.contains("`[Target]("));
}

#[test]
fn html_detail_tags_render_full_markdown() {
    let mut entry = greet();
    entry.tags = remarks(concat!(
        "> [!TIP]\n> A tip.\n\n",
        "1. outer\n   - nested\n2. second\n\n",
        "| a | b |\n| - | - |\n| 1 | 2 |\n\n",
        "```ts title=\"x.ts\"\nfirst()\n\nsecond()\n```\n\n",
        "~~~js\ntilde()\n```\nstill()\n~~~\n\n",
        "A snake_case_name, <script>alert(1)</script> and [bad](javascript:alert(1)).\n\n",
        "A note[^1].\n\n[^1]: Not a footnote.",
    ));
    let page = page(&module(vec![entry]), &html_typedoc_options(), "greet");
    let start = position(&page, "<h4>Remarks</h4>");
    let section = &page[start..position(&page, "<h4>Signature</h4>")];

    assert!(section.contains("<blockquote class=\"ox-callout ox-callout--tip\">"));
    assert!(section.contains("<li>outer\n<ul>\n<li>nested</li>\n</ul>\n</li>"));
    assert!(section.contains("<td>1</td>"));
    assert!(section.contains("<code class=\"language-ts\">first()&#10;\nsecond()\n</code>"));
    assert!(section.contains("<code class=\"language-js\">tilde()\n```\nstill()\n</code>"));
    assert!(section.contains("snake_case_name"));
    assert!(section.contains("&lt;script&gt;"));
    assert!(!section.contains("<script>"));
    assert!(!section.contains("javascript:"));
    assert!(!section.contains("id=\"fn-1\""));
    assert!(!section.contains("\n\n"));
}

#[test]
fn deep_detail_headings_stop_at_level_six() {
    let mut entry = test_entry("Maker", "interface", "/repo/src/maker.ts", "Makes it.");
    entry.members = vec![ApiDocMember {
        name: "make".to_string(),
        kind: "method".to_string(),
        signature: Some("make(): object".to_string()),
        returns: Some(ApiReturnDoc {
            type_annotation: "object".to_string(),
            members: vec![ApiDocMember {
                name: "value".to_string(),
                kind: "property".to_string(),
                type_annotation: Some("string".to_string()),
                tags: remarks("Deep remarks."),
                ..ApiDocMember::default()
            }],
            ..ApiReturnDoc::default()
        }),
        ..ApiDocMember::default()
    }];
    let options = MarkdownDocsOptions {
        render_style: MarkdownRenderStyle::Markdown,
        ..MarkdownDocsOptions::default()
    };
    let markdown = generate_markdown(&module(vec![entry]), &options);
    let page = markdown.values().find(|page| page.contains("Deep remarks.")).unwrap();

    assert!(page.contains("\n###### Remarks\n\nDeep remarks."));
}

#[test]
fn empty_detail_tags_render_nothing() {
    let mut entry = greet();
    entry.tags = details(" \n", "  \n ");

    let markdown = page(&module(vec![entry.clone()]), &markdown_typedoc_options(), "greet");
    assert!(!markdown.contains("## Description"));
    assert!(!markdown.contains("## Remarks"));
    assert!(!markdown.contains("## Tags"));
    let html = page(&module(vec![entry]), &html_typedoc_options(), "greet");
    assert!(!html.contains("<h4>Description</h4>"));
    assert!(!html.contains("<h4>Remarks</h4>"));
    assert!(!html.contains("<h4>Tags</h4>"));
}
