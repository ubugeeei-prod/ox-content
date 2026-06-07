//! End-to-end protocol tests for the `ox-content-lsp` binary.
//!
//! These spawn the real compiled server and speak LSP over stdio — the
//! exact JSON-RPC framing an editor uses — so they exercise the wiring
//! that the in-crate unit tests can't: capability advertisement, request
//! dispatch, and the shape of each response on the wire. No editor and no
//! `cargo run` indirection; `cargo test` builds the bin and hands us its
//! path via `CARGO_BIN_EXE_ox-content-lsp`.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

/// A live `ox-content-lsp` process plus the minimal JSON-RPC plumbing
/// needed to drive it.
struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ox-content-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ox-content-lsp");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout"));
        Self { child, stdin, stdout, next_id: 0 }
    }

    /// Writes one `Content-Length`-framed JSON-RPC message.
    fn write_message(&mut self, message: &Value) {
        let body = serde_json::to_string(message).expect("serialize message");
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .expect("write to server");
        self.stdin.flush().expect("flush server stdin");
    }

    /// Sends a request and returns the id to match the response against.
    /// A `Null` params is omitted entirely — methods like `shutdown` take
    /// no params and reject an explicit `null`.
    fn request(&mut self, method: &str, params: Value) -> i64 {
        self.next_id += 1;
        let id = self.next_id;
        let mut message = json!({ "jsonrpc": "2.0", "id": id, "method": method });
        if !params.is_null() {
            message["params"] = params;
        }
        self.write_message(&message);
        id
    }

    /// Sends a notification (no id, no response expected). `Null` params
    /// is omitted for the same reason as in `request`.
    fn notify(&mut self, method: &str, params: Value) {
        let mut message = json!({ "jsonrpc": "2.0", "method": method });
        if !params.is_null() {
            message["params"] = params;
        }
        self.write_message(&message);
    }

    /// Reads one framed message off the server's stdout.
    fn read_message(&mut self) -> Value {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).expect("read header line");
            assert!(read != 0, "server closed stdout while reading headers");
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break; // blank line terminates the header block
            }
            if let Some(value) = trimmed.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>().expect("Content-Length"));
            }
        }

        let len = content_length.expect("message had no Content-Length header");
        let mut body = vec![0u8; len];
        self.stdout.read_exact(&mut body).expect("read message body");
        serde_json::from_slice(&body).expect("parse message body")
    }

    /// Reads messages until the response to `id` arrives, skipping the
    /// notifications (log messages, diagnostics) the server emits in
    /// between. Returns the `result` payload, asserting the call did not
    /// fail.
    fn await_response(&mut self, id: i64) -> Value {
        loop {
            let message = self.read_message();
            // Server-to-client requests also carry an id, so a response is
            // distinguished by the absence of a `method` field.
            let is_response = message.get("method").is_none()
                && message.get("id").and_then(Value::as_i64) == Some(id);
            if is_response {
                assert!(
                    message.get("error").is_none(),
                    "request {id} failed: {}",
                    message["error"]
                );
                return message.get("result").cloned().unwrap_or(Value::Null);
            }
        }
    }

    /// Reads messages until a notification with `method` arrives,
    /// skipping responses and other notifications. Returns its `params`.
    fn await_notification(&mut self, method: &str) -> Value {
        loop {
            let message = self.read_message();
            if message.get("method").and_then(Value::as_str) == Some(method) {
                return message.get("params").cloned().unwrap_or(Value::Null);
            }
        }
    }

    /// Runs the standard handshake and opens `uri` with `text` as a
    /// Markdown document.
    fn initialize_and_open(&mut self, uri: &str, text: &str) -> Value {
        self.initialize_and_open_with(uri, text, json!({}))
    }

    /// Like [`initialize_and_open`], but forwards `init_options` as the
    /// server's `initializationOptions` (used to point the server at a
    /// frontmatter schema).
    fn initialize_and_open_with(&mut self, uri: &str, text: &str, init_options: Value) -> Value {
        let id = self.request(
            "initialize",
            json!({
                "capabilities": {},
                "processId": null,
                "rootUri": null,
                "initializationOptions": init_options,
            }),
        );
        let init_result = self.await_response(id);
        self.notify("initialized", json!({}));
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "markdown",
                    "version": 1,
                    "text": text,
                }
            }),
        );
        init_result
    }

    fn shutdown(&mut self) {
        let id = self.request("shutdown", Value::Null);
        let _ = self.await_response(id);
        self.notify("exit", Value::Null);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Make sure we never leak a server process if an assertion panics
        // before the explicit shutdown.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Builds a `file://` URI under the temp dir. The file never has to exist:
/// document content is delivered over `didOpen`, and link resolution is
/// lexical.
fn temp_uri(name: &str) -> String {
    let mut dir = std::env::temp_dir();
    dir.push("ox-content-lsp-e2e");
    dir.push(name);
    format!("file://{}", dir.display())
}

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
    assert!(folds.contains(&(0, 4)), "missing h1 section fold, got {folds:?}");
    assert!(folds.contains(&(2, 4)), "missing h2 section fold, got {folds:?}");

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
    assert!(folds.contains(&(0, 3)), "missing frontmatter fold, got {folds:?}");

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
    assert!(value.contains("title"), "hover should name the field, got {value:?}");
    assert!(value.contains("string"), "hover should state the type, got {value:?}");

    server.shutdown();
}
