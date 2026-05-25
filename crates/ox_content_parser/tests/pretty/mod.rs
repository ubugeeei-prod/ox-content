//! Deterministic pretty-printer for the Markdown AST used by snapshot tests.
//!
//! Produces an indented tree representation that captures node kind, all
//! semantic attributes, and source spans. The output is stable across runs
//! (no addresses, no allocator-dependent ordering), so any structural drift
//! shows up as a snapshot diff rather than a hand-rolled assertion failure.

#![allow(dead_code)]

use std::fmt::Write as _;

use ox_content_ast::{AlignKind, Document, Node, Span};

pub fn format_document(doc: &Document<'_>, source: &str, out: &mut String) {
    line(out, 0, format_args!("Document {}", span(doc.span, source)));
    for child in &doc.children {
        format_node(child, source, 1, out);
    }
}

fn format_node(node: &Node<'_>, source: &str, depth: usize, out: &mut String) {
    match node {
        Node::Paragraph(p) => {
            line(out, depth, format_args!("Paragraph {}", span(p.span, source)));
            for child in &p.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::Heading(h) => {
            line(out, depth, format_args!("Heading depth={} {}", h.depth, span(h.span, source)));
            for child in &h.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::ThematicBreak(t) => {
            line(out, depth, format_args!("ThematicBreak {}", span(t.span, source)));
        }
        Node::BlockQuote(b) => {
            line(out, depth, format_args!("BlockQuote {}", span(b.span, source)));
            for child in &b.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::List(l) => {
            line(
                out,
                depth,
                format_args!(
                    "List ordered={} start={} spread={} {}",
                    l.ordered,
                    l.start.map_or_else(|| "-".to_string(), |s| s.to_string()),
                    l.spread,
                    span(l.span, source),
                ),
            );
            for item in &l.children {
                format_list_item(item, source, depth + 1, out);
            }
        }
        Node::ListItem(item) => format_list_item(item, source, depth, out),
        Node::CodeBlock(c) => {
            line(
                out,
                depth,
                format_args!(
                    "CodeBlock lang={:?} meta={:?} value={:?} {}",
                    c.lang,
                    c.meta,
                    c.value,
                    span(c.span, source),
                ),
            );
        }
        Node::Html(h) => {
            line(out, depth, format_args!("Html value={:?} {}", h.value, span(h.span, source)));
        }
        Node::Table(t) => {
            let aligns = t.align.iter().copied().map(align_to_str).collect::<Vec<_>>().join(",");
            line(out, depth, format_args!("Table align=[{}] {}", aligns, span(t.span, source)));
            for row in &t.children {
                line(out, depth + 1, format_args!("TableRow {}", span(row.span, source)));
                for cell in &row.children {
                    line(out, depth + 2, format_args!("TableCell {}", span(cell.span, source)));
                    for child in &cell.children {
                        format_node(child, source, depth + 3, out);
                    }
                }
            }
        }
        Node::Text(t) => {
            line(out, depth, format_args!("Text {:?} {}", t.value, span(t.span, source)));
        }
        Node::Emphasis(e) => {
            line(out, depth, format_args!("Emphasis {}", span(e.span, source)));
            for child in &e.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::Strong(s) => {
            line(out, depth, format_args!("Strong {}", span(s.span, source)));
            for child in &s.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::InlineCode(c) => {
            line(out, depth, format_args!("InlineCode {:?} {}", c.value, span(c.span, source)));
        }
        Node::Break(b) => {
            line(out, depth, format_args!("Break {}", span(b.span, source)));
        }
        Node::Link(l) => {
            line(
                out,
                depth,
                format_args!("Link url={:?} title={:?} {}", l.url, l.title, span(l.span, source)),
            );
            for child in &l.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::Image(i) => {
            line(
                out,
                depth,
                format_args!(
                    "Image url={:?} alt={:?} title={:?} {}",
                    i.url,
                    i.alt,
                    i.title,
                    span(i.span, source),
                ),
            );
        }
        Node::Delete(d) => {
            line(out, depth, format_args!("Delete {}", span(d.span, source)));
            for child in &d.children {
                format_node(child, source, depth + 1, out);
            }
        }
        Node::FootnoteReference(f) => {
            line(
                out,
                depth,
                format_args!(
                    "FootnoteReference identifier={:?} label={:?} {}",
                    f.identifier,
                    f.label,
                    span(f.span, source),
                ),
            );
        }
        Node::Definition(d) => {
            line(
                out,
                depth,
                format_args!(
                    "Definition identifier={:?} label={:?} url={:?} title={:?} {}",
                    d.identifier,
                    d.label,
                    d.url,
                    d.title,
                    span(d.span, source),
                ),
            );
        }
        Node::FootnoteDefinition(f) => {
            line(
                out,
                depth,
                format_args!(
                    "FootnoteDefinition identifier={:?} label={:?} {}",
                    f.identifier,
                    f.label,
                    span(f.span, source),
                ),
            );
            for child in &f.children {
                format_node(child, source, depth + 1, out);
            }
        }
    }
}

fn format_list_item(
    item: &ox_content_ast::ListItem<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    line(
        out,
        depth,
        format_args!(
            "ListItem spread={} checked={} {}",
            item.spread,
            match item.checked {
                None => "-",
                Some(true) => "true",
                Some(false) => "false",
            },
            span(item.span, source),
        ),
    );
    for child in &item.children {
        format_node(child, source, depth + 1, out);
    }
}

fn span(span: Span, source: &str) -> String {
    let len = source.len() as u32;
    if span.start <= len && span.end <= len && span.start <= span.end {
        format!("[{}..{}]", span.start, span.end)
    } else {
        format!("[{}..{} INVALID]", span.start, span.end)
    }
}

fn align_to_str(a: AlignKind) -> &'static str {
    match a {
        AlignKind::None => "none",
        AlignKind::Left => "left",
        AlignKind::Center => "center",
        AlignKind::Right => "right",
    }
}

fn line(out: &mut String, depth: usize, args: std::fmt::Arguments<'_>) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    let _ = out.write_fmt(args);
    out.push('\n');
}
