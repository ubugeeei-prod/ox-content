use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn run(root: &Path, args: &[&str], input: Option<&str>) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxct"))
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

#[test]
fn native_lint_discovers_globs_and_respects_config() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "# Heading\n\nA repeated repeated word.\n").unwrap();
    fs::write(root.path().join("b.md"), "# Heading\n\nClean prose.\n").unwrap();
    fs::create_dir(root.path().join("node_modules")).unwrap();
    fs::write(root.path().join("node_modules/ignored.md"), "# Heading\n### Jump\n").unwrap();
    let result = run(root.path(), &["lint", "--no-markdownlint", "--format", "json"], None);
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["checkedFileCount"], 2);
    assert_eq!(report["diagnostics"][0]["file"], "a.md");
    fs::write(
        root.path().join("lint.json"),
        r#"{"include":["*.md"],"rules":{"repeatedWords":false}}"#,
    )
    .unwrap();
    assert!(run(root.path(), &["lint", "--config", "lint.json"], None).status.success());
    let result =
        run(root.path(), &["lint", ".", "b.md", "--ignore", "a.md", "--format", "json"], None);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["checkedFileCount"], 1);
    let result = run(
        root.path(),
        &[
            "lint",
            "--stdin",
            "--no-markdownlint",
            "--stdin-filepath",
            "input.mdx",
            "--format",
            "json",
            "--max-warnings",
            "10",
        ],
        Some("export const x = 1;\n\n# Heading\n\nA repeated repeated word.\n"),
    );
    assert!(result.status.success());
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["file"], "input.mdx");
}

#[test]
fn native_project_creation_preserves_existing_files() {
    let root = tempfile::tempdir().unwrap();
    for template in ["docs", "blog", "minimal"] {
        let result = run(
            root.path(),
            &["new", template, "--template", template, "--yes", "--no-install"],
            None,
        );
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let pkg = fs::read_to_string(root.path().join(template).join("package.json")).unwrap();
        let data: Value = serde_json::from_str(&pkg).unwrap();
        assert_eq!(data["scripts"]["build"], "vp build");
        assert_eq!(data["devDependencies"]["@ox-content/vite-plugin"], env!("CARGO_PKG_VERSION"));
        assert_eq!(run(root.path(), &["new", template, "--yes"], None).status.code(), Some(1));
        assert_eq!(
            fs::read_to_string(root.path().join(template).join("package.json")).unwrap(),
            pkg
        );
    }
    assert_eq!(
        run(root.path(), &["new", "invalid", "--skin", "missing", "--yes"], None).status.code(),
        Some(1)
    );
    assert!(!root.path().join("invalid").exists());
}

#[test]
fn native_ide_merges_comments_idempotently_and_preserves_permissions() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".vscode")).unwrap();
    let settings = root.path().join(".vscode/settings.json");
    fs::write(&settings, "{\n// Keep this comment\n\"editor.tabSize\":4,\"files.associations\":{\"*.foo\":\"text\"},\n}\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&settings, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let args = ["ide", "install", "--ide", "vscode", "--ide", "zed", "--config-only", "--yes"];
    let result = run(root.path(), &args, None);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let content = fs::read_to_string(&settings).unwrap();
    assert!(content.contains("Keep this comment"));
    let parsed: Value =
        jsonc_parser::parse_to_serde_value(&content, &jsonc_parser::ParseOptions::default())
            .unwrap();
    assert_eq!(parsed["files.associations"]["*.foo"], "text");
    assert_eq!(parsed["files.associations"]["*.mdc"], "markdown");
    assert_eq!(parsed["editor.tabSize"], 4);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&settings).unwrap().permissions().mode() & 0o777, 0o600);
    }
    let before = fs::read_dir(root.path().join(".vscode")).unwrap().count();
    assert!(run(root.path(), &args, None).status.success());
    assert_eq!(fs::read_to_string(settings).unwrap(), content);
    assert_eq!(fs::read_dir(root.path().join(".vscode")).unwrap().count(), before);
}

#[test]
fn native_viewer_and_help_work_without_node() {
    let root = tempfile::tempdir().unwrap();
    let result = run(
        root.path(),
        &["tui", "--stdin", "--no-color", "--width", "64"],
        Some("# 日本語 👩‍💻\n\nA **bold** paragraph.\n"),
    );
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("日本語 👩‍💻"));
    assert!(!text.contains('\x1b'));
    for args in [
        vec!["--help"],
        vec!["new", "--help"],
        vec!["ide", "install", "--help"],
        vec!["lint", "--help"],
        vec!["tui", "--help"],
    ] {
        assert!(run(root.path(), &args, None).status.success());
    }
}
