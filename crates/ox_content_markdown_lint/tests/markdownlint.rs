use ox_content_markdown_lint::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    source: String,
    config: MarkdownlintConfig,
    expected: Vec<(String, u32)>,
}

#[test]
fn markdownlint_0411_conformance() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/markdownlint.json")).unwrap();
    let mut failures = Vec::new();
    for case in cases {
        let linter = MarkdownLinter::new(Some(MarkdownLintOptions {
            markdownlint: Some(case.config),
            ..Default::default()
        }));
        let mut actual: Vec<_> = linter
            .lint_without_mask(&case.source)
            .diagnostics
            .into_iter()
            .map(|diagnostic| (diagnostic.rule_id, diagnostic.line))
            .collect();
        actual.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
        if actual != case.expected {
            failures.push(format!("{}: expected {:?}, got {:?}", case.name, case.expected, actual));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn configuration_rejects_unknown_rules_options_and_invalid_regex() {
    for invalid in [
        r#"{"MD002":true}"#,
        r#"{"MD013":{"lenght":80}}"#,
        r#"{"MD048":{"style":"spaces"}}"#,
        r#"{"MD051":{"ignored_pattern":"("}}"#,
    ] {
        assert!(serde_json::from_str::<MarkdownlintConfig>(invalid).is_err(), "{invalid}");
    }
    assert_eq!(markdownlint_rules().len(), 53);
}

#[test]
fn markdownlint_fixes_preserve_hard_breaks_and_crlf() {
    let options: MarkdownLintOptions = serde_json::from_str(
        r#"{"markdownlint":{"default":false,"MD009":true,"MD012":true,"MD047":true}}"#,
    )
    .unwrap();
    let source = "😀 Text   \r\nnext.\r\n\r\n\r\nEnd";
    let fixed = fix_markdown(source, Some(options.clone()));
    assert_eq!(fixed.output, "😀 Text  \r\nnext.\r\n\r\nEnd\r\n");
    assert_eq!(fixed.applied_fixes, 3);
    assert!(fixed.result.diagnostics.is_empty());
    assert_eq!(fix_markdown(&fixed.output, Some(options)).applied_fixes, 0);
}

#[test]
fn prose_is_opt_in_with_markdownlint() {
    let options: MarkdownLintOptions =
        serde_json::from_str(r#"{"markdownlint":{"default":false}}"#).unwrap();
    assert!(lint_markdown("TODO with with!!\n", Some(options)).diagnostics.is_empty());
    let options: MarkdownLintOptions =
        serde_json::from_str(r#"{"markdownlint":{"default":false},"textRules":{"noTodo":true}}"#)
            .unwrap();
    assert_eq!(
        lint_markdown("TODO with with!!\n", Some(options)).diagnostics[0].rule_id,
        "no-todo"
    );
}
