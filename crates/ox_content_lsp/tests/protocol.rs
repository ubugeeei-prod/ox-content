//! End-to-end protocol tests for the `ox-content-lsp` binary.
//!
//! These spawn the real compiled server and speak LSP over stdio — the
//! exact JSON-RPC framing an editor uses — so they exercise the wiring
//! that the in-crate unit tests can't: capability advertisement, request
//! dispatch, and the shape of each response on the wire. No editor and no
//! `cargo run` indirection; `cargo test` builds the bin and hands us its
//! path via `CARGO_BIN_EXE_ox-content-lsp`.

use serde_json::{json, Value};

use protocol_support::{temp_uri, Server};

mod protocol_support;

#[test]
fn initialize_advertises_the_expected_capabilities() {
    let mut server = Server::start();
    let result = server.initialize_and_open(&temp_uri("caps.md"), "# Hello\n");
    let caps = &result["capabilities"];

    assert_eq!(caps["foldingRangeProvider"], json!(true));
    assert!(caps["documentLinkProvider"].is_object(), "missing documentLinkProvider");
    assert_eq!(caps["documentSymbolProvider"], json!(true));
    assert_eq!(caps["hoverProvider"], json!(true));
    assert!(caps["completionProvider"].is_object(), "missing completionProvider");

    server.shutdown();
}

#[test]
fn folding_range_returns_heading_and_nested_sections() {
    let mut server = Server::start();
    let uri = temp_uri("folding.md");
    server.initialize_and_open(&uri, "# Title\n\n## Section\n\ncontent\n");

    let id = server.request("textDocument/foldingRange", json!({ "textDocument": { "uri": uri } }));
    let ranges = server.await_response(id);
    let folds: Vec<(i64, i64)> = ranges
        .as_array()
        .expect("foldingRange returns an array")
        .iter()
        .map(|range| (range["startLine"].as_i64().unwrap(), range["endLine"].as_i64().unwrap()))
        .collect();

    // The h1 owns the whole body (down to the last content line) and the
    // h2 owns its own lines.
    assert_eq!(folds, vec![(0, 4), (2, 4)]);

    server.shutdown();
}

#[test]
fn document_link_resolves_relative_and_external_targets() {
    let mut server = Server::start();
    let uri = temp_uri("links.md");
    server.initialize_and_open(&uri, "[next](./other.md) and [site](https://example.com/docs)\n");

    let id = server.request("textDocument/documentLink", json!({ "textDocument": { "uri": uri } }));
    let links = server.await_response(id);
    let targets: Vec<String> = links
        .as_array()
        .expect("documentLink returns an array")
        .iter()
        .map(|link| link["target"].as_str().expect("link target is a string").to_string())
        .collect();

    assert!(
        targets.iter().any(|target| target.ends_with("/other.md") && target.starts_with("file://")),
        "missing resolved relative link, got {targets:?}"
    );
    assert!(
        targets.iter().any(|target| target == "https://example.com/docs"),
        "missing external link, got {targets:?}"
    );

    server.shutdown();
}

#[test]
fn document_symbol_returns_headings() {
    let mut server = Server::start();
    let uri = temp_uri("symbols.md");
    server.initialize_and_open(&uri, "# Title\n\n## Section\n");

    let id =
        server.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } }));
    let symbols = server.await_response(id);
    let names: Vec<String> = symbols
        .as_array()
        .expect("documentSymbol returns an array")
        .iter()
        .map(|symbol| symbol["name"].as_str().unwrap_or_default().to_string())
        .collect();

    assert!(names.iter().any(|name| name == "Title"), "missing Title heading, got {names:?}");
    assert!(names.iter().any(|name| name == "Section"), "missing Section heading, got {names:?}");

    server.shutdown();
}

#[test]
fn folding_range_includes_the_frontmatter_block() {
    let mut server = Server::start();
    let uri = temp_uri("frontmatter-fold.md");
    server.initialize_and_open(&uri, "---\ntitle: Doc\ntags: a\n---\n\n# Heading\n\nbody\n");

    let id = server.request("textDocument/foldingRange", json!({ "textDocument": { "uri": uri } }));
    let ranges = server.await_response(id);
    let folds: Vec<(i64, i64)> = ranges
        .as_array()
        .expect("foldingRange returns an array")
        .iter()
        .map(|range| (range["startLine"].as_i64().unwrap(), range["endLine"].as_i64().unwrap()))
        .collect();

    // The frontmatter block spans lines 0..3 (opening `---` through
    // closing `---`).
    assert_eq!(folds, vec![(0, 3), (5, 7)]);

    server.shutdown();
}

#[test]
fn completion_offers_markdown_snippets() {
    let mut server = Server::start();
    let uri = temp_uri("completion.md");
    server.initialize_and_open(&uri, "# Title\n\n\n");

    // An empty line offers the Markdown snippet set.
    let id = server.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 0 } }),
    );
    let response = server.await_response(id);
    // The response is either a bare array or a CompletionList `{ items }`.
    let items = response
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| response.as_array())
        .expect("completion returns items");
    let labels: Vec<String> =
        items.iter().map(|item| item["label"].as_str().unwrap_or_default().to_string()).collect();

    assert!(labels.iter().any(|label| label == "h1"), "missing h1 snippet, got {labels:?}");
    assert!(labels.iter().any(|label| label == "table"), "missing table snippet, got {labels:?}");

    server.shutdown();
}

#[test]
fn diagnostics_report_a_dead_relative_link() {
    let mut server = Server::start();
    let uri = temp_uri("dead-link.md");
    // The link target does not exist under the temp dir, so the link
    // checker should flag it on open.
    server.initialize_and_open(&uri, "See [missing](./does-not-exist.md).\n");

    let params = server.await_notification("textDocument/publishDiagnostics");
    assert_eq!(params["uri"].as_str(), Some(uri.as_str()));
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|diag| diag["source"].as_str() == Some("ox-content-link")),
        "expected an ox-content-link diagnostic, got {diagnostics:?}"
    );

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

    // Drain the normal on-open diagnostics first so the next publish is
    // the save-triggered textlint run.
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

#[test]
fn hover_describes_a_frontmatter_field() {
    // Hover over a frontmatter key only resolves when a schema is
    // configured, so write one to a temp file and point the server at it
    // via initializationOptions.
    let mut schema_path = std::env::temp_dir();
    schema_path.push("ox-content-lsp-e2e");
    std::fs::create_dir_all(&schema_path).expect("create temp dir");
    schema_path.push("hover-schema.json");
    std::fs::write(
        &schema_path,
        r#"{ "properties": { "title": { "type": "string", "description": "The page title" } } }"#,
    )
    .expect("write schema");

    let mut server = Server::start();
    let uri = temp_uri("hover.md");
    server.initialize_and_open_with(
        &uri,
        "---\ntitle: Hello\n---\n\n# Heading\n",
        json!({ "frontmatterSchema": schema_path.display().to_string() }),
    );

    // The `title` key sits on line 1; hover in the middle of it.
    let id = server.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 2 } }),
    );
    let hover = server.await_response(id);
    let value = hover["contents"]["value"].as_str().expect("hover markup value");
    insta::assert_snapshot!(value);

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
printf '%s\n' '[{"filePath":"doc.md","messages":[{"ruleId":"fixture/no-textlint","message":"fixture textlint diagnostic","line":1,"column":4,"severity":1}]}]'
exit 1
"#,
    )
    .expect("write fake textlint command");

    let mut permissions = std::fs::metadata(&path).expect("fake command metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod fake textlint command");
    path.display().to_string()
}
