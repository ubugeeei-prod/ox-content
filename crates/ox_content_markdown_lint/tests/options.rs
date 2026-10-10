//! Native rule switches, severities, dictionaries and `oxlint-*` comments.

use ox_content_markdown_lint::*;
use serde_json::{Value, json};

/// Options with spellcheck off unless `extra` says otherwise.
fn options(rules: Value, extra: Value) -> MarkdownLintOptions {
    let mut merged = json!({ "spellcheck": false });
    merged.as_object_mut().unwrap().extend(rules.as_object().cloned().unwrap_or_default());
    let mut value = json!({ "rules": merged });
    value.as_object_mut().unwrap().extend(extra.as_object().cloned().unwrap_or_default());
    serde_json::from_value(value).unwrap()
}

/// `(rule, line, column, end line, end column)`.
type Range = (&'static str, u32, u32, u32, u32);

fn ranges(source: &str, options: MarkdownLintOptions) -> Vec<(String, u32, u32, u32, u32)> {
    lint_markdown(source, Some(options))
        .diagnostics
        .into_iter()
        .map(|v| (v.rule_id, v.line, v.column, v.end_line, v.end_column))
        .collect()
}

/// `(switch, on by default, source, diagnostics while on)`.
const SWITCHES: &[(&str, bool, &str, &[Range])] = &[
    ("duplicateHeadings", true, "# A\n\n## B\n\n## B\n", &[("duplicate-heading", 5, 1, 6, 1)]),
    ("headingIncrement", true, "# A\n\n### C\n", &[("heading-increment", 3, 1, 3, 4)]),
    (
        "repeatedPunctuation",
        true,
        "Wait!! What?? 本当！！ ok.. a,,b\n",
        &[
            ("repeated-punctuation", 1, 5, 1, 7),
            ("repeated-punctuation", 1, 12, 1, 14),
            ("repeated-punctuation", 1, 17, 1, 19),
        ],
    ),
    (
        "repeatedWords",
        true,
        "the the cat Cat\n",
        &[("repeated-word", 1, 5, 1, 8), ("repeated-word", 1, 13, 1, 16)],
    ),
    (
        "trailingSpaces",
        true,
        "a \nb\t\nc  \nd\n",
        &[("trailing-spaces", 1, 2, 1, 3), ("trailing-spaces", 2, 2, 2, 3)],
    ),
    (
        "emptyHeadings",
        true,
        "#\n\n## ![](x.png)\n\n### `code`\n",
        &[("empty-heading", 1, 1, 2, 1), ("empty-heading", 3, 1, 4, 1)],
    ),
    ("codeFenceClosed", true, "text\n\n```js\ncode\n", &[("code-fence-closed", 3, 1, 3, 6)]),
    (
        "emptyLinks",
        true,
        "[a]() [](b) [c](d) [ ]( )\n",
        &[("empty-link", 1, 1, 1, 6), ("empty-link", 1, 7, 1, 12), ("empty-link", 1, 20, 1, 26)],
    ),
    ("firstHeadingH1", false, "## A\n", &[("first-heading-h1", 1, 1, 2, 1)]),
    ("singleH1", false, "# A\n\n# B\n", &[("single-h1", 3, 1, 4, 1)]),
    (
        "codeFenceLanguage",
        false,
        "```\ncode\n```\n\n~~~\ncode\n~~~\n\n```js\nx\n```\n",
        &[("code-fence-language", 1, 1, 1, 4), ("code-fence-language", 5, 1, 5, 4)],
    ),
    (
        "imageAlt",
        false,
        "![](a.png) ![alt](b.png) ![ ](c.png)\n",
        &[("image-alt", 1, 1, 1, 11), ("image-alt", 1, 26, 1, 37)],
    ),
    ("finalNewline", false, "text", &[("final-newline", 1, 5, 1, 5)]),
];

#[test]
fn every_rule_switch_reports_at_its_documented_position_and_can_be_turned_off() {
    for (switch, default_on, source, expected) in SWITCHES {
        let expected: Vec<_> = expected
            .iter()
            .map(|(rule, a, b, c, d)| ((*rule).to_string(), *a, *b, *c, *d))
            .collect();
        let lint = |rules: Value| ranges(source, options(rules, json!({})));
        assert_eq!(lint(json!({ *switch: true })), expected, "{switch}");
        assert!(lint(json!({ *switch: false })).is_empty(), "{switch}");
        assert_eq!(lint(json!({})), if *default_on { expected } else { Vec::new() }, "{switch}");
    }
    // `strict()` is exactly the eight document-style switches.
    let strict = serde_json::to_value(MarkdownLintRuleOptions::strict()).unwrap();
    let strict = strict.as_object().unwrap();
    let mut enabled: Vec<_> =
        strict.iter().filter(|(_, v)| **v == true).map(|(k, _)| k.as_str()).collect();
    enabled.sort_unstable();
    assert_eq!(
        enabled.join(" "),
        "codeFenceClosed codeFenceLanguage emptyHeadings emptyLinks finalNewline firstHeadingH1 imageAlt singleH1"
    );
    assert!(strict.values().all(|v| v.is_null() || *v == true));
}

#[test]
fn the_blank_line_limit_is_a_threshold_with_matching_message_and_fix() {
    for (limit, source, message, lines, fixed) in [
        (json!(0), "a\n\nb\n", "More than 0 blank lines in a row.", vec![2], "a\nb\n"),
        (json!(1), "a\n\n\n\nb\n", "More than 1 blank line in a row.", vec![3, 4], "a\n\nb\n"),
        (json!(2), "a\n\n\n\nb\n", "More than 2 blank lines in a row.", vec![4], "a\n\n\nb\n"),
        (
            json!(null),
            "a\n\n\nb\r\n\r\n\r\nc\n",
            "More than 1 blank line in a row.",
            vec![3, 6],
            "a\n\nb\r\n\r\nc\n",
        ),
    ] {
        let config = options(json!({ "maxConsecutiveBlankLines": limit }), json!({}));
        let result = lint_markdown(source, Some(config.clone()));
        assert_eq!(result.diagnostics.iter().map(|v| v.line).collect::<Vec<_>>(), lines, "{limit}");
        for value in &result.diagnostics {
            assert_eq!(value.rule_id, "max-consecutive-blank-lines");
            assert_eq!((value.message.as_str(), value.column, value.end_column), (message, 1, 1));
        }
        let output = fix_markdown(source, Some(config));
        assert_eq!((output.output.as_str(), output.applied_fixes as usize), (fixed, lines.len()));
    }
    // Blank lines inside code are content, and fence bodies never count.
    let code = "```txt\n\n\n\n```\n\n    a\n\n\n\n    b\n";
    assert!(ranges(code, options(json!({}), json!({}))).is_empty());
}

#[test]
fn unknown_or_mistyped_configuration_is_rejected_at_every_level() {
    for invalid in [
        json!({ "rule": {} }),
        json!({ "rules": { "spellCheck": true } }),
        json!({ "rules": { "repeated_words": true } }),
        json!({ "rules": { "repeatedWords": "yes" } }),
        json!({ "rules": { "maxConsecutiveBlankLines": -1 } }),
        json!({ "rules": { "maxConsecutiveBlankLines": 1.5 } }),
        json!({ "text_rules": {} }),
        json!({ "textRules": { "sentenseLength": 10 } }),
        json!({ "textRules": { "sentenceLength": -1 } }),
        json!({ "textRules": { "noTodo": 1 } }),
        json!({ "textRules": { "terminology": { "term": "a", "replacement": "b" } } }),
        json!({ "textRules": { "terminology": [{ "term": "a" }] } }),
        json!({ "textRules": { "terminology": [{ "term": "a", "replacement": "b", "regex": true }] } }),
        json!({ "dictionary": { "word": [] } }),
        json!({ "dictionary": { "byLanguage": { "en": ["wrld"] } } }),
        json!({ "dictionary": { "byLanguage": [{ "language": "en" }] } }),
        json!({ "dictionary": { "byLanguage": [{ "language": "en", "words": [], "x": 1 }] } }),
        json!({ "severities": "error" }),
        json!({ "severities": { "repeated-word": "fatal" } }),
        json!({ "severities": { "repeated-word": "Error" } }),
        json!({ "severities": [{ "rule": "repeated-word", "severity": "error" }] }),
        json!({ "severities": [{ "ruleId": "repeated-word" }] }),
        json!({ "languages": "en" }),
        json!({ "mdx": "yes" }),
        json!({ "noInlineConfig": 1 }),
        json!({ "no_inline_config": true }),
        json!([]),
    ] {
        assert!(
            serde_json::from_value::<MarkdownLintOptions>(invalid.clone()).is_err(),
            "{invalid}"
        );
    }
    for valid in [
        json!({}),
        json!({ "severities": null, "rules": null, "textRules": null, "dictionary": null }),
        json!({ "severities": {} }),
        json!({ "severities": [] }),
        json!({ "languages": [], "mdx": false, "noInlineConfig": false, "markdownlint": null }),
        json!({ "textRules": { "terminology": [] }, "dictionary": { "byLanguage": [] } }),
    ] {
        assert!(serde_json::from_value::<MarkdownLintOptions>(valid.clone()).is_ok(), "{valid}");
    }
}

#[test]
fn severities_accept_maps_and_entry_lists_and_drive_counts_and_fixes() {
    let source = "the the \n";
    let map = json!({ "severities": { "repeated-word": "info", "trailing-spaces": "error" } });
    let list = json!({ "severities": [
        { "ruleId": "repeated-word", "severity": "info" },
        { "ruleId": "trailing-spaces", "severity": "error" }
    ] });
    let results: Vec<_> = [map, list]
        .into_iter()
        .map(|extra| lint_markdown(source, Some(options(json!({}), extra))))
        .collect();
    assert_eq!(
        serde_json::to_value(&results[0]).unwrap(),
        serde_json::to_value(&results[1]).unwrap()
    );
    let found: Vec<_> =
        results[0].diagnostics.iter().map(|v| (v.rule_id.as_str(), v.severity.as_str())).collect();
    assert_eq!(found, [("repeated-word", "info"), ("trailing-spaces", "error")]);
    let counts = (results[0].error_count, results[0].warning_count, results[0].info_count);
    assert_eq!(counts, (1, 0, 1));
    // Informational diagnostics are still fixed; switched-off rules are not.
    let fix = |severities: Value| {
        let fixed =
            fix_markdown(source, Some(options(json!({}), json!({ "severities": severities }))));
        (fixed.output, fixed.applied_fixes)
    };
    assert_eq!(fix(json!({ "repeated-word": "info" })), ("the\n".to_string(), 2));
    assert_eq!(fix(json!({ "repeated-word": "off" })), ("the the\n".to_string(), 1));
    assert_eq!(fix(json!({ "repeated-word": "off", "trailing-spaces": "off" })).1, 0);
    // Entries for rules that do not exist change nothing.
    let unknown = lint_markdown(
        source,
        Some(options(json!({}), json!({ "severities": { "no-such-rule": "error" } }))),
    );
    assert_eq!((unknown.error_count, unknown.warning_count, unknown.info_count), (0, 2, 0));
}

#[test]
fn spellcheck_languages_fall_back_to_english_and_dictionaries_extend_it() {
    let source = "Hello wrld\n";
    let lint = |extra: Value| {
        let mut value = json!({ "dictionary": { "words": ["Hello"] } });
        for (key, entry) in extra.as_object().cloned().unwrap_or_default() {
            match (value.get_mut(&key).and_then(Value::as_object_mut), entry.as_object()) {
                (Some(target), Some(entry)) => target.extend(entry.clone()),
                _ => value[key] = entry,
            }
        }
        lint_markdown(source, Some(serde_json::from_value(value).unwrap())).diagnostics
    };
    // Unsupported, empty and duplicated language lists all mean English, once.
    for languages in [json!(null), json!([]), json!(["xx", "tlh"]), json!(["en", "en", "xx"])] {
        let found = lint(json!({ "languages": languages }));
        assert_eq!(found.len(), 1, "{languages}");
        let value = &found[0];
        assert_eq!(
            (value.rule_id.as_str(), value.line, value.column, value.end_column),
            ("spellcheck", 1, 7, 11)
        );
        assert_eq!(value.message, "Unknown en word \"wrld\".");
        assert_eq!(value.language.as_deref(), Some("en"));
        assert_eq!(
            value.suggestions.as_deref().and_then(<[String]>::first).map(String::as_str),
            Some("world")
        );
        assert!(value.fix.is_none(), "spelling suggestions are never applied");
    }
    assert!(lint(json!({ "languages": ["ja"] })).is_empty());
    for known in [
        json!({ "dictionary": { "words": ["Hello", "WRLD"] } }),
        json!({ "dictionary": { "ignoredWords": ["Wrld"] } }),
        json!({ "dictionary": { "byLanguage": [{ "language": "en", "words": ["wrld"] }] } }),
        json!({ "rules": { "spellcheck": false } }),
        json!({ "severities": { "spellcheck": "off" } }),
    ] {
        assert!(lint(known.clone()).is_empty(), "{known}");
    }
    // Words for an unsupported or inactive language do not leak into English.
    for language in ["xx", "fr"] {
        let extra = json!({ "dictionary": { "byLanguage": [{ "language": language, "words": ["wrld"] }] } });
        assert_eq!(lint(extra).len(), 1, "{language}");
    }
}

#[test]
fn terminology_skips_inert_entries_and_reports_unsafe_replacements_without_editing() {
    let terms = |terms: Value| options(json!({}), json!({ "textRules": { "terminology": terms } }));
    let inert = terms(json!([
        { "term": "", "replacement": "anything" },
        { "term": "Same", "replacement": "Same" }
    ]));
    assert!(lint_markdown("Same Javascript\n", Some(inert)).diagnostics.is_empty());
    let unsafe_terms = terms(json!([{ "term": "Original", "replacement": "*New*" }]));
    let fixed = fix_markdown("Original text\n", Some(unsafe_terms));
    assert_eq!((fixed.output.as_str(), fixed.applied_fixes), ("Original text\n", 0));
    let value = &fixed.result.diagnostics[0];
    assert_eq!((value.rule_id.as_str(), value.column, value.end_column), ("terminology", 1, 9));
    assert_eq!(value.message, "Use \"*New*\" instead of \"Original\".");
    assert_eq!(value.suggestions.as_deref(), Some(&["*New*".to_string()][..]));
    assert!(value.fix.is_none());
}

#[test]
fn oxlint_comments_scope_rules_by_line_and_name() {
    let lines = |source: &str| -> Vec<(String, u32)> {
        ranges(source, options(json!({}), json!({}))).into_iter().map(|v| (v.0, v.1)).collect()
    };
    let word = |line| ("repeated-word".to_string(), line);
    let space = |line| ("trailing-spaces".to_string(), line);
    for (source, expected) in [
        (
            "<!-- oxlint-disable repeated-word, trailing-spaces -->\nthe the \n<!-- oxlint-enable trailing-spaces -->\nthe the \n",
            vec![space(4)],
        ),
        (
            "<!-- oxlint-disable-next-line trailing-spaces -->\nthe the \nthe the \n",
            vec![word(2), word(3), space(3)],
        ),
        ("the the <!-- oxlint-disable-next-line -->\nthe the\nthe the\n", vec![word(1), word(3)]),
        ("<!-- oxlint-enable -->\nthe the\n", vec![word(2)]),
        ("<!--\n  oxlint-disable repeated-word\n-->\nthe the \n", vec![space(4)]),
        ("<!-- oxlint-disable unknown-rule -->\nthe the\n", vec![word(2)]),
        ("<!-- oxlint-bogus -->\nthe the\n", vec![word(2)]),
        // Code is content: a comment shown as an example never changes linting.
        ("`<!-- oxlint-disable -->`\n\nthe the\n", vec![word(3)]),
        ("    <!-- oxlint-disable -->\n\nthe the\n", vec![word(3)]),
        ("~~~html\n<!-- oxlint-disable -->\n~~~\n\nthe the\n", vec![word(5)]),
    ] {
        assert_eq!(lines(source), expected, "{source:?}");
    }
}

#[test]
fn the_markdownlint_setting_selects_which_rule_family_runs() {
    let source = "## Sub\ntext \n\n\n\nthe the\n";
    let ids = |value: Value| -> Vec<String> {
        lint_markdown(source, Some(serde_json::from_value(value).unwrap()))
            .diagnostics
            .into_iter()
            .map(|v| v.rule_id)
            .collect()
    };
    let native = ids(json!({ "rules": { "spellcheck": false } }));
    assert_eq!(
        native,
        [
            "trailing-spaces",
            "max-consecutive-blank-lines",
            "max-consecutive-blank-lines",
            "repeated-word"
        ]
    );
    assert_eq!(ids(json!({ "markdownlint": false, "rules": { "spellcheck": false } })), native);
    for enabled in [json!(true), json!({})] {
        let found = ids(json!({ "markdownlint": enabled }));
        assert!(found.len() >= 4 && found.iter().all(|id| id.starts_with("MD")), "{found:?}");
    }
    // The profile replaces the native structure rules; prose rules stay opt-in.
    assert!(ids(json!({ "markdownlint": { "default": false } })).is_empty());
    assert_eq!(
        ids(json!({ "markdownlint": { "default": false }, "rules": { "repeatedWords": true } })),
        ["repeated-word"]
    );
}
