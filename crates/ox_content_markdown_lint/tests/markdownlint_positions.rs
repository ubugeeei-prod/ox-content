//! Columns, severities and fix text for the native markdownlint rules.
//!
//! `fixtures/markdownlint.json` pins rule and line against markdownlint
//! 0.41.1. That format cannot express where on the line a diagnostic sits or
//! what a fix writes, so those are pinned here. Every range and every fixed
//! document in the two tables was also compared with upstream's `errorRange`
//! and `applyFixes` output and is identical.

use ox_content_markdown_lint::*;

fn options(markdownlint: &str) -> MarkdownLintOptions {
    serde_json::from_str(&format!(r#"{{"markdownlint":{markdownlint}}}"#)).unwrap()
}

/// `(rule, line, column, end column)`: one-based UTF-16 columns, end exclusive.
type Range = (&'static str, u32, u32, u32);

const RANGES: &[(&str, &str, &[Range])] = &[
    (
        "* a\n- b\n  + c\n",
        r#"{"default":false,"MD004":{"style":"asterisk"}}"#,
        &[("MD004", 2, 1, 2), ("MD004", 3, 3, 4)],
    ),
    (
        "text \nkeep  \nmore   \n",
        r#"{"default":false,"MD009":true}"#,
        &[("MD009", 1, 5, 6), ("MD009", 3, 5, 8)],
    ),
    (
        "👩‍💻 😀 \n日本語\u{3000}\n",
        r#"{"default":false,"MD009":true}"#,
        &[("MD009", 1, 9, 10), ("MD009", 2, 4, 5)],
    ),
    ("a \r\nb\t \r\n", r#"{"default":false,"MD009":true}"#, &[("MD009", 1, 2, 3)]),
    (
        "a\tb\tc\n日本\t語\n",
        r#"{"default":false,"MD010":true}"#,
        &[("MD010", 1, 2, 3), ("MD010", 1, 4, 5), ("MD010", 2, 3, 4)],
    ),
    (
        "see (text)[https://example.com] now\n",
        r#"{"default":false,"MD011":true}"#,
        &[("MD011", 1, 5, 32)],
    ),
    ("0123456789 x\n", r#"{"default":false,"MD013":{"line_length":10}}"#, &[("MD013", 1, 11, 13)]),
    (
        "😀😀😀😀😀 x y\n日本語日本語日本語日本 x\n",
        r#"{"default":false,"MD013":{"line_length":10}}"#,
        &[("MD013", 1, 11, 15), ("MD013", 2, 11, 14)],
    ),
    (
        "# What?\n\n## 見出し！！\n\n### Done. ###\n",
        r#"{"default":false,"MD026":{"punctuation":".?！"}}"#,
        &[("MD026", 1, 7, 8), ("MD026", 3, 7, 9), ("MD026", 5, 9, 10)],
    ),
    (
        "<DIV>x</DIV>\n\n日本 <Span>y</Span> <kbd>k</kbd>\n",
        r#"{"default":false,"MD033":{"allowed_elements":["div"]}}"#,
        &[("MD033", 3, 4, 10), ("MD033", 3, 19, 24)],
    ),
    (
        "<https://a.example> https://b.example and 日本 user@example.com\n",
        r#"{"default":false,"MD034":true}"#,
        &[("MD034", 1, 21, 38), ("MD034", 1, 46, 62)],
    ),
    (
        "** bold ** and * em * and __ u __\n",
        r#"{"default":false,"MD037":true}"#,
        &[
            ("MD037", 1, 3, 4),
            ("MD037", 1, 8, 9),
            ("MD037", 1, 17, 18),
            ("MD037", 1, 20, 21),
            ("MD037", 1, 29, 30),
            ("MD037", 1, 31, 32),
        ],
    ),
    (
        "` a` and `b ` and ` c ` and `` ` ``\n",
        r#"{"default":false,"MD038":true}"#,
        &[("MD038", 1, 2, 3), ("MD038", 1, 12, 13)],
    ),
    (
        "[ a](x) [b ](x) [ c ](x) [d](x)\n",
        r#"{"default":false,"MD039":true}"#,
        &[("MD039", 1, 2, 3), ("MD039", 1, 11, 12), ("MD039", 1, 18, 19), ("MD039", 1, 20, 21)],
    ),
    (
        "[a]() [b](#) [c](<>) [d](#frag)\n",
        r#"{"default":false,"MD042":true}"#,
        &[("MD042", 1, 1, 6), ("MD042", 1, 7, 13), ("MD042", 1, 14, 21)],
    ),
    (
        "日本 javascript, Github.com and `github`\n",
        r#"{"default":false,"MD044":{"names":["JavaScript","GitHub"]}}"#,
        &[("MD044", 1, 4, 14), ("MD044", 1, 16, 22), ("MD044", 1, 32, 38)],
    ),
    ("x ![](a.png) ![alt](b.png)\n", r#"{"default":false,"MD045":true}"#, &[("MD045", 1, 3, 13)]),
    ("# T\n\n日本語", r#"{"default":false,"MD047":true}"#, &[("MD047", 3, 3, 4)]),
    (
        "_a_ and *b* 日本 *c*\n",
        r#"{"default":false,"MD049":{"style":"underscore"}}"#,
        &[("MD049", 1, 9, 10), ("MD049", 1, 11, 12), ("MD049", 1, 16, 17), ("MD049", 1, 18, 19)],
    ),
    (
        "**a** and __b__\n",
        r#"{"default":false,"MD050":{"style":"asterisk"}}"#,
        &[("MD050", 1, 11, 13), ("MD050", 1, 14, 16)],
    ),
    (
        "# Title\n\n[a](#title) [b](#missing) [c](#Title)\n",
        r#"{"default":false,"MD051":true}"#,
        &[("MD051", 3, 13, 26), ("MD051", 3, 27, 38)],
    ),
    (
        "[a](x) <https://e.example> [b][r] [r]\n\n[r]: /r\n",
        r#"{"default":false,"MD054":{"inline":false,"autolink":false}}"#,
        &[("MD054", 1, 1, 7), ("MD054", 1, 8, 27)],
    ),
    (
        "| a | b |\n|---|---|\n| c  | d |\n",
        r#"{"default":false,"MD060":{"style":"compact"}}"#,
        &[
            ("MD060", 2, 1, 2),
            ("MD060", 2, 5, 6),
            ("MD060", 2, 5, 6),
            ("MD060", 2, 9, 10),
            ("MD060", 3, 6, 7),
        ],
    ),
];

#[test]
fn ranges_use_utf16_columns_and_stay_on_one_line() {
    for (source, config, expected) in RANGES {
        let result = lint_markdown(source, Some(options(config)));
        let actual: Vec<_> = result
            .diagnostics
            .iter()
            .map(|v| (v.rule_id.as_str(), v.line, v.column, v.end_column))
            .collect();
        assert_eq!(actual, *expected, "{source:?} with {config}");
        assert!(result.diagnostics.iter().all(|v| v.end_line == v.line), "{source:?}");
        assert_eq!(result.error_count as usize, expected.len(), "{source:?}");
    }
}

/// `(source, markdownlint config, fixed document, applied fixes)`.
const FIXES: &[(&str, &str, &str, u32)] = &[
    ("text \nmore   \n\nend\t\n", r#"{"default":false,"MD009":true}"#, "text\nmore\n\nend\n", 3),
    ("text  \nnext\n", r#"{"default":false,"MD009":{"br_spaces":0}}"#, "text\nnext\n", 1),
    (
        "# Title  \n\ntext  \nnext  \n",
        r#"{"default":false,"MD009":{"strict":true}}"#,
        "# Title\n\ntext  \nnext\n",
        2,
    ),
    (
        "```txt\ncode \n```\n",
        r#"{"default":false,"MD009":{"code_blocks":true}}"#,
        "```txt\ncode\n```\n",
        1,
    ),
    ("```txt\ncode \n```\n", r#"{"default":false,"MD009":true}"#, "```txt\ncode \n```\n", 0),
    ("👩‍💻 \r\n日本語\u{3000}\r\n", r#"{"default":false,"MD009":true}"#, "👩‍💻\r\n日本語\r\n", 2),
    ("a\n\n\n\n\nb\n", r#"{"default":false,"MD012":{"maximum":2}}"#, "a\n\n\nb\n", 2),
    ("a\r\n\r\n\r\n\r\nb\r\n", r#"{"default":false,"MD012":true}"#, "a\r\n\r\nb\r\n", 2),
    (
        "#  Two\n\n##   Three\n\n> #    Quoted\n",
        r#"{"default":false,"MD019":true}"#,
        "# Two\n\n## Three\n\n> # Quoted\n",
        3,
    ),
    ("#  Closed  #\n", r#"{"default":false,"MD021":true}"#, "# Closed #\n", 2),
    (
        "Use javascript and Github.com, not `javascript`.\n",
        r#"{"default":false,"MD044":{"names":["JavaScript","GitHub"]}}"#,
        "Use JavaScript and GitHub.com, not `JavaScript`.\n",
        3,
    ),
    (
        "Use javascript, not `javascript`.\n",
        r#"{"default":false,"MD044":{"names":["JavaScript"],"code_blocks":false}}"#,
        "Use JavaScript, not `javascript`.\n",
        1,
    ),
    ("# T\n\ntext", r#"{"default":false,"MD047":true}"#, "# T\n\ntext\n", 1),
    ("# T\r\n\r\ntext", r#"{"default":false,"MD047":true}"#, "# T\r\n\r\ntext\r\n", 1),
    (
        "#  Title  #\n\n\n\njavascript \n\n##   Sub\nend",
        r#"{"default":false,"MD009":true,"MD012":true,"MD019":true,"MD021":true,"MD044":{"names":["JavaScript"]},"MD047":true}"#,
        "# Title #\n\nJavaScript\n\n## Sub\nend\n",
        8,
    ),
    (
        "<!-- markdownlint-disable MD009 -->\ntext \n<!-- markdownlint-enable MD009 -->\nmore \n",
        r#"{"default":false,"MD009":true}"#,
        "<!-- markdownlint-disable MD009 -->\ntext \n<!-- markdownlint-enable MD009 -->\nmore\n",
        1,
    ),
    (
        "<!-- markdownlint-disable-next-line -->\n#  A\n\n#  B\n",
        r#"{"default":false,"MD019":true}"#,
        "<!-- markdownlint-disable-next-line -->\n#  A\n\n# B\n",
        1,
    ),
    ("text \n", r#"{"default":false,"MD009":"warning"}"#, "text\n", 1),
];

#[test]
fn fixes_write_exact_text_and_leave_nothing_fixable() {
    for (source, config, expected, applied) in FIXES {
        let linter = MarkdownLinter::new(Some(options(config)));
        let fixed = linter.fix(source);
        assert_eq!(fixed.output, *expected, "{source:?} with {config}");
        assert_eq!(fixed.applied_fixes, *applied, "{source:?} with {config}");
        assert!(fixed.result.diagnostics.iter().all(|v| v.fix.is_none()), "{:?}", fixed.result);
        assert_eq!(linter.fix(&fixed.output).applied_fixes, 0, "{source:?} with {config}");
    }
}

#[test]
fn rules_without_a_fix_stay_reported_next_to_applied_fixes() {
    let config = r#"{"default":false,"MD009":true,"MD019":true,"MD022":true,"MD026":true}"#;
    let fixed = fix_markdown("#  Title!\ntext \n", Some(options(config)));
    assert_eq!(fixed.output, "# Title!\ntext\n");
    assert_eq!(fixed.applied_fixes, 2);
    let remaining: Vec<_> =
        fixed.result.diagnostics.iter().map(|v| (v.rule_id.as_str(), v.line, v.column)).collect();
    assert_eq!(remaining, [("MD022", 1, 1), ("MD026", 1, 8)]);
    assert_eq!(fixed.result.error_count, 2);
}

#[test]
fn severity_follows_string_object_alias_and_tag_configuration() {
    for (config, severity) in [
        (r#"{"default":false,"MD009":true}"#, "error"),
        (r#"{"default":false,"MD009":{}}"#, "error"),
        (r#"{"default":false,"MD009":"error"}"#, "error"),
        (r#"{"default":false,"MD009":"warning"}"#, "warning"),
        (r#"{"default":false,"MD009":{"severity":"warning","br_spaces":0}}"#, "warning"),
        (r#"{"default":false,"no-trailing-spaces":"warning"}"#, "warning"),
        (r#"{"default":false,"whitespace":"warning"}"#, "warning"),
        (r#"{"default":false,"whitespace":"warning","MD009":"error"}"#, "error"),
        (r#"{"MD009":{"severity":"warning"},"MD041":false}"#, "warning"),
    ] {
        let result = lint_markdown("text \n", Some(options(config)));
        let found: Vec<_> =
            result.diagnostics.iter().map(|v| (v.rule_id.as_str(), v.severity.as_str())).collect();
        assert_eq!(found, [("MD009", severity)], "{config}");
        let counts = (result.error_count, result.warning_count, result.info_count);
        assert_eq!(counts, if severity == "error" { (1, 0, 0) } else { (0, 1, 0) }, "{config}");
    }
}

#[test]
fn the_severities_option_overrides_markdownlint_rules_by_identifier() {
    let run = |severities: &str| {
        let options: MarkdownLintOptions = serde_json::from_str(&format!(
            r#"{{"markdownlint":{{"default":false,"MD009":true}},"severities":{severities}}}"#
        ))
        .unwrap();
        let fixed = fix_markdown("text \n", Some(options.clone()));
        (lint_markdown("text \n", Some(options)), fixed.output, fixed.applied_fixes)
    };
    let (info, output, applied) = run(r#"{"MD009":"info"}"#);
    assert_eq!((info.error_count, info.warning_count, info.info_count), (0, 0, 1));
    assert_eq!(info.diagnostics[0].severity, "info");
    assert_eq!((output.as_str(), applied), ("text\n", 1));
    // A rule switched off reports nothing and therefore edits nothing.
    let (off, output, applied) = run(r#"{"MD009":"off"}"#);
    assert!(off.diagnostics.is_empty());
    assert_eq!((output.as_str(), applied), ("text \n", 0));
    // Overrides are keyed by the reported identifier, not by markdownlint aliases.
    let (alias, _, applied) = run(r#"{"no-trailing-spaces":"off"}"#);
    assert_eq!((alias.error_count, applied), (1, 1));
}

#[test]
fn messages_are_catalog_descriptions_and_identifiers_are_canonical() {
    let source = "## Sub!\ntext \n* a\n- b\n\n\n\n<b>x</b> https://example.com\n";
    for config in ["{}", "true", r#"{"default":true}"#] {
        let result = lint_markdown(source, Some(options(config)));
        assert!(result.diagnostics.len() >= 8, "{config}: {result:?}");
        for value in &result.diagnostics {
            let rule = markdownlint_rules().iter().find(|rule| rule.id == value.rule_id);
            assert_eq!(rule.map(|rule| rule.description), Some(value.message.as_str()));
            assert_eq!(value.severity, "error");
        }
    }
    // Aliases, tags and any letter case report under the MD identifier.
    for config in [
        r#"{"default":false,"md009":true}"#,
        r#"{"default":false,"No-Trailing-Spaces":true}"#,
        r#"{"default":false,"WHITESPACE":true}"#,
    ] {
        let result = lint_markdown("text \n", Some(options(config)));
        assert_eq!(result.diagnostics[0].rule_id, "MD009", "{config}");
    }
}

#[test]
fn no_inline_config_ignores_markdownlint_comments_but_not_oxlint_comments() {
    let source = "<!-- markdownlint-disable MD009 -->\ntext \n<!-- markdownlint-configure-file {\"MD009\":false} -->\n<!-- markdownlint-disable-file -->\n";
    let lines = |options: &str| {
        lint_markdown(source, Some(serde_json::from_str(options).unwrap()))
            .diagnostics
            .into_iter()
            .map(|v| (v.rule_id, v.line))
            .collect::<Vec<_>>()
    };
    assert!(lines(r#"{"markdownlint":{"default":false,"MD009":true}}"#).is_empty());
    assert!(
        lines(r#"{"markdownlint":{"default":false,"MD009":true},"noInlineConfig":false}"#)
            .is_empty()
    );
    assert_eq!(
        lines(r#"{"markdownlint":{"default":false,"MD009":true},"noInlineConfig":true}"#),
        [("MD009".to_string(), 2)]
    );
    // Ox Content's own comments filter markdownlint rules too and stay active.
    let source = "<!-- oxlint-disable MD009 -->\ntext \n<!-- oxlint-enable MD009 -->\ntext \n";
    for inline in [false, true] {
        let options = format!(
            r#"{{"markdownlint":{{"default":false,"MD009":true}},"noInlineConfig":{inline}}}"#
        );
        let result = lint_markdown(source, Some(serde_json::from_str(&options).unwrap()));
        let lines: Vec<_> = result.diagnostics.iter().map(|v| v.line).collect();
        assert_eq!(lines, [4], "noInlineConfig={inline}");
    }
}
