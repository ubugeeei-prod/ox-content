//! `oxct lint --fix` edits documents in place, and only ever safely.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

#[track_caller]
fn fix(project: &Project, args: &[&str]) -> serde_json::Value {
    project.run(&[&["lint", "--fix", "--format", "json"], args].concat()).json()
}

/// Apply fixes to one document and return `(output, fixedCount, remaining rule ids)`.
#[track_caller]
fn fixed(source: &str, args: &[&str]) -> (String, u64, Vec<String>) {
    let project = Project::new();
    project.write("a.md", source);
    let run = project.run(&[&["lint", "--fix", "--format", "json"], args].concat());
    (project.read("a.md"), run.json()["fixedCount"].as_u64().unwrap(), run.rules())
}

#[test]
fn markdownlint_whitespace_rules_are_fixed_in_place() {
    for (source, output, fixes) in [
        ("# Title \n", "# Title\n", 1),
        ("# Title\n\ntext   \n", "# Title\n\ntext\n", 1),
        ("# Title\n\n\n\ntext\n", "# Title\n\ntext\n", 2),
        ("# Title\n\ntext", "# Title\n\ntext\n", 1),
        // Extra spaces before a continuation line shrink to a hard break.
        ("# Title \n\n\ntext   \nend", "# Title\n\ntext  \nend\n", 4),
    ] {
        assert_eq!(fixed(source, &[]), (output.to_string(), fixes, vec![]), "{source:?}");
    }
}

#[test]
fn hard_breaks_line_endings_and_unicode_survive_fixes() {
    for (source, output) in [
        // Two trailing spaces are a Markdown hard break.
        ("# Title\n\nfirst  \nsecond\n", "# Title\n\nfirst  \nsecond\n"),
        ("# Title\r\n\r\ntext \r\n", "# Title\r\n\r\ntext\r\n"),
        ("# Title\r\n\r\n\r\n\r\ntext", "# Title\r\n\r\ntext\r\n"),
        ("# 日本語 👩‍💻 \n\nテキスト \n", "# 日本語 👩‍💻\n\nテキスト\n"),
        ("# e\u{301}toile \n", "# e\u{301}toile\n"),
    ] {
        let (actual, _, remaining) = fixed(source, &[]);
        assert_eq!(actual, output, "{source:?}");
        assert!(remaining.is_empty(), "{source:?}: {remaining:?}");
    }
}

#[test]
fn prose_fixes_remove_repeated_words_and_apply_terminology() {
    let project = Project::new();
    project.write(
        ".oxlint.json",
        r#"{"textRules":{"terminology":[{"term":"Javascript","replacement":"JavaScript"}]}}"#,
    );
    project.write("a.md", "# Guide\n\nJavascript is is `Javascript` \n");
    let report = fix(&project, &[]);
    assert_eq!(report["fixedCount"], 3);
    assert_eq!(report["diagnostics"], serde_json::json!([]));
    assert_eq!(project.read("a.md"), "# Guide\n\nJavaScript is `Javascript`\n");
}

#[test]
fn fixing_is_idempotent_and_reports_only_what_remains() {
    let project = Project::new();
    project.write("a.md", "#  Title \n\n\n* one\n- two\n\n[]()\n\ttab   \nend");
    let first = project.run(&["lint", "--fix", "--format", "json"]);
    assert_eq!(first.code, Some(1), "unfixable findings still fail the run");
    let report = first.json();
    assert!(report["fixedCount"].as_u64().unwrap() >= 3, "{report}");
    let content = project.read("a.md");
    assert_eq!(content, "# Title\n\n* one\n- two\n\n[]()\n\ttab  \nend\n");
    // The report describes the fixed document, so every remaining finding has no pending fix.
    let lines = content.lines().count() as u64;
    for (_, rule, line, _) in first.diagnostics() {
        assert!((1..=lines).contains(&line), "{rule} at {line}");
    }
    let second = project.run(&["lint", "--fix", "--format", "json"]);
    assert_eq!(second.json()["fixedCount"], 0);
    assert_eq!(second.diagnostics(), first.diagnostics());
    assert_eq!(project.read("a.md"), content);
    let check = project.run(&["lint", "--format", "json"]);
    assert_eq!(check.diagnostics(), first.diagnostics());
}

#[test]
fn only_documents_with_fixes_are_rewritten_and_nothing_else_is_left_behind() {
    let project = Project::new();
    project.write("dirty.md", "# Dirty \n");
    project.write("clean.md", "# Clean\n");
    project.write("docs/nested.md", "# Nested\n\n\n\ntext\n");
    let before = std::fs::metadata(project.file("clean.md")).unwrap().modified().unwrap();
    let report = fix(&project, &[]);
    assert_eq!((&report["checkedFileCount"], &report["fixedCount"]), (&3.into(), &3.into()));
    assert_eq!(project.read("dirty.md"), "# Dirty\n");
    assert_eq!(project.read("docs/nested.md"), "# Nested\n\ntext\n");
    assert_eq!(std::fs::metadata(project.file("clean.md")).unwrap().modified().unwrap(), before);
    assert_eq!(project.list("."), ["clean.md", "dirty.md", "docs"]);
    assert_eq!(project.list("docs"), ["nested.md"]);
}

#[test]
fn fixes_respect_ignores_inline_comments_and_disabled_rules() {
    let project = Project::new();
    project.write("keep.md", "# Keep \n");
    project.write("skip.md", "# Skip \n");
    project.write("comment.md", "# Comment\n\n<!-- markdownlint-disable MD009 -->\ntext \n");
    project.write(".markdownlint.json", r#"{"MD047":false}"#);
    project.write("eof.md", "# No newline");
    let report = fix(&project, &["--ignore", "skip.md"]);
    assert_eq!((&report["checkedFileCount"], &report["fixedCount"]), (&3.into(), &1.into()));
    assert_eq!(project.read("keep.md"), "# Keep\n");
    assert_eq!(project.read("skip.md"), "# Skip \n");
    assert!(project.read("comment.md").ends_with("text \n"));
    assert_eq!(project.read("eof.md"), "# No newline");
    // Ignoring the comments makes the same finding fixable again.
    assert_eq!(fix(&project, &["comment.md", "--no-inline-config"])["fixedCount"], 1);
    assert!(project.read("comment.md").ends_with("text\n"));
}

#[test]
fn text_summary_counts_fixes() {
    let project = Project::new();
    project.write("a.md", "# Title \n\n\n\ntext");
    let run = project.run(&["lint", "--fix"]);
    run.success();
    assert!(run.stdout.contains("1 files · 0 errors · 0 warnings · 4 fixes · "), "{}", run.stdout);
}

#[test]
fn severity_off_disables_the_fix_with_the_rule() {
    let project = Project::new();
    project.write(".oxlint.json", r#"{"severities":{"trailing-spaces":"off"}}"#);
    project.write("a.md", "with with \n");
    assert_eq!(fix(&project, &[])["fixedCount"], 1);
    assert_eq!(project.read("a.md"), "with \n");
}

/// Fix counts and edits stay per-file when documents span several worker batches.
#[test]
fn fixes_apply_to_every_document_across_worker_batches() {
    let project = Project::new();
    for index in 0..260 {
        let content = if index % 2 == 0 { "# Title \n" } else { "# Title\n" };
        project.write(&format!("{index:03}.md"), content);
    }
    let report = fix(&project, &["--threads", "3"]);
    assert_eq!((&report["checkedFileCount"], &report["fixedCount"]), (&260.into(), &130.into()));
    assert_eq!(report["diagnostics"], serde_json::json!([]));
    for index in 0..260 {
        assert_eq!(project.read(&format!("{index:03}.md")), "# Title\n", "{index}");
    }
}

#[cfg(unix)]
#[test]
fn fixed_documents_keep_their_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    for mode in [0o600, 0o640, 0o644, 0o755] {
        let name = format!("{mode:o}.md");
        let file = project.write(&name, "# Title \n");
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    assert_eq!(fix(&project, &[])["fixedCount"], 4);
    for mode in [0o600, 0o640, 0o644, 0o755] {
        let file = project.file(&format!("{mode:o}.md"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "# Title\n");
        assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, mode);
    }
}

#[cfg(unix)]
#[test]
fn symlinks_are_never_replaced_by_regular_files() {
    let project = Project::new();
    project.write("real/target.md", "# Title \n");
    std::os::unix::fs::symlink("real/target.md", project.file("link.md")).unwrap();
    let run = project.run(&["lint", "--fix", "link.md"]);
    run.rejected("Refusing to fix non-regular file");
    assert!(std::fs::symlink_metadata(project.file("link.md")).unwrap().file_type().is_symlink());
    assert_eq!(project.read("real/target.md"), "# Title \n");
    // Without --fix the linked document is still checked.
    assert_eq!(project.run(&["lint", "link.md", "--format", "json"]).rules(), ["MD009"]);
}

#[cfg(unix)]
#[test]
fn read_only_directories_fail_without_corrupting_the_document() {
    use std::os::unix::fs::PermissionsExt;
    let project = Project::new();
    project.write("locked/a.md", "# Title \n");
    let directory = project.file("locked");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o555)).unwrap();
    let run = project.run(&["lint", "--fix", "locked"]);
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o755)).unwrap();
    // Privileged users bypass directory permissions; then the fix simply succeeds.
    if run.code == Some(0) {
        assert_eq!(project.read("locked/a.md"), "# Title\n");
    } else {
        assert_eq!(run.code, Some(1));
        assert!(run.stderr.contains("a.md"), "{}", run.stderr);
        assert_eq!(project.read("locked/a.md"), "# Title \n");
    }
    assert_eq!(project.list("locked"), ["a.md"]);
}

#[test]
fn fix_cannot_be_combined_with_stdin() {
    let project = Project::new();
    project.write("a.md", "# Title \n");
    project.run_with(&["lint", "--fix", "--stdin"], b"# Title \n").rejected("cannot be used with");
    assert_eq!(project.read("a.md"), "# Title \n");
}
