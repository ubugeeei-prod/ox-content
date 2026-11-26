use std::fs;
use std::path::PathBuf;

use super::*;

fn workspace_with_textlint(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ox-content-lsp-config-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    fs::write(
        dir.join(".ox-content.json"),
        r#"{"textlint":{"enabled":true,"command":"workspace-textlint --flag"}}"#,
    )
    .expect("write workspace config");
    dir
}

#[test]
fn workspace_textlint_section_is_ignored_until_the_workspace_is_trusted() {
    let dir = workspace_with_textlint("untrusted");
    let config = ResolvedConfig::load(Some(&dir), &InitializationOptions::default());
    let _ = fs::remove_dir_all(&dir);

    if env::var_os("OX_CONTENT_TEXTLINT_ENABLED").is_none() {
        assert!(!config.textlint.enabled);
    }
    assert_ne!(config.textlint.command.as_deref(), Some("workspace-textlint --flag"));
}

#[test]
fn trusted_workspace_reads_the_textlint_section() {
    let dir = workspace_with_textlint("trusted");
    let init =
        InitializationOptions { workspace_trusted: true, ..InitializationOptions::default() };
    let config = ResolvedConfig::load(Some(&dir), &init);
    let _ = fs::remove_dir_all(&dir);

    assert!(config.textlint.enabled);
    assert_eq!(config.textlint.command.as_deref(), Some("workspace-textlint --flag"));
}

#[test]
fn init_options_enable_textlint_without_trust() {
    let dir = workspace_with_textlint("init");
    let init = InitializationOptions {
        textlint_enabled: true,
        textlint_command: Some("editor-textlint".to_string()),
        ..InitializationOptions::default()
    };
    let config = ResolvedConfig::load(Some(&dir), &init);
    let _ = fs::remove_dir_all(&dir);

    assert!(config.textlint.enabled);
    assert_eq!(config.textlint.command.as_deref(), Some("editor-textlint"));
}

#[test]
fn workspace_trust_flag_deserializes_from_camel_case() {
    let init: InitializationOptions =
        serde_json::from_value(serde_json::json!({ "workspaceTrusted": true })).expect("parse");
    assert!(init.workspace_trusted);
    assert!(!InitializationOptions::default().workspace_trusted);
}
