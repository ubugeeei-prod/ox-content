//! textlint integration.
//!
//! Runs [textlint](https://textlint.github.io) as a subprocess and
//! converts its JSON output into LSP diagnostics. The subprocess
//! contract is intentionally narrow:
//!
//! ```bash
//! <command> --format json --stdin --stdin-filename <path>
//! ```
//!
//! `<command>` defaults to `npx textlint` so the LSP works against
//! any project that installed textlint as a devDependency. Users with
//! a global binary or a custom wrapper can override it via
//! `oxContent.textlint.command`.
//!
//! The integration runs **only on save** to keep the typing path
//! fast — textlint can take a few hundred milliseconds per file and
//! we don't want to queue subprocess spawns on every keystroke. The
//! `did_save` LSP handler triggers the run; `did_change` skips it.

use std::path::Path;

use serde::Deserialize;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// User-facing configuration. Defaults are picked so the integration
/// is a no-op unless the user opts in (textlint is heavy and noisy
/// for projects that don't use it).
#[derive(Clone, Debug, Default)]
pub struct TextlintConfig {
    /// Whether to run textlint at all.
    pub enabled: bool,
    /// Shell command to invoke. Empty falls back to `npx textlint`.
    pub command: Option<String>,
}

impl TextlintConfig {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.enabled
    }

    fn argv(&self) -> Vec<String> {
        let raw = self.command.as_deref().map(str::trim).filter(|c| !c.is_empty());
        match raw {
            Some(c) => shlex_split(c),
            None => vec!["npx".into(), "textlint".into()],
        }
    }
}

/// Parsed textlint JSON output. textlint emits a per-file array of
/// messages even when invoked with a single file via stdin; we only
/// ever ask for the one stdin file so the first entry is the answer.
#[derive(Debug, Deserialize)]
struct TextlintFileResult {
    messages: Vec<TextlintMessage>,
}

#[derive(Debug, Deserialize)]
struct TextlintMessage {
    #[serde(rename = "ruleId")]
    rule_id: Option<String>,
    message: String,
    #[serde(default)]
    line: u32,
    #[serde(default)]
    column: u32,
    /// textlint severity: 1 = warning, 2 = error. Anything else maps
    /// to Information (textlint sometimes emits 0 for fix-only entries).
    #[serde(default)]
    severity: u32,
}

/// Parse a textlint `--format json` payload into LSP diagnostics.
/// Returns an empty list when the payload is empty / malformed —
/// the LSP caller surfaces a log-level warning separately rather
/// than blocking the publish path.
pub fn parse_diagnostics(payload: &str) -> Vec<Diagnostic> {
    let Ok(files) = serde_json::from_str::<Vec<TextlintFileResult>>(payload) else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    for file in files {
        for message in file.messages {
            diagnostics.push(message_to_diagnostic(message));
        }
    }
    diagnostics
}

fn message_to_diagnostic(message: TextlintMessage) -> Diagnostic {
    // textlint reports 1-indexed coordinates; the LSP wants
    // 0-indexed. Clamp at 0 so an over-zealous saturating sub never
    // produces a negative number.
    let line = message.line.saturating_sub(1);
    let column = message.column.saturating_sub(1);
    let position = Position { line, character: column };
    Diagnostic {
        range: Range { start: position, end: position },
        severity: Some(severity_from_textlint(message.severity)),
        source: Some("textlint".to_string()),
        code: message.rule_id.map(tower_lsp::lsp_types::NumberOrString::String),
        message: message.message,
        ..Default::default()
    }
}

fn severity_from_textlint(severity: u32) -> DiagnosticSeverity {
    match severity {
        2 => DiagnosticSeverity::ERROR,
        1 => DiagnosticSeverity::WARNING,
        _ => DiagnosticSeverity::INFORMATION,
    }
}

/// Run textlint on `source`, returning LSP diagnostics. The
/// subprocess is spawned async via tokio so it does not block the
/// LSP request handler. Empty diagnostics on any subprocess error so
/// a broken `textlint` install does not flood the publish channel
/// with red squiggles — the caller logs the failure separately.
pub async fn run(source: &str, file_path: &Path, config: &TextlintConfig) -> Vec<Diagnostic> {
    if !config.is_active() {
        return Vec::new();
    }

    let mut argv = config.argv();
    let Some(program) = argv.first().cloned() else {
        return Vec::new();
    };
    let mut args: Vec<String> = argv.drain(1..).collect();
    args.extend(["--format".into(), "json".into(), "--stdin".into()]);
    args.extend(["--stdin-filename".into(), file_path.to_string_lossy().into()]);

    let Ok(mut child) = tokio::process::Command::new(program)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        return Vec::new();
    };

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt as _;
        if stdin.write_all(source.as_bytes()).await.is_err() {
            return Vec::new();
        }
        // Drop stdin so textlint sees EOF and returns.
        drop(stdin);
    }

    let Ok(output) = child.wait_with_output().await else {
        return Vec::new();
    };

    // textlint exits 1 when there are problems; we still want to
    // parse stdout in that case.
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_diagnostics(&stdout)
}

/// Pure helpers — split a shell-style command line on whitespace,
/// respecting single and double quotes. textlint config strings come
/// straight from `oxContent.textlint.command` so we want to be
/// reasonably forgiving (`npx textlint --no-color` should split into
/// three arguments).
fn shlex_split(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in input.chars() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests;
