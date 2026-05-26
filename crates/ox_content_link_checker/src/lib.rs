//! Dead link checker for Ox Content Markdown.
//!
//! Resolves every `Link` / `Image` / link reference definition emitted
//! by the parser against the filesystem and against the document's own
//! heading slugs. The checker is intentionally **offline-only**:
//! external HTTP links pass through with no network call so the same
//! binary is safe to run in CI without timeouts, retries, or rate
//! limits, and produces deterministic output across runs. A future
//! `http-check` feature flag can layer network checks on top without
//! changing this contract.
//!
//! ```
//! use ox_content_link_checker::{check_source, CheckOptions};
//! use std::path::PathBuf;
//!
//! let source = "[broken](missing.md)\n";
//! let opts = CheckOptions::for_file(PathBuf::from("/tmp/doc.md"));
//! let diagnostics = check_source(source, &opts);
//! assert_eq!(diagnostics.len(), 1);
//! ```

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ox_content_allocator::Allocator;
use ox_content_ast::{Document, Node, Span};
use ox_content_parser::{Parser, ParserOptions};
use serde::Serialize;

/// Kind of link target that resolution can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkKind {
    /// `[text](./foo.md)` or `<img src="foo.png">`.
    File,
    /// `[text](#anchor)`.
    Anchor,
    /// `[text](./foo.md#anchor)`.
    FileAnchor,
    /// `[text](https://...)` — passed through, never asserted unless
    /// HTTP checking is enabled (currently always disabled).
    External,
    /// `[text](mailto:a@b.example)` etc.
    Scheme,
    /// Could not be parsed (e.g. malformed `mailto:` payload).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub kind: LinkKind,
    /// The raw URL or reference identifier as written in the source.
    pub target: String,
}

/// Inputs that change link resolution outcomes. Constructed by the
/// caller (CLI / LSP) before each `check_source` call.
#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// Absolute path of the file being checked. Used to resolve
    /// relative paths and to skip self-anchor warnings.
    pub file_path: PathBuf,
    /// Treated as the root of paths that start with `/`. Defaults to
    /// the file's directory when `None`.
    pub src_dir: Option<PathBuf>,
    /// Patterns whose match short-circuits diagnostics. Each entry is
    /// matched against the raw target string via plain `contains`; this
    /// is intentionally simple — the LSP/CLI surface can layer
    /// glob/regex on top without changing the checker.
    pub ignore_patterns: Vec<String>,
}

impl CheckOptions {
    pub fn for_file(file_path: PathBuf) -> Self {
        Self { file_path, src_dir: None, ignore_patterns: Vec::new() }
    }
}

/// Run the checker over a Markdown source string and return the
/// (possibly empty) list of diagnostics.
#[must_use]
pub fn check_source(source: &str, options: &CheckOptions) -> Vec<Diagnostic> {
    let allocator = Allocator::for_source_len(source.len());
    let parser = Parser::with_options(&allocator, source, ParserOptions::gfm());
    let Ok(document) = parser.parse() else {
        return Vec::new();
    };

    let line_index = LineIndex::new(source);
    let anchors = collect_anchors(source, &document);
    let base_dir = options.file_path.parent().map(Path::to_path_buf);

    let mut walker = Walker {
        diagnostics: Vec::new(),
        line_index: &line_index,
        anchors: &anchors,
        base_dir: base_dir.as_deref(),
        src_dir: options.src_dir.as_deref(),
        ignore_patterns: &options.ignore_patterns,
    };
    walker.walk(&document.children);
    walker.diagnostics
}

struct Walker<'src, 'opts> {
    diagnostics: Vec<Diagnostic>,
    line_index: &'src LineIndex,
    anchors: &'src HashSet<String>,
    base_dir: Option<&'opts Path>,
    src_dir: Option<&'opts Path>,
    ignore_patterns: &'opts [String],
}

impl<'src> Walker<'src, '_> {
    fn walk(&mut self, nodes: &[Node<'src>]) {
        for node in nodes {
            match node {
                Node::Link(link) => {
                    self.check_target(link.url, link.span, false);
                    self.walk(&link.children);
                }
                Node::Image(image) => {
                    self.check_target(image.url, image.span, true);
                }
                Node::Paragraph(p) => self.walk(&p.children),
                Node::Heading(h) => self.walk(&h.children),
                Node::BlockQuote(b) => self.walk(&b.children),
                Node::List(list) => {
                    for item in &list.children {
                        self.walk(&item.children);
                    }
                }
                Node::Emphasis(e) => self.walk(&e.children),
                Node::Strong(s) => self.walk(&s.children),
                Node::Delete(d) => self.walk(&d.children),
                Node::Table(table) => {
                    for row in &table.children {
                        for cell in &row.children {
                            self.walk(&cell.children);
                        }
                    }
                }
                Node::FootnoteDefinition(f) => self.walk(&f.children),
                _ => {}
            }
        }
    }

    fn check_target(&mut self, url: &str, span: Span, is_image: bool) {
        if url.is_empty() {
            return;
        }
        if self.ignore_patterns.iter().any(|pat| url.contains(pat)) {
            return;
        }

        let target = url.to_string();
        match classify(&target) {
            LinkKind::External | LinkKind::Scheme => {}
            LinkKind::Anchor => {
                let anchor = anchor_of(&target).unwrap_or_default();
                if !self.anchors.contains(anchor) {
                    self.push(
                        span,
                        LinkKind::Anchor,
                        target.clone(),
                        format!("Anchor `#{anchor}` is not defined in this document."),
                        Severity::Error,
                    );
                }
            }
            LinkKind::File | LinkKind::FileAnchor => {
                self.check_file_target(&target, span, is_image);
            }
            LinkKind::Unknown => {
                self.push(
                    span,
                    LinkKind::Unknown,
                    target.clone(),
                    format!("Could not classify link target `{target}`."),
                    Severity::Warning,
                );
            }
        }
    }

    fn check_file_target(&mut self, target: &str, span: Span, is_image: bool) {
        let (file_part, anchor_part) = split_anchor(target);
        let Some(resolved) = self.resolve_file(file_part) else {
            self.push(
                span,
                LinkKind::File,
                target.to_string(),
                format!("Could not resolve `{file_part}` (no base directory available)."),
                Severity::Error,
            );
            return;
        };

        if !resolved.exists() {
            let label = if is_image { "image" } else { "link" };
            self.push(
                span,
                if anchor_part.is_some() { LinkKind::FileAnchor } else { LinkKind::File },
                target.to_string(),
                format!(
                    "Broken {label} target: `{file_part}` does not exist (resolved to {}).",
                    resolved.display()
                ),
                Severity::Error,
            );
            return;
        }

        if let Some(anchor) = anchor_part {
            // Cross-file anchor validation requires parsing the other
            // document. Defer it to a follow-up so this PR stays
            // local-only; we emit an informational note instead.
            if !anchor.is_empty() {
                self.push(
                    span,
                    LinkKind::FileAnchor,
                    target.to_string(),
                    format!(
                        "Cross-file anchor `#{anchor}` is not validated yet \
                        (file exists, anchor unchecked)."
                    ),
                    Severity::Warning,
                );
            }
        }
    }

    fn resolve_file(&self, raw: &str) -> Option<PathBuf> {
        let path = Path::new(raw);
        if path.is_absolute() {
            // POSIX absolute path inside a Markdown document is a
            // workspace-rooted reference, not a host-absolute one.
            // Strip the leading slash and join under src_dir / base_dir.
            let stripped = raw.trim_start_matches('/');
            let base = self.src_dir.or(self.base_dir)?;
            return Some(base.join(stripped));
        }
        self.base_dir.map(|base| base.join(raw))
    }

    fn push(
        &mut self,
        span: Span,
        kind: LinkKind,
        target: String,
        message: String,
        severity: Severity,
    ) {
        let (line, column) = self.line_index.position(span.start as usize);
        let (end_line, end_column) = self.line_index.position(span.end as usize);
        self.diagnostics.push(Diagnostic {
            severity,
            message,
            line,
            column,
            end_line,
            end_column,
            kind,
            target,
        });
    }
}

fn classify(target: &str) -> LinkKind {
    if target.starts_with('#') {
        return LinkKind::Anchor;
    }
    if let Some(scheme_end) = target.find(':') {
        let scheme = &target[..scheme_end];
        if is_url_scheme(scheme) {
            return if matches!(scheme, "http" | "https") {
                LinkKind::External
            } else {
                LinkKind::Scheme
            };
        }
    }
    if target.contains('#') {
        LinkKind::FileAnchor
    } else {
        LinkKind::File
    }
}

fn is_url_scheme(scheme: &str) -> bool {
    if scheme.is_empty() {
        return false;
    }
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else { return false };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

fn split_anchor(target: &str) -> (&str, Option<&str>) {
    target.split_once('#').map_or((target, None), |(file, anchor)| (file, Some(anchor)))
}

fn anchor_of(target: &str) -> Option<&str> {
    target.strip_prefix('#')
}

fn collect_anchors<'src>(source: &str, document: &'src Document<'src>) -> HashSet<String> {
    let mut anchors = HashSet::new();
    collect_anchors_into(source, &document.children, &mut anchors);
    anchors
}

fn collect_anchors_into<'src>(source: &str, nodes: &'src [Node<'src>], out: &mut HashSet<String>) {
    for node in nodes {
        if let Node::Heading(heading) = node {
            let text = inline_text(source, &heading.children);
            out.insert(slugify(&text));
        }
    }
}

fn inline_text(source: &str, nodes: &[Node<'_>]) -> String {
    let mut buf = String::new();
    flatten(source, nodes, &mut buf);
    buf
}

fn flatten(source: &str, nodes: &[Node<'_>], buf: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) => buf.push_str(t.value),
            Node::InlineCode(c) => buf.push_str(c.value),
            Node::Emphasis(e) => flatten(source, &e.children, buf),
            Node::Strong(s) => flatten(source, &s.children, buf),
            Node::Delete(d) => flatten(source, &d.children, buf),
            Node::Link(l) => flatten(source, &l.children, buf),
            _ => {
                let span = node.span();
                let text = &source[span.start as usize..span.end as usize];
                buf.push_str(text);
            }
        }
    }
}

/// GitHub-style heading slug. Lowercase, strip everything that is not
/// `[a-z0-9 -]`, collapse spaces into `-`. Matches the slug rules
/// `ox_content_renderer` uses, so anchors emitted by the renderer for a
/// given heading round-trip through the checker.
fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('-');
        }
        // Drop everything else (punctuation, emoji, etc.).
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}

struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { line_starts }
    }

    fn position(&self, offset: usize) -> (u32, u32) {
        match self.line_starts.binary_search(&offset) {
            Ok(idx) => (idx as u32 + 1, 1),
            Err(idx) => {
                let line = idx as u32; // idx is the first start *after* offset
                let line_start = self.line_starts[idx - 1];
                let column = (offset - line_start) as u32 + 1;
                (line, column)
            }
        }
    }
}

#[cfg(test)]
mod tests;
