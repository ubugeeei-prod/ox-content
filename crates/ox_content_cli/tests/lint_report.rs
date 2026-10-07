//! The shape of `oxct lint` output, its exit status and its argument validation.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;
use serde_json::Value;

fn project() -> Project {
    let project = Project::new();
    project.write("a.md", "# Guide\n\nwith with \n");
    project.write("b.md", "# Guide\n\nClean prose.\n");
    project
}

/// Replace the only nondeterministic part of a text report.
fn stable(stdout: &str) -> String {
    let (summary, duration) = stdout.trim_end().rsplit_once(" · ").unwrap();
    let milliseconds = duration.strip_suffix("ms").unwrap();
    assert!(milliseconds.parse::<f64>().unwrap() >= 0.0, "{duration}");
    format!("{summary} · <duration>")
}

#[test]
fn text_report_lists_sorted_locations_then_a_summary() {
    let run = project().run(&["lint", "--no-markdownlint"]);
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stderr, "");
    assert_eq!(
        stable(&run.stdout),
        "◆ Ox Content · Markdown lint\n\
         a.md:3:6 warning Repeated word \"with\" looks accidental. (repeated-word)\n\
         a.md:3:10 warning Trailing whitespace is not allowed. (trailing-spaces)\n\
         \n\
         2 files · 0 errors · 2 warnings · 0 fixes · <duration>"
    );
}

#[test]
fn clean_text_report_is_only_the_banner_and_summary() {
    let run = project().run(&["lint", "b.md"]);
    run.success();
    assert_eq!(
        stable(&run.stdout),
        "◆ Ox Content · Markdown lint\n\n1 files · 0 errors · 0 warnings · 0 fixes · <duration>"
    );
}

#[test]
fn piped_output_never_contains_terminal_escapes() {
    let project = project();
    for args in [&["lint"][..], &["lint", "--no-color"], &["lint", "--format", "json"]] {
        let run = project.run(args);
        assert!(!run.stdout.contains('\x1b'), "{args:?}: {}", run.stdout);
    }
    let mut command = project.command(&["lint"]);
    command.env("CLICOLOR_FORCE", "1").env("FORCE_COLOR", "1");
    assert!(!oxct::Run::of(command, None).stdout.contains('\x1b'));
}

#[test]
fn json_report_has_a_stable_camel_case_schema() {
    let report = project().run(&["lint", "--no-markdownlint", "--format", "json"]).json();
    let keys = |value: &Value| value.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        keys(&report),
        [
            "checkedFileCount",
            "errorCount",
            "warningCount",
            "fixedCount",
            "durationMs",
            "diagnostics"
        ]
    );
    assert_eq!(report["checkedFileCount"], 2);
    assert_eq!((&report["errorCount"], &report["warningCount"]), (&0.into(), &2.into()));
    assert!(report["durationMs"].as_f64().unwrap() >= 0.0);
    assert_eq!(
        report["diagnostics"][0],
        serde_json::json!({
            "file": "a.md",
            "ruleId": "repeated-word",
            "severity": "warning",
            "message": "Repeated word \"with\" looks accidental.",
            "line": 3,
            "column": 6,
            "endLine": 3,
            "endColumn": 10,
            "language": "en",
            "fix": { "start": 13, "end": 18, "text": "" }
        })
    );
    // Optional members are omitted rather than serialized as null.
    assert_eq!(
        keys(&report["diagnostics"][1]),
        ["file", "ruleId", "severity", "message", "line", "column", "endLine", "endColumn", "fix"]
    );
}

#[test]
fn json_report_ends_with_a_single_newline_and_keeps_stderr_empty() {
    let run = project().run(&["lint", "--format", "json"]);
    assert!(run.stdout.ends_with("}\n") && !run.stdout.ends_with("\n\n"));
    assert_eq!(run.stderr, "");
}

#[test]
fn diagnostics_sort_by_file_line_column_then_rule() {
    let project = Project::new();
    project.write("b.md", "#  B!\n");
    project.write("a.md", "#  A!\n\ntext \n");
    let run = project.run(&["lint", "--format", "json"]);
    assert_eq!(
        run.diagnostics(),
        [
            ("a.md".into(), "MD019".into(), 1, 2),
            ("a.md".into(), "MD026".into(), 1, 5),
            ("a.md".into(), "MD009".into(), 3, 5),
            ("b.md".into(), "MD019".into(), 1, 2),
            ("b.md".into(), "MD026".into(), 1, 5),
        ]
    );
    let mut sorted = run.diagnostics();
    sorted.sort_by(|a, b| (&a.0, a.2, a.3, &a.1).cmp(&(&b.0, b.2, b.3, &b.1)));
    assert_eq!(run.diagnostics(), sorted);
}

#[test]
fn exit_status_follows_errors_and_the_warning_budget() {
    let project = project();
    for (args, code) in [
        (&["b.md"][..], 0),
        (&["a.md"], 1),
        (&["a.md", "--no-markdownlint"], 1),
        (&["a.md", "--no-markdownlint", "--max-warnings", "1"], 1),
        (&["a.md", "--no-markdownlint", "--max-warnings", "2"], 0),
        (&["a.md", "--no-markdownlint", "--max-warnings", "9007199254740991"], 0),
        // Errors are never absorbed by the warning budget.
        (&["a.md", "--max-warnings", "9007199254740991"], 1),
    ] {
        let run = project.run(&[&["lint"], args].concat());
        assert_eq!(run.code, Some(code), "{args:?}: {}", run.stdout);
    }
}

#[test]
fn counts_match_the_listed_diagnostics() {
    let project = project();
    project.write(".oxlint.json", r#"{"severities":{"repeated-word":"error"}}"#);
    let run = project.run(&["lint", "--format", "json"]);
    let report = run.json();
    let severities: Vec<_> = report["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["severity"].as_str().unwrap())
        .collect();
    assert_eq!(severities, ["error", "warning"]);
    assert_eq!((&report["errorCount"], &report["warningCount"]), (&1.into(), &1.into()));
}

#[test]
fn rule_listing_needs_no_documents_and_no_valid_configuration() {
    let project = Project::new();
    project.write(".markdownlint.json", "{ not json");
    let text = project.run(&["lint", "--list-rules"]);
    text.success();
    let lines: Vec<_> = text.stdout.lines().collect();
    assert_eq!(lines.len(), 53);
    assert_eq!(
        lines[0],
        "MD001 Heading levels should only increment by one level at a time (heading-increment)"
    );
    for line in &lines {
        let (id, rest) = line.split_once(' ').unwrap();
        assert!(id.len() == 5 && id.starts_with("MD"), "{line}");
        assert!(id[2..].bytes().all(|byte| byte.is_ascii_digit()), "{line}");
        assert!(rest.ends_with(')') && rest.contains(" ("), "{line}");
    }
    let rules = project.run(&["lint", "--list-rules", "--format", "json"]).json();
    let rules = rules.as_array().unwrap();
    assert_eq!(rules.len(), 53);
    for (rule, line) in rules.iter().zip(lines) {
        let id = rule["id"].as_str().unwrap();
        assert_eq!(id, format!("MD{:03}", rule["number"].as_u64().unwrap()));
        assert!(line.starts_with(&format!("{id} {} (", rule["description"].as_str().unwrap())));
        assert!(!rule["aliases"].as_array().unwrap().is_empty(), "{id}");
        assert!(!rule["tags"].as_array().unwrap().is_empty(), "{id}");
    }
}

#[test]
fn invalid_arguments_are_rejected_before_any_document_is_read() {
    let project = project();
    for (args, reason) in [
        (&["--format", "xml"][..], "invalid value 'xml'"),
        (&["--format"], "a value is required"),
        (&["--threads", "0"], "0 is not in 1..=256"),
        (&["--threads", "257"], "257 is not in 1..=256"),
        (&["--threads", "many"], "invalid value 'many'"),
        (&["--max-warnings", "-1"], "unexpected argument '-1'"),
        (&["--max-warnings", "9007199254740992"], "is not in 0..=9007199254740991"),
        (&["--stdin", "a.md"], "'--stdin' cannot be used with '[PATHS]...'"),
        (&["--markdownlint", "--no-markdownlint"], "cannot be used with '--no-markdownlint'"),
        (&["--fix", "--stdin"], "cannot be used with"),
        (&["--bogus"], "unexpected argument '--bogus'"),
        (&["--version"], "unexpected argument '--version'"),
        (&["--config"], "a value is required"),
    ] {
        project.run(&[&["lint"], args].concat()).rejected(reason);
    }
    assert_eq!(project.read("a.md"), "# Guide\n\nwith with \n");
}

#[test]
fn every_thread_count_in_range_produces_the_same_report() {
    let project = project();
    let report = |threads: &str| {
        let mut report = project.run(&["lint", "--format", "json", "--threads", threads]).json();
        report.as_object_mut().unwrap().remove("durationMs");
        report
    };
    let serial = report("1");
    for threads in ["2", "64", "256"] {
        assert_eq!(report(threads), serial, "--threads {threads}");
    }
}

#[test]
fn stdin_is_checked_instead_of_the_working_directory() {
    let project = project();
    let run = project.run_with(&["lint", "--stdin", "--format", "json"], b"# Title\n");
    run.success();
    assert_eq!(run.json()["checkedFileCount"], 1);
    assert_eq!(run.json()["diagnostics"], serde_json::json!([]));
    let run = project.run_with(&["lint", "--stdin", "--format", "json"], b"#  Title\n");
    assert_eq!(run.diagnostics(), [("stdin.md".into(), "MD019".into(), 1, 2)]);
    let labelled = ["lint", "--stdin", "--stdin-filepath", "docs/日本語.md", "--format", "json"];
    let run = project.run_with(&labelled, b"#  Title\n");
    assert_eq!(run.diagnostics()[0].0, "docs/日本語.md");
}

#[test]
fn stdin_edge_cases_keep_their_exit_status_meaningful() {
    let project = Project::new();
    project.run_with(&["lint", "--stdin"], b"").success();
    project.run_with(&["lint", "--stdin"], b"\xff\xfe").rejected("valid UTF-8");
    let crlf = project.run_with(
        &["lint", "--stdin", "--no-markdownlint", "--format", "json"],
        b"# T\r\n\r\nwith with\r\n",
    );
    assert_eq!(crlf.diagnostics(), [("stdin.md".into(), "repeated-word".into(), 3, 6)]);
}

#[test]
fn stdin_file_path_selects_mdx_rules_case_insensitively() {
    let project = Project::new();
    // Only MDX treats the import and the JSX expression as syntax instead of prose.
    let source = b"import Card from './Card'\n\n<Card tone={theme.wrld} />\n";
    let lint = |path: &str| {
        let args =
            ["lint", "--stdin", "--stdin-filepath", path, "--no-markdownlint", "--spellcheck"];
        project.run_with(&[&args[..], &["--format", "json"]].concat(), source).rules()
    };
    for mdx in ["page.mdx", "docs/Page.MDX", "page.MdX"] {
        assert!(lint(mdx).is_empty(), "{mdx}");
    }
    for markdown in ["page.md", "page.mdc", "page.mdx.md", "mdx"] {
        assert!(lint(markdown).contains(&"spellcheck".to_string()), "{markdown}");
    }
}

#[test]
fn unreadable_documents_fail_with_their_path() {
    let project = project();
    std::fs::write(project.file("binary.md"), b"\xff\xfe\x00").unwrap();
    let run = project.run(&["lint", "--format", "json"]);
    run.rejected("valid UTF-8");
    assert!(run.stderr.contains("binary.md"), "{}", run.stderr);
}
