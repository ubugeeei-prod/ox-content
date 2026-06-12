use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

/// A live `ox-content-lsp` process plus the minimal JSON-RPC plumbing
/// needed to drive it.
pub struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    pub fn start() -> Self {
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

    /// Sends a request and returns the id to match the response against.
    /// A `Null` params is omitted entirely. Methods like `shutdown` take
    /// no params and reject an explicit `null`.
    pub fn request(&mut self, method: &str, params: Value) -> i64 {
        self.next_id += 1;
        let id = self.next_id;
        let mut message = json!({ "jsonrpc": "2.0", "id": id, "method": method });
        if !params.is_null() {
            message["params"] = params;
        }
        self.write_message(&message);
        id
    }

    /// Reads messages until the response to `id` arrives, skipping the
    /// notifications the server emits in between. Returns the `result`
    /// payload, asserting the call did not fail.
    pub fn await_response(&mut self, id: i64) -> Value {
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
    pub fn await_notification(&mut self, method: &str) -> Value {
        loop {
            let message = self.read_message();
            if message.get("method").and_then(Value::as_str) == Some(method) {
                return message.get("params").cloned().unwrap_or(Value::Null);
            }
        }
    }

    /// Runs the standard handshake and opens `uri` with `text` as a
    /// Markdown document.
    pub fn initialize_and_open(&mut self, uri: &str, text: &str) -> Value {
        self.initialize_and_open_with(uri, text, json!({}))
    }

    /// Like [`initialize_and_open`], but forwards `init_options` as the
    /// server's `initializationOptions`.
    pub fn initialize_and_open_with(
        &mut self,
        uri: &str,
        text: &str,
        init_options: Value,
    ) -> Value {
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

    pub fn shutdown(&mut self) {
        let id = self.request("shutdown", Value::Null);
        let _ = self.await_response(id);
        self.notify("exit", Value::Null);
    }

    /// Sends a notification with no response expected.
    pub fn notify(&mut self, method: &str, params: Value) {
        let mut message = json!({ "jsonrpc": "2.0", "method": method });
        if !params.is_null() {
            message["params"] = params;
        }
        self.write_message(&message);
    }

    /// Writes one `Content-Length`-framed JSON-RPC message.
    fn write_message(&mut self, message: &Value) {
        let body = serde_json::to_string(message).expect("serialize message");
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .expect("write to server");
        self.stdin.flush().expect("flush server stdin");
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
                break;
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
pub fn temp_uri(name: &str) -> String {
    let mut dir = std::env::temp_dir();
    dir.push("ox-content-lsp-e2e");
    dir.push(name);
    format!("file://{}", dir.display())
}
