use std::path::{Path, PathBuf};

use rustc_hash::FxHashSet;

use ox_content_ast::{Node, Span};

use crate::line_index::LineIndex;
use crate::target::{anchor_of, classify, split_anchor};
use crate::{Diagnostic, LinkKind, Severity};

pub struct Walker<'src, 'opts> {
    diagnostics: Vec<Diagnostic>,
    line_index: &'src LineIndex,
    anchors: &'src FxHashSet<String>,
    base_dir: Option<&'opts Path>,
    src_dir: Option<&'opts Path>,
    ignore_patterns: &'opts [String],
}

impl<'src, 'opts> Walker<'src, 'opts> {
    pub fn new(
        line_index: &'src LineIndex,
        anchors: &'src FxHashSet<String>,
        base_dir: Option<&'opts Path>,
        src_dir: Option<&'opts Path>,
        ignore_patterns: &'opts [String],
    ) -> Self {
        Self { diagnostics: Vec::new(), line_index, anchors, base_dir, src_dir, ignore_patterns }
    }

    pub fn walk(&mut self, nodes: &[Node<'src>]) {
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

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
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
            // document. Defer it to a follow-up so this change stays
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
