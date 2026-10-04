use ox_content_markdown_lint::*;

fn options() -> MarkdownLintOptions {
    MarkdownLintOptions {
        rules: Some(MarkdownLintRuleOptions { spellcheck: Some(false), ..Default::default() }),
        ..Default::default()
    }
}
fn ids(source: &str, options: MarkdownLintOptions) -> Vec<String> {
    lint_markdown(source, Some(options)).diagnostics.into_iter().map(|v| v.rule_id).collect()
}

#[test]
fn headings_include_setext_inline_code_and_nested_blocks() {
    let source = "Title\n=====\n\n### Jump\n\n# `one`\n# `two`\n# **Title**\n\n> ### Nested\n";
    assert_eq!(
        ids(source, options()),
        ["heading-increment", "duplicate-heading", "heading-increment"]
    );
    assert_eq!(ids("#\n\n[]()\n", options()), ["empty-heading", "empty-link"]);
}

#[test]
fn strict_rules_are_individually_configurable() {
    let mut config = options();
    let mut rules = MarkdownLintRuleOptions::strict();
    rules.spellcheck = Some(false);
    config.rules = Some(rules);
    let source = "## Start\n\n# A\n# B\n\n![](/image.png)\n\n```\ncode\n```";
    assert_eq!(
        ids(source, config),
        ["first-heading-h1", "single-h1", "image-alt", "code-fence-language", "final-newline"]
    );
}

#[test]
fn code_comments_frontmatter_mdx_and_list_continuations_are_distinguished() {
    let source = "---\ntitle: TODO TODO!!\n---\n\n# Heading\n\n- Item\n\n    visible visible\n\n```rust\nrepeated repeated!!\n```   \n\nAfter after\n\nText <!-- hidden hidden --> visible visible\n";
    let result = lint_markdown(source, Some(options()));
    assert_eq!(
        result.diagnostics.iter().map(|v| &v.rule_id).collect::<Vec<_>>(),
        ["repeated-word", "repeated-word", "repeated-word"],
        "{result:?}"
    );
    let mut config = options();
    config.mdx = Some(true);
    assert_eq!(
        ids("import X from './X';\n\n<X value={1}>visible visible {user.name}</X>\n", config),
        ["repeated-word"]
    );
    assert!(ids("    code code!!\n\n```txt\ncode code!!\n```\n", options()).is_empty());
    assert_eq!(ids("```rust\nunclosed\n", options()), ["code-fence-closed"]);
}

#[test]
fn hard_breaks_crlf_and_safe_edits_preserve_semantics() {
    let source = "Hello  \r\nworld.\r\n\r\n\r\n😀 with with   \r\nnext.\r\n";
    let fixed = fix_markdown(source, Some(options()));
    assert_eq!(fixed.output, "Hello  \r\nworld.\r\n\r\n😀 with  \r\nnext.\r\n");
    assert_eq!(fixed.applied_fixes, 3);
    assert!(fixed.result.diagnostics.is_empty());
    assert_eq!(fix_markdown(&fixed.output, Some(options())).applied_fixes, 0);
    let result = lint_markdown("😀 with with\n", Some(options()));
    assert_eq!((result.diagnostics[0].column, result.diagnostics[0].end_column), (9, 13));
    let fix = result.diagnostics[0].fix.as_ref().unwrap();
    assert_eq!(&"😀 with with\n"[fix.start as usize..fix.end as usize], " with");
}

#[test]
fn punctuation_and_masked_syntax_do_not_create_duplicate_words() {
    assert!(ids("word, word; word `code` word. [word](/url) word\n", options()).is_empty());
    assert_eq!(ids("word word\n", options()), ["repeated-word"]);
    let result = lint_markdown(
        "[guide](https://example.com?TODO!!) https://example.com?TODO!!\n",
        Some(options()),
    );
    assert!(result.diagnostics.is_empty(), "{result:?}");
}

#[test]
fn textlint_inspired_rules_are_opt_in_and_span_soft_breaks() {
    let source = "短い、文、です。\n長い **文章** が\n続きます！ TODO\n";
    assert!(ids(source, options()).is_empty());
    let mut config = options();
    config.text_rules = Some(MarkdownLintTextRules {
        sentence_length: Some(10),
        max_ten: Some(1),
        no_exclamation_question_mark: Some(true),
        no_todo: Some(true),
        ..Default::default()
    });
    let result = lint_markdown(source, Some(config));
    assert_eq!(
        result.diagnostics.iter().map(|v| v.rule_id.as_str()).collect::<Vec<_>>(),
        ["max-ten", "sentence-length", "no-exclamation-question-mark", "no-todo"]
    );
    let sentence = &result.diagnostics[1];
    assert_eq!((sentence.line, sentence.end_line), (2, 3));
}

#[test]
fn prose_ignores_code_urls_expressions_and_attributes() {
    let mut config = options();
    config.mdx = Some(true);
    config.text_rules = Some(MarkdownLintTextRules {
        no_todo: Some(true),
        no_exclamation_question_mark: Some(true),
        sentence_length: Some(12),
        ..Default::default()
    });
    assert!(
        ids(
            "<Card title=\"TODO?!\">Good `TODO?!` {x.TODO} [Good](https://x?TODO!)</Card>\n",
            config.clone()
        )
        .is_empty()
    );
    let result =
        lint_markdown("[https://example.com?TODO!](https://example.com?TODO!)\n", Some(config));
    assert!(result.diagnostics.is_empty(), "{result:?}");
}

#[test]
fn terminology_respects_word_boundaries_and_never_edits_syntax() {
    let mut config = options();
    config.text_rules = Some(MarkdownLintTextRules {
        terminology: Some(vec![
            MarkdownLintTerm { term: "Javascript".into(), replacement: "JavaScript".into() },
            MarkdownLintTerm { term: "表記ゆれ".into(), replacement: "表記揺れ".into() },
        ]),
        ..Default::default()
    });
    let fixed = fix_markdown(
        "Javascript 表記ゆれ `Javascript` [Javascript](/Javascript) JavascriptX\n",
        Some(config),
    );
    assert_eq!(
        fixed.output,
        "JavaScript 表記揺れ `Javascript` [JavaScript](/Javascript) JavascriptX\n"
    );
    assert_eq!(fixed.applied_fixes, 3);
    assert!(fixed.result.diagnostics.is_empty());
}

#[test]
fn comment_directives_filter_rules_and_fixes_but_never_execute_inside_code() {
    let source = "<!-- oxlint-disable repeated-word -->\nwith with\n<!-- oxlint-enable repeated-word -->\nwith with\n<!-- oxlint-disable-next-line -->\nwith with\n\n```html\n<!-- oxlint-disable -->\n```\n\nwith with\n";
    let result = lint_markdown(source, Some(options()));
    assert_eq!(result.diagnostics.iter().map(|v| v.line).collect::<Vec<_>>(), [4, 12]);
    let fixed = fix_markdown(source, Some(options()));
    assert_eq!(fixed.applied_fixes, 2);
    assert!(fixed.output.contains("-->\nwith with"));
    let masked = lint_markdown("<!-- oxlint-disable-next-line spellcheck -->\nwrld\nwrld\n", None)
        .masked_document;
    assert_eq!(masked.lines().nth(1).unwrap(), "    ");
    assert_eq!(masked.lines().nth(2).unwrap(), "wrld");
}

#[test]
fn severity_controls_counts_exit_policy_and_fix_eligibility() {
    let mut config = options();
    config.severities = Some(vec![
        MarkdownLintRuleSeverity {
            rule_id: "repeated-word".into(),
            severity: MarkdownLintSeverity::Error,
        },
        MarkdownLintRuleSeverity {
            rule_id: "trailing-spaces".into(),
            severity: MarkdownLintSeverity::Off,
        },
    ]);
    let result = lint_markdown("with with \n", Some(config.clone()));
    assert_eq!((result.error_count, result.warning_count), (1, 0));
    assert_eq!(fix_markdown("with with \n", Some(config)).output, "with \n");
}

#[test]
fn parallel_batches_match_serial_order_and_diagnostics() {
    let sources: Vec<_> = (0..48).map(|i| format!("# Page {i}\n\nwith with\n")).collect();
    let config = options();
    let parallel = lint_markdown_documents(&sources, Some(config.clone()));
    let serial: Vec<_> = sources.iter().map(|s| lint_markdown(s, Some(config.clone()))).collect();
    for (a, b) in parallel.into_iter().zip(serial) {
        assert_eq!(serde_json::to_value(a).unwrap(), serde_json::to_value(b).unwrap());
    }
}

#[test]
fn entities_and_code_whitespace_are_excluded_from_prose_rules() {
    let mut config = options();
    config.text_rules =
        Some(MarkdownLintTextRules { sentence_length: Some(12), ..Default::default() });
    assert!(ids("Good `code          code` prose.\n", config).is_empty());
    let result = lint_markdown("Hello &amp; world.\n", None);
    assert!(result.diagnostics.is_empty(), "{result:?}");
}

#[test]
fn inline_html_does_not_hide_visible_prose_or_create_structural_fixes() {
    assert_eq!(ids("Text <span>with with</span>\n", options()), ["repeated-word"]);
    for replacement in ["- item", "1. item", "---", "", "  prose", "prose  ", "&amp;"] {
        let mut config = options();
        config.text_rules = Some(MarkdownLintTextRules {
            terminology: Some(vec![MarkdownLintTerm {
                term: "Original".into(),
                replacement: replacement.into(),
            }]),
            ..Default::default()
        });
        let fixed = fix_markdown("Original\n", Some(config));
        assert_eq!(fixed.applied_fixes, 0, "{replacement}");
        assert_eq!(fixed.output, "Original\n");
    }
    let fixed = fix_markdown("Prose.\n\n   \n", Some(options()));
    assert_eq!(fixed.output, "Prose.\n\n");
    assert!(fixed.result.diagnostics.is_empty());
}

#[test]
fn terminology_prefers_longest_match_and_config_accepts_severity_maps() {
    let config: MarkdownLintOptions = serde_json::from_str(r#"{"rules":{"spellcheck":false},"textRules":{"terminology":[{"term":"Java","replacement":"JVM"},{"term":"Javascript","replacement":"JavaScript"}]},"severities":{"terminology":"error"}}"#).unwrap();
    let fixed = fix_markdown("Javascript Java\n", Some(config));
    assert_eq!(fixed.output, "JavaScript JVM\n");
    assert_eq!(fixed.applied_fixes, 2);
    assert!(fixed.result.diagnostics.is_empty());
}
