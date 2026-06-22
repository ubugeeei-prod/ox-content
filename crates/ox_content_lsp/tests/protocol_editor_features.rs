use serde_json::{json, Value};

use protocol_support::{temp_uri, Server};

mod protocol_support;

#[test]
fn document_link_resolves_i18n_key_to_dictionary_file() {
    let root = std::env::temp_dir().join("ox-content-lsp-e2e-i18n");
    let dict_dir = root.join("content/i18n/en");
    let src_dir = root.join("src");
    std::fs::create_dir_all(&dict_dir).expect("create dict dir");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(dict_dir.join("common.json"), r#"{"hello":"Hello"}"#).expect("write dict");
    let source_path = src_dir.join("app.ts");
    let source_uri = format!("file://{}", source_path.display());
    let root_uri = format!("file://{}", root.display());

    let mut server = Server::start();
    let id = server.request(
        "initialize",
        json!({
            "capabilities": {},
            "processId": null,
            "rootUri": root_uri,
            "initializationOptions": {}
        }),
    );
    let _ = server.await_response(id);
    server.notify("initialized", json!({}));
    server.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": source_uri,
                "languageId": "typescript",
                "version": 1,
                "text": "const msg = i18n.t('common.hello');\n",
            }
        }),
    );
    let _ = server.await_notification("textDocument/publishDiagnostics");

    let id = server
        .request("textDocument/documentLink", json!({ "textDocument": { "uri": source_uri } }));
    let links = server.await_response(id);
    let targets: Vec<String> = links
        .as_array()
        .expect("documentLink returns an array")
        .iter()
        .filter_map(|link| link["target"].as_str().map(str::to_string))
        .collect();
    assert!(
        targets.iter().any(|target| target.ends_with("/content/i18n/en/common.json")),
        "missing i18n dictionary link, got {targets:?}"
    );

    server.shutdown();
}

#[test]
fn completion_offers_builtin_meta_frontmatter_fields() {
    let mut server = Server::start();
    let uri = temp_uri("frontmatter-meta.md");
    server.initialize_and_open(&uri, "---\nmeta:\n  \n---\n\n# Heading\n");

    let _ = server.await_notification("textDocument/publishDiagnostics");
    let id = server.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 2 } }),
    );
    let response = server.await_response(id);
    let items = response
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| response.as_array())
        .expect("completion returns items");
    let labels: Vec<String> =
        items.iter().map(|item| item["label"].as_str().unwrap_or_default().to_string()).collect();

    assert!(labels.iter().any(|label| label == "canonical"), "missing canonical, got {labels:?}");
    assert!(labels.iter().any(|label| label == "robots"), "missing robots, got {labels:?}");

    server.shutdown();
}

#[test]
fn diagnostics_validate_builtin_meta_frontmatter_fields() {
    let mut server = Server::start();
    let uri = temp_uri("frontmatter-meta-diagnostic.md");
    server.initialize_and_open(&uri, "---\nmeta:\n  unknown: true\n---\n\n# Heading\n");

    let params = server.await_notification("textDocument/publishDiagnostics");
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|diag| {
            diag["source"].as_str() == Some("ox-content")
                && diag["message"].as_str() == Some("Unknown frontmatter field `unknown`")
        }),
        "expected unknown meta field diagnostic, got {diagnostics:?}"
    );

    server.shutdown();
}

#[test]
fn diagnostics_report_default_half_full_width_spacing_rule() {
    let mut server = Server::start();
    let uri = temp_uri("spacing.md");
    server.initialize_and_open(&uri, "Rust と TypeScript\n");

    let params = server.await_notification("textDocument/publishDiagnostics");
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diag| diag["source"].as_str() == Some("ox-content-spacing"))
            .count(),
        2,
        "expected two spacing diagnostics, got {diagnostics:?}"
    );

    server.shutdown();
}

#[test]
fn code_action_returns_spacing_quickfix() {
    let mut server = Server::start();
    let uri = temp_uri("spacing-action.md");
    server.initialize_and_open(&uri, "Rust と\n");

    let params = server.await_notification("textDocument/publishDiagnostics");
    let diagnostic = params["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .find(|diag| diag["source"].as_str() == Some("ox-content-spacing"))
        .cloned()
        .expect("spacing diagnostic");
    let id = server.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri },
            "range": diagnostic["range"],
            "context": { "diagnostics": [diagnostic] }
        }),
    );
    let response = server.await_response(id);
    let actions = response.as_array().expect("codeAction returns an array");
    assert!(
        actions.iter().any(|action| {
            action["kind"].as_str() == Some("quickfix")
                && action["title"].as_str() == Some("Fix half/full-width spacing")
                && action["edit"]["changes"][uri.as_str()][0]["newText"].as_str() == Some("")
        }),
        "missing spacing quickfix, got {actions:?}"
    );

    server.shutdown();
}

#[test]
fn formatting_requires_half_full_width_spacing_when_configured() {
    let mut server = Server::start();
    let uri = temp_uri("spacing-format.md");
    server.initialize_and_open_with(
        &uri,
        "RustとTypeScript\n",
        json!({ "spaceBetweenHalfAndFullWidth": "require" }),
    );
    let _ = server.await_notification("textDocument/publishDiagnostics");

    let id = server.request(
        "textDocument/formatting",
        json!({
            "textDocument": { "uri": uri },
            "options": { "tabSize": 2, "insertSpaces": true }
        }),
    );
    let response = server.await_response(id);
    let edits = response.as_array().expect("formatting returns edits");
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|edit| edit["newText"].as_str() == Some(" ")));

    server.shutdown();
}

#[test]
fn will_save_wait_until_is_opt_in_for_spacing_fixes() {
    let mut server = Server::start();
    let uri = temp_uri("spacing-save.md");
    server.initialize_and_open_with(&uri, "Rust と\n", json!({ "spacingAutoFixOnSave": true }));
    let _ = server.await_notification("textDocument/publishDiagnostics");

    let id = server.request(
        "textDocument/willSaveWaitUntil",
        json!({
            "textDocument": { "uri": uri },
            "reason": 1
        }),
    );
    let response = server.await_response(id);
    let edits = response.as_array().expect("willSaveWaitUntil returns edits");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0]["newText"].as_str(), Some(""));

    server.shutdown();
}

#[cfg(unix)]
#[test]
fn did_save_publishes_textlint_diagnostics_when_enabled() {
    let command = write_fake_textlint_command("protocol-textlint");

    let mut server = Server::start();
    let uri = temp_uri("textlint.md");
    server.initialize_and_open_with(
        &uri,
        "Textlint fixture\n",
        json!({
            "textlintEnabled": true,
            "textlintCommand": command,
        }),
    );

    let _ = server.await_notification("textDocument/publishDiagnostics");
    server.notify("textDocument/didSave", json!({ "textDocument": { "uri": uri } }));

    let params = server.await_notification("textDocument/publishDiagnostics");
    assert_eq!(params["uri"].as_str(), Some(uri.as_str()));
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|diag| {
            diag["source"].as_str() == Some("textlint")
                && diag["code"].as_str() == Some("fixture/no-textlint")
                && diag["message"].as_str() == Some("fixture textlint diagnostic")
        }),
        "expected a textlint diagnostic, got {diagnostics:?}"
    );

    server.shutdown();
}

#[cfg(unix)]
#[test]
fn code_action_returns_textlint_quickfix() {
    let command = write_fake_textlint_command("protocol-textlint-fix");

    let mut server = Server::start();
    let uri = temp_uri("textlint-fix.md");
    server.initialize_and_open_with(
        &uri,
        "Javascript fixture\n",
        json!({
            "textlintEnabled": true,
            "textlintCommand": command,
        }),
    );

    let _ = server.await_notification("textDocument/publishDiagnostics");
    server.notify("textDocument/didSave", json!({ "textDocument": { "uri": uri } }));
    let params = server.await_notification("textDocument/publishDiagnostics");
    let diagnostic = params["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .find(|diag| diag["source"].as_str() == Some("textlint"))
        .cloned()
        .expect("textlint diagnostic");

    let id = server.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri },
            "range": diagnostic["range"],
            "context": { "diagnostics": [diagnostic] }
        }),
    );
    let response = server.await_response(id);
    let actions = response.as_array().expect("codeAction returns an array");
    assert!(
        actions.iter().any(|action| {
            action["kind"].as_str() == Some("quickfix")
                && action["title"].as_str() == Some("Apply textlint fix")
                && action["edit"]["changes"][uri.as_str()][0]["newText"].as_str()
                    == Some("JavaScript")
        }),
        "missing textlint quickfix, got {actions:?}"
    );

    server.shutdown();
}

#[cfg(unix)]
fn write_fake_textlint_command(name: &str) -> String {
    use std::os::unix::fs::PermissionsExt;

    let mut path = std::env::temp_dir();
    path.push("ox-content-lsp-e2e");
    std::fs::create_dir_all(&path).expect("create temp dir");
    path.push(format!("{name}.sh"));
    std::fs::write(
        &path,
        r#"#!/bin/sh
cat >/dev/null
printf '%s\n' '[{"filePath":"doc.md","messages":[{"ruleId":"fixture/no-textlint","message":"fixture textlint diagnostic","line":1,"column":4,"severity":1,"fix":{"range":[0,10],"text":"JavaScript"}}]}]'
exit 1
"#,
    )
    .expect("write fake textlint command");

    let mut permissions = std::fs::metadata(&path).expect("fake command metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod fake textlint command");
    path.display().to_string()
}
