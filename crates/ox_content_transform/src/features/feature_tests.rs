use super::*;

#[test]
fn wiki_links_become_markdown_links() {
    let options = ResolvedWikiLinkOptions { base_url: "/docs/".to_string() };
    let mut out = String::new();
    replace_wiki_links("See [[Guide Page#Install|the guide]].", &options, &mut out);
    assert_eq!(out, "See [the guide](/docs/Guide%20Page#install).");
}

#[test]
fn emoji_shortcodes_use_defaults_and_custom_values() {
    let options = ResolvedEmojiShortcodeOptions {
        custom: std::iter::once(("shipit".to_string(), "ship".to_string())).collect(),
    };
    let mut out = String::new();
    replace_emoji_shortcodes(":smile: :shipit: :octocat: :unknown:", &options, &mut out);
    assert_eq!(out, "\u{1F604} ship \u{1F431} :unknown:");
}

#[test]
fn extracts_docs_test_blocks_by_meta() {
    let blocks = extract_docs_tests(
        "```ts test\nexpect(1).toBe(1)\n```\n```js\nnoop()\n```",
        Some(&crate::DocsTestOptions {
            enabled: Some(true),
            languages: None,
            require_meta: Some(true),
        }),
    );
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].language, "ts");
}

#[test]
fn lints_code_block_trailing_spaces() {
    let diagnostics = lint_code_blocks(
        "```ts\nconst x = 1;  \n```",
        Some(&crate::CodeBlockLintOptions {
            enabled: Some(true),
            languages: None,
            require_language: Some(false),
            trailing_spaces: Some(true),
        }),
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].line, 2);
}
