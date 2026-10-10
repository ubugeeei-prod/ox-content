//! Rule selection through `oxct lint` flags and Ox Content configuration.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

/// `(ruleId, line, column)` reported for a single document.
#[track_caller]
fn lint(source: &str, config: Option<&str>, flags: &[&str]) -> Vec<(String, u64, u64)> {
    let project = Project::new();
    project.write("a.md", source);
    let mut args = vec!["lint", "a.md", "--format", "json"];
    if let Some(config) = config {
        project.write("lint.json", config);
        args.extend(["--config", "lint.json"]);
    }
    args.extend(flags);
    project
        .run(&args)
        .diagnostics()
        .into_iter()
        .map(|(_, rule, line, column)| (rule, line, column))
        .collect()
}

#[track_caller]
fn rules(source: &str, config: Option<&str>, flags: &[&str]) -> Vec<String> {
    lint(source, config, flags).into_iter().map(|(rule, ..)| rule).collect()
}

const STRUCTURE: &str = "## Start\n\n# A\n\n# B\n\n#\n\n[]()\n\n![](/i.png)\n\n```\ncode\n```";

#[test]
fn default_prose_profile_reports_only_broken_structure() {
    assert_eq!(
        lint(STRUCTURE, None, &["--no-markdownlint"]),
        [("empty-heading".into(), 7, 1), ("empty-link".into(), 9, 1)]
    );
}

#[test]
fn strict_adds_the_document_style_rules() {
    assert_eq!(
        lint(STRUCTURE, None, &["--no-markdownlint", "--strict"]),
        [
            ("first-heading-h1".into(), 1, 1),
            ("single-h1".into(), 5, 1),
            ("empty-heading".into(), 7, 1),
            ("single-h1".into(), 7, 1),
            ("empty-link".into(), 9, 1),
            ("image-alt".into(), 11, 1),
            ("code-fence-language".into(), 13, 1),
            ("final-newline".into(), 15, 4),
        ]
    );
}

#[test]
fn strict_never_overrides_an_explicit_rule_choice() {
    let config = r#"{"rules":{"firstHeadingH1":false,"imageAlt":false,"finalNewline":false}}"#;
    let reported = rules(STRUCTURE, Some(config), &["--strict"]);
    for disabled in ["first-heading-h1", "image-alt", "final-newline"] {
        assert!(!reported.contains(&disabled.to_string()), "{disabled}: {reported:?}");
    }
    for enabled in ["single-h1", "code-fence-language", "empty-heading", "empty-link"] {
        assert!(reported.contains(&enabled.to_string()), "{enabled}: {reported:?}");
    }
}

#[test]
fn each_structure_rule_can_be_enabled_alone() {
    for (rule, id) in [
        ("firstHeadingH1", "first-heading-h1"),
        ("singleH1", "single-h1"),
        ("imageAlt", "image-alt"),
        ("codeFenceLanguage", "code-fence-language"),
        ("finalNewline", "final-newline"),
    ] {
        let config =
            format!(r#"{{"rules":{{"{rule}":true,"emptyHeadings":false,"emptyLinks":false}}}}"#);
        let reported = rules(STRUCTURE, Some(&config), &[]);
        assert!(
            !reported.is_empty() && reported.iter().all(|found| found == id),
            "{rule}: {reported:?}"
        );
    }
}

#[test]
fn each_default_rule_can_be_disabled_alone() {
    let source = "# A\n\n### Jump\n\n# A\n\n\n\nwith with!! \n";
    let all = [
        "heading-increment",
        "duplicate-heading",
        "max-consecutive-blank-lines",
        "repeated-word",
        "repeated-punctuation",
        "trailing-spaces",
    ];
    let reported = rules(source, Some("{\"rules\":{}}"), &[]);
    for id in all {
        assert!(reported.contains(&id.to_string()), "{id}: {reported:?}");
    }
    for (rule, id) in [
        ("headingIncrement", "heading-increment"),
        ("duplicateHeadings", "duplicate-heading"),
        ("repeatedWords", "repeated-word"),
        ("repeatedPunctuation", "repeated-punctuation"),
        ("trailingSpaces", "trailing-spaces"),
    ] {
        let reported = rules(source, Some(&format!(r#"{{"rules":{{"{rule}":false}}}}"#)), &[]);
        let expected: Vec<_> = all.iter().filter(|other| **other != id).collect();
        for other in expected {
            assert!(reported.contains(&(*other).to_string()), "{rule} hid {other}");
        }
        assert!(!reported.contains(&id.to_string()), "{rule}: {reported:?}");
    }
}

#[test]
fn blank_line_budget_is_configurable() {
    let source = "a\n\n\nb\n\n\n\nc\n";
    let blank = |config: Option<&str>| {
        lint(source, config, &["--no-markdownlint"])
            .into_iter()
            .map(|(_, line, _)| line)
            .collect::<Vec<_>>()
    };
    assert_eq!(blank(None), [3, 6, 7]);
    assert_eq!(blank(Some(r#"{"rules":{"maxConsecutiveBlankLines":2}}"#)), [7]);
    assert!(blank(Some(r#"{"rules":{"maxConsecutiveBlankLines":3}}"#)).is_empty());
}

#[test]
fn spellcheck_is_opt_in_by_flag_or_configuration() {
    let source = "The wrld oxcontentish kolor.\n";
    let unknown = [
        ("spellcheck".to_string(), 1, 5),
        ("spellcheck".to_string(), 1, 10),
        ("spellcheck".to_string(), 1, 23),
    ];
    assert!(lint(source, None, &["--no-markdownlint"]).is_empty());
    assert!(!rules(source, None, &[]).contains(&"spellcheck".to_string()));
    assert_eq!(lint(source, None, &["--no-markdownlint", "--spellcheck"]), unknown);
    assert_eq!(lint(source, Some(r#"{"rules":{"spellcheck":true}}"#), &[]), unknown);
    // The flag only ever adds the rule.
    assert_eq!(lint(source, Some(r#"{"rules":{"spellcheck":false}}"#), &["--spellcheck"]), unknown);
}

#[test]
fn dictionaries_accept_global_language_and_ignored_words() {
    let source = "The wrld oxcontentish kolor.\n";
    for (dictionary, remaining) in [
        (r#"{"words":["wrld"]}"#, 2),
        (r#"{"ignoredWords":["kolor","wrld"]}"#, 1),
        (r#"{"words":["wrld"],"ignoredWords":["kolor"]}"#, 1),
        (r#"{"byLanguage":[{"language":"en","words":["wrld","kolor","oxcontentish"]}]}"#, 0),
        (r#"{"byLanguage":[{"language":"fr","words":["wrld","kolor","oxcontentish"]}]}"#, 3),
        (r#"{"words":["WRLD","Kolor","OxContentIsh"]}"#, 0),
    ] {
        let config = format!(r#"{{"rules":{{"spellcheck":true}},"dictionary":{dictionary}}}"#);
        assert_eq!(rules(source, Some(&config), &[]).len(), remaining, "{dictionary}");
    }
}

#[test]
fn unsupported_languages_fall_back_to_english() {
    let source = "The wrld.\n";
    for languages in [r#"["xx"]"#, "[]", r#"["en","xx"]"#] {
        let config = format!(r#"{{"languages":{languages},"rules":{{"spellcheck":true}}}}"#);
        assert_eq!(lint(source, Some(&config), &[]), [("spellcheck".into(), 1, 5)], "{languages}");
    }
}

#[test]
fn text_rules_are_off_until_configured_and_report_exact_positions() {
    let source = "日本語の文章です。これは、とても、長い、文、です。Hello!! TODO: fix\n";
    assert_eq!(rules(source, Some("{\"rules\":{}}"), &[]), ["repeated-punctuation"]);
    let config = r#"{"languages":["ja","en"],"textRules":{"sentenceLength":12,"maxTen":2,"noTodo":true,"noExclamationQuestionMark":true,"terminology":[{"term":"Hello","replacement":"Hi"}]}}"#;
    assert_eq!(
        lint(source, Some(config), &[]),
        [
            ("max-ten".into(), 1, 10),
            ("sentence-length".into(), 1, 10),
            ("terminology".into(), 1, 26),
            ("no-exclamation-question-mark".into(), 1, 31),
            ("repeated-punctuation".into(), 1, 31),
            ("no-exclamation-question-mark".into(), 1, 32),
            ("no-todo".into(), 1, 34),
        ]
    );
}

#[test]
fn each_text_rule_is_independent() {
    let source = "これは、とても、長い、文、です。Hello? TODO\n";
    for (rule, id) in [
        (r#""sentenceLength":8"#, "sentence-length"),
        (r#""maxTen":1"#, "max-ten"),
        (r#""noTodo":true"#, "no-todo"),
        (r#""noExclamationQuestionMark":true"#, "no-exclamation-question-mark"),
        (r#""terminology":[{"term":"Hello","replacement":"Hi"}]"#, "terminology"),
    ] {
        let config = format!(r#"{{"textRules":{{{rule}}}}}"#);
        assert_eq!(rules(source, Some(&config), &[]), [id], "{rule}");
    }
    let disabled =
        r#"{"textRules":{"noTodo":false,"noExclamationQuestionMark":false,"terminology":[]}}"#;
    assert!(rules(source, Some(disabled), &[]).is_empty());
}

#[test]
fn severities_rename_levels_for_prose_and_markdownlint_rules() {
    let source = "# Title\n\nwith with \n";
    let levels = |config: &str| {
        let project = Project::new();
        project.write("a.md", source);
        project.write("lint.json", config);
        let run = project.run(&["lint", "--config", "lint.json", "--format", "json"]);
        let report = run.json();
        let found: Vec<_> = report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                format!(
                    "{}={}",
                    entry["ruleId"].as_str().unwrap(),
                    entry["severity"].as_str().unwrap()
                )
            })
            .collect();
        (
            found,
            report["errorCount"].as_u64().unwrap(),
            report["warningCount"].as_u64().unwrap(),
            run.code,
        )
    };
    assert_eq!(
        levels(r#"{"severities":{"repeated-word":"error"}}"#),
        (vec!["repeated-word=error".into(), "trailing-spaces=warning".into()], 1, 1, Some(1))
    );
    // Informational findings are listed but never fail the run.
    assert_eq!(
        levels(r#"{"severities":{"repeated-word":"off","trailing-spaces":"info"}}"#),
        (vec!["trailing-spaces=info".into()], 0, 0, Some(0))
    );
    assert_eq!(
        levels(r#"{"markdownlint":true,"severities":{"MD009":"warning"}}"#),
        (vec!["MD009=warning".into()], 0, 1, Some(1))
    );
    assert_eq!(
        levels(r#"{"markdownlint":true,"severities":{"MD009":"off"}}"#),
        (vec![], 0, 0, Some(0))
    );
}

#[test]
fn markdownlint_comments_scope_rules_and_can_be_ignored() {
    let source = "# Title\n\n<!-- markdownlint-disable MD009 -->\ntext \n<!-- markdownlint-enable MD009 -->\ntext \n\ntext <!-- markdownlint-disable-line MD009 --> \n\n<!-- markdownlint-disable-next-line -->\ntext \ntext \n";
    assert_eq!(lint(source, None, &[]), [("MD009".into(), 6, 5), ("MD009".into(), 12, 5)]);
    let all: Vec<_> =
        lint(source, None, &["--no-inline-config"]).into_iter().map(|(_, line, _)| line).collect();
    assert_eq!(all, [4, 6, 8, 11, 12]);
    let config = r#"{"markdownlint":true,"noInlineConfig":true}"#;
    assert_eq!(lint(source, Some(config), &[]).len(), 5);
}

/// markdownlint scans raw lines, so a comment inside a code block still takes effect.
#[test]
fn markdownlint_comments_apply_even_inside_code_like_upstream() {
    let source = "# Title\n\n```html\n<!-- markdownlint-disable -->\n```\n\ntext \n";
    assert!(lint(source, None, &[]).is_empty());
    assert_eq!(lint(source, None, &["--no-inline-config"]), [("MD009".into(), 7, 5)]);
}

#[test]
fn prose_directives_scope_prose_rules() {
    let source = "<!-- oxlint-disable repeated-word -->\nwith with\n<!-- oxlint-enable repeated-word -->\nwith with\n\n<!-- oxlint-disable-next-line -->\nwith with \n\nwith with\n";
    assert_eq!(
        lint(source, None, &["--no-markdownlint"]),
        [("repeated-word".into(), 4, 6), ("repeated-word".into(), 9, 6)]
    );
}

#[test]
fn mdx_documents_are_linted_as_mdx_alongside_markdown() {
    let project = Project::new();
    let source = "import Card from './Card'\n\n<Card tone={theme.wrld} />\n";
    project.write("page.mdx", source);
    project.write("page.md", source);
    project.write("UPPER.MDX", source);
    let run = project.run(&[
        "lint",
        "page.mdx",
        "page.md",
        "UPPER.MDX",
        "--no-markdownlint",
        "--spellcheck",
        "--format",
        "json",
    ]);
    let files: Vec<_> = run.diagnostics().into_iter().map(|(file, ..)| file).collect();
    assert!(!files.is_empty() && files.iter().all(|file| file == "page.md"), "{files:?}");
}
