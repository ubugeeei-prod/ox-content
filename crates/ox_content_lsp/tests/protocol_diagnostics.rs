use serde_json::{Value, json};

use protocol_support::{Server, temp_uri};

mod protocol_support;

const FRONTMATTER_LINKS: &str = concat!(
    "---\n",
    "meta:\n",
    "  unknown: true\n",
    "---\n",
    "\n",
    "# Heading\n",
    "\n",
    "See [missing](./does-not-exist.md).\n",
);
const LINK_LINE: u32 = 7;

#[test]
fn diagnostics_cover_frontmatter_and_links_together() {
    let mut server = Server::start();
    let uri = temp_uri("frontmatter-links.md");
    server.initialize_and_open(&uri, FRONTMATTER_LINKS);

    let params = server.await_diagnostics_version(&uri, 1);
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|diag| {
            diag["source"].as_str() == Some("ox-content")
                && diag["code"].as_str() == Some("frontmatter-unknown")
                && diag["message"].as_str() == Some("Unknown frontmatter field `unknown`")
        }),
        "missing frontmatter diagnostic, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diag| {
            diag["source"].as_str() == Some("ox-content-link")
                && diag["code"].as_str() == Some("link-missing-file")
        }),
        "missing link diagnostic, got {diagnostics:?}"
    );

    server.shutdown();
}

#[test]
fn incremental_did_change_recomputes_links_and_keeps_frontmatter() {
    let mut server = Server::start();
    let uri = temp_uri("incremental-links.md");
    server.initialize_and_open(&uri, FRONTMATTER_LINKS);
    let _ = server.await_diagnostics_version(&uri, 1);

    // Replace `./does-not-exist.md` with `#heading` so the link becomes valid.
    server.did_change_incremental(&uri, 2, (LINK_LINE, 14), (LINK_LINE, 33), "#heading");
    let params = server.await_diagnostics_version(&uri, 2);
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().any(|diag| diag["code"].as_str() == Some("frontmatter-unknown")),
        "frontmatter diagnostic must survive a body-only edit, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().all(|diag| diag["source"].as_str() != Some("ox-content-link")),
        "fixed link must not keep a stale link diagnostic, got {diagnostics:?}"
    );

    server.shutdown();
}

#[test]
fn later_document_version_wins_over_cancelled_edits() {
    let mut server = Server::start();
    let uri = temp_uri("rapid-edits.md");
    server.initialize_and_open(&uri, FRONTMATTER_LINKS);
    let _ = server.await_diagnostics_version(&uri, 1);

    // v2 still broken; v3 fixes the link. The later version must win.
    server.did_change_incremental(&uri, 2, (LINK_LINE, 14), (LINK_LINE, 33), "./also-missing.md");
    server.did_change_incremental(&uri, 3, (LINK_LINE, 14), (LINK_LINE, 31), "#heading");
    let params = server.await_diagnostics_version(&uri, 3);
    let diagnostics = params["diagnostics"].as_array().expect("diagnostics array");
    assert!(
        diagnostics.iter().all(|diag| {
            diag["source"].as_str() != Some("ox-content-link")
                && diag["code"].as_str() != Some("link-missing-file")
        }),
        "version 3 must publish the fixed document, got {diagnostics:?}"
    );

    let hover_id = server.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": 2 } }),
    );
    loop {
        let message = server.next_message();
        if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        {
            let version = message["params"]["version"].as_i64();
            assert!(
                version.is_some_and(|value| value >= 3),
                "stale diagnostics after version 3: {message}"
            );
            continue;
        }
        if message.get("method").is_none()
            && message.get("id").and_then(Value::as_i64) == Some(hover_id)
        {
            break;
        }
    }

    server.shutdown();
}
