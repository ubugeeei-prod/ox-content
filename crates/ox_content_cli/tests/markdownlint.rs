use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oxct"))
        .arg("lint")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}
fn report(root: &Path, args: &[&str]) -> Value {
    let result = run(root, args);
    serde_json::from_slice(&result.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&result.stderr)))
}

#[test]
fn native_profile_is_default_and_lists_all_rules_without_discovering_files() {
    let root = tempfile::tempdir().unwrap();
    let rules = report(root.path(), &["--list-rules", "--format", "json"]);
    assert_eq!(rules.as_array().unwrap().len(), 53);
    fs::write(root.path().join("a.md"), "#  Title!\n\nTODO with with!!\n").unwrap();
    let data = report(root.path(), &["--format", "json"]);
    assert_eq!(data["errorCount"], 2);
    assert_eq!(data["diagnostics"][0]["ruleId"], "MD019");
    assert_eq!(data["diagnostics"][1]["ruleId"], "MD026");
    assert_eq!(run(root.path(), &["--max-warnings", "100"]).status.code(), Some(1));
}

#[test]
fn jsonc_yaml_extends_ignore_and_negation_work_together() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("config")).unwrap();
    fs::write(
        root.path().join("config/base.jsonc"),
        "{\n// shared\n\"default\":false,\"MD009\":true,\n}",
    )
    .unwrap();
    fs::write(root.path().join(".markdownlint.yaml"), "extends: config/base.jsonc\nMD026: false\n")
        .unwrap();
    fs::write(root.path().join(".markdownlintignore"), "*.md\n!keep.md\n").unwrap();
    fs::write(root.path().join("skip.md"), "ignored \n").unwrap();
    fs::write(root.path().join("keep.md"), "😀 text \r\n").unwrap();
    let data = report(root.path(), &["--format", "json", "--fix"]);
    assert_eq!(data["checkedFileCount"], 1);
    assert_eq!(data["fixedCount"], 1);
    assert_eq!(fs::read_to_string(root.path().join("keep.md")).unwrap(), "😀 text\r\n");
    assert_eq!(fs::read_to_string(root.path().join("skip.md")).unwrap(), "ignored \n");
}

#[test]
fn extends_cycles_and_invalid_rules_fail_before_fixing() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "text \n").unwrap();
    fs::write(root.path().join(".markdownlint.json"), r#"{"extends":"base.json"}"#).unwrap();
    fs::write(root.path().join("base.json"), r#"{"extends":".markdownlint.json"}"#).unwrap();
    let result = run(root.path(), &["--fix"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("Circular"));
    assert_eq!(fs::read_to_string(root.path().join("a.md")).unwrap(), "text \n");
    fs::write(root.path().join(".markdownlint.json"), r#"{"MD099":true}"#).unwrap();
    assert!(String::from_utf8_lossy(&run(root.path(), &["--fix"]).stderr).contains("Unknown"));
}

#[test]
fn severity_and_no_inline_config_affect_exit_status() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".markdownlint.json"), r#"{"default":false,"MD009":"warning"}"#)
        .unwrap();
    fs::write(root.path().join("a.md"), "<!-- markdownlint-disable MD009 -->\ntext \n").unwrap();
    assert!(run(root.path(), &[]).status.success());
    assert_eq!(run(root.path(), &["--no-inline-config"]).status.code(), Some(1));
    assert!(run(root.path(), &["--no-inline-config", "--max-warnings", "1"]).status.success());
}
