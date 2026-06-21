use tower_lsp::lsp_types::DiagnosticSeverity;

use super::*;

const SAMPLE_PAYLOAD: &str = r#"[
  {
    "filePath": "doc.md",
    "messages": [
      {
        "ruleId": "ja-technical-writing/sentence-length",
        "message": "Line 1 sentence length(82) exceeds the maximum sentence length of 80.",
        "line": 1,
        "column": 1,
        "severity": 2
      },
      {
        "ruleId": "prh",
        "message": "use 'JavaScript' instead of 'Javascript'",
        "line": 4,
        "column": 12,
        "severity": 1
      }
    ]
  }
]"#;

#[test]
fn parses_messages_into_diagnostics_with_zero_indexed_positions() {
    let diagnostics = parse_diagnostics(SAMPLE_PAYLOAD);
    assert_eq!(diagnostics.len(), 2);

    let first = &diagnostics[0];
    assert_eq!(first.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(first.range.start.line, 0, "textlint line 1 → LSP line 0");
    assert_eq!(first.range.start.character, 0);
    assert_eq!(first.source.as_deref(), Some("textlint"));
    insta::assert_snapshot!(first.message);
    let code = first.code.as_ref().unwrap();
    let tower_lsp::lsp_types::NumberOrString::String(rule) = code else {
        panic!("expected a string rule id, got {code:?}");
    };
    assert_eq!(rule, "ja-technical-writing/sentence-length");

    let second = &diagnostics[1];
    assert_eq!(second.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(second.range.start.line, 3, "textlint line 4 → LSP line 3");
    assert_eq!(second.range.start.character, 11);
}

#[test]
fn empty_or_invalid_payloads_yield_no_diagnostics() {
    assert!(parse_diagnostics("").is_empty());
    assert!(parse_diagnostics("not json").is_empty());
    assert!(parse_diagnostics("[]").is_empty());
    assert!(parse_diagnostics(r#"[{"filePath": "x", "messages": []}]"#).is_empty());
}

#[test]
fn severity_other_than_one_or_two_maps_to_information() {
    let payload =
        r#"[{"filePath":"d","messages":[{"message":"info","line":1,"column":1,"severity":0}]}]"#;
    let diagnostics = parse_diagnostics(payload);
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::INFORMATION));
}

#[test]
fn missing_rule_id_is_tolerated() {
    let payload =
        r#"[{"filePath":"d","messages":[{"message":"plain","line":1,"column":1,"severity":1}]}]"#;
    let diagnostics = parse_diagnostics(payload);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].code.is_none());
}

#[test]
fn config_is_inactive_by_default() {
    let config = TextlintConfig::default();
    assert!(!config.is_active());
}

#[test]
fn config_with_empty_command_falls_back_to_npx_textlint() {
    let config = TextlintConfig { enabled: true, command: Some(String::new()) };
    assert_eq!(config.argv(), vec!["npx".to_string(), "textlint".into()]);
}

#[test]
fn config_with_command_splits_on_whitespace() {
    let config = TextlintConfig { enabled: true, command: Some("textlint --no-color".into()) };
    assert_eq!(config.argv(), vec!["textlint".to_string(), "--no-color".into()]);
}

#[test]
fn config_with_quoted_command_preserves_arg_with_spaces() {
    let config =
        TextlintConfig { enabled: true, command: Some(r#"my-runner "with space""#.into()) };
    assert_eq!(config.argv(), vec!["my-runner".to_string(), "with space".into()]);
}

#[test]
fn shlex_handles_mixed_single_and_double_quotes() {
    assert_eq!(
        shlex_split(r#"cmd 'a b' "c d" e"#),
        vec!["cmd".to_string(), "a b".into(), "c d".into(), "e".into()],
    );
}

#[tokio::test]
async fn run_returns_empty_when_disabled() {
    let config = TextlintConfig::default();
    let diagnostics = run("# heading\n", std::path::Path::new("/tmp/doc.md"), &config).await;
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn run_returns_empty_when_command_is_missing() {
    // Pretend the user typed a non-existent binary. The runner
    // must swallow the spawn failure and return an empty list so
    // a misconfigured textlint never floods the publish channel.
    let config =
        TextlintConfig { enabled: true, command: Some("/this/binary/does/not/exist".into()) };
    let diagnostics = run("# heading\n", std::path::Path::new("/tmp/doc.md"), &config).await;
    assert!(diagnostics.is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn run_parses_stdout_from_configured_command_even_when_it_exits_one() {
    use std::os::unix::fs::PermissionsExt;

    let mut script = std::env::temp_dir();
    script.push("ox-content-textlint-tests");
    std::fs::create_dir_all(&script).expect("create temp dir");
    script.push("fake-textlint.sh");
    std::fs::write(
        &script,
        r#"#!/bin/sh
cat >/dev/null
printf '%s\n' '[{"filePath":"doc.md","messages":[{"ruleId":"fixture/rule","message":"fixture message","line":2,"column":4,"severity":2}]}]'
exit 1
"#,
    )
    .expect("write fake textlint command");

    let mut permissions = std::fs::metadata(&script).expect("fake command metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script, permissions).expect("chmod fake textlint command");

    let config = TextlintConfig { enabled: true, command: Some(script.display().to_string()) };
    let diagnostics = run("# heading\n", std::path::Path::new("/tmp/doc.md"), &config).await;

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].source.as_deref(), Some("textlint"));
    assert_eq!(diagnostics[0].message, "fixture message");
    assert_eq!(diagnostics[0].range.start.line, 1);
    assert_eq!(diagnostics[0].range.start.character, 3);
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
}
