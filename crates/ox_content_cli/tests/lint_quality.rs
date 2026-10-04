use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn run(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oxct")).args(arguments).current_dir(root).output().unwrap()
}
fn report(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn parallel_fix_discovers_prose_config_preserves_permissions_and_is_idempotent() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".oxlint.json"), r#"{"rules":{"finalNewline":true},"textRules":{"terminology":[{"term":"Javascript","replacement":"JavaScript"}]}}"#).unwrap();
    for index in 0..24 {
        fs::write(
            root.path().join(format!("{index:02}.md")),
            "# Guide\r\n\r\nJavascript with with",
        )
        .unwrap();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root.path().join("00.md"), fs::Permissions::from_mode(0o600)).unwrap();
    }
    let fixed = run(root.path(), &["lint", "--fix", "--threads", "4", "--format", "json"]);
    assert!(fixed.status.success(), "{}", String::from_utf8_lossy(&fixed.stderr));
    let data = report(&fixed);
    assert_eq!(data["fixedCount"], 72);
    assert_eq!(data["checkedFileCount"], 24);
    assert_eq!(data["diagnostics"], serde_json::json!([]));
    assert_eq!(
        fs::read_to_string(root.path().join("00.md")).unwrap(),
        "# Guide\r\n\r\nJavaScript with\r\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(root.path().join("00.md")).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(report(&run(root.path(), &["lint", "--fix", "--format", "json"]))["fixedCount"], 0);
}

#[test]
fn diagnostic_order_is_identical_for_one_and_four_threads() {
    let root = tempfile::tempdir().unwrap();
    for index in 0..24 {
        fs::write(root.path().join(format!("{index:02}.md")), "# Guide\n\nwith with\n").unwrap();
    }
    let mut a = report(&run(root.path(), &["lint", "--threads", "1", "--format", "json"]));
    let mut b = report(&run(root.path(), &["lint", "--threads", "4", "--format", "json"]));
    a.as_object_mut().unwrap().remove("durationMs");
    b.as_object_mut().unwrap().remove("durationMs");
    assert_eq!(a, b);
}

#[test]
fn errors_ignore_warning_budget_and_invalid_options_do_not_edit() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "with with\n").unwrap();
    fs::write(root.path().join(".oxlint.json"), r#"{"severities":{"repeated-word":"error"}}"#)
        .unwrap();
    let output = run(root.path(), &["lint", "--max-warnings", "100", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(report(&output)["errorCount"], 1);
    assert_eq!(run(root.path(), &["lint", "--threads", "0", "--fix"]).status.code(), Some(1));
    assert_eq!(run(root.path(), &["lint", "--fix", "--stdin"]).status.code(), Some(1));
    assert_eq!(fs::read_to_string(root.path().join("a.md")).unwrap(), "with with\n");
    fs::write(root.path().join(".oxlint.json"), r#"{"textRules":{"sentenseLength":10}}"#).unwrap();
    assert_eq!(run(root.path(), &["lint", "--fix"]).status.code(), Some(1));
}

#[test]
fn stdin_json_prose_diagnostics_do_not_interleave_with_stdout() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".oxlint.json"), r#"{"textRules":{"noTodo":true}}"#).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxct"))
        .args(["lint", "--stdin", "--format", "json"])
        .current_dir(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"TODO: finish this\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(report(&output)["diagnostics"][0]["ruleId"], "no-todo");
}
