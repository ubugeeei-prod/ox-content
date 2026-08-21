//! Rendering support for GitHub-style callout block quotes.
//!
//! Callouts are encoded as normal block quotes in the AST. These helpers detect the
//! marker in the first paragraph, strip it from the body, and emit the themed wrapper
//! while leaving non-callout block quotes on the regular rendering path.

use ox_content_ast::{BlockQuote, Node, Paragraph};

use super::super::callout::CalloutKind;
use super::HtmlRenderer;

impl HtmlRenderer {
    fn render_paragraph_with_skipped_text_prefix<'a>(
        &mut self,
        paragraph: &Paragraph<'a>,
        mut skip_chars: usize,
    ) {
        let paragraph_start = self.output.len();
        self.write("<p>");
        let body_start = self.output.len();

        // The former temporary renderer had no initialized autolink index,
        // so callout body text was escaped without bare-URL rewriting. Keep
        // that observable behavior while writing into the existing buffer.
        let autolink_index = self.autolink_index.take();
        // Whitespace-only text between the marker and the body (the parser
        // may emit the separating newline as its own Text node) is part of
        // the marker line, not body content.
        let mut before_body = true;

        for child in &paragraph.children {
            match child {
                Node::Text(text) if skip_chars > 0 || before_body => {
                    let mut value = text.value;
                    if skip_chars > 0 {
                        if skip_chars >= value.len() {
                            skip_chars -= value.len();
                            continue;
                        }
                        value = &value[skip_chars..];
                        skip_chars = 0;
                    }
                    value = value.trim_start();
                    if value.is_empty() {
                        continue;
                    }
                    before_body = false;
                    self.write_escaped(value);
                }
                _ => {
                    before_body = false;
                    self.render_node(child);
                }
            }
        }
        self.autolink_index = autolink_index;

        if self.output[body_start..].trim().is_empty() {
            self.output.truncate(paragraph_start);
        } else {
            self.write("</p>\n");
        }
    }

    fn detect_callout<'a>(paragraph: &Paragraph<'a>) -> Option<(CalloutKind, usize)> {
        // Fast bail: a callout marker is `[!KIND]...` so the very first
        // text byte must be `[`. The previous version unconditionally
        // allocated a `String prefix` and pushed Text values into it
        // before checking — pure waste for the overwhelmingly common
        // case of a regular block quote.
        let Node::Text(first_text) = paragraph.children.first()? else {
            return None;
        };
        if first_text.value.as_bytes().first() != Some(&b'[') {
            return None;
        }

        // Keep the overwhelmingly common contiguous case to one slice scan.
        if let Some((kind, remainder)) = CalloutKind::parse_marker(first_text.value) {
            let consumed = first_text.value.len().saturating_sub(remainder.len());
            return Some((kind, consumed));
        }

        // Failed link parsing can split `[!NOTE]` into adjacent Text nodes.
        // Walk those nodes as one logical marker without concatenating them.
        // `IMPORTANT` is the longest accepted name, so every candidate that
        // could succeed fits in this stack buffer.
        let mut name = [0u8; 9];
        let mut name_len = 0usize;
        let mut consumed = 0usize;
        let mut state = 0u8;
        let mut trailing_name_whitespace = false;

        for child in &paragraph.children {
            let Node::Text(text) = child else {
                return None;
            };

            for ch in text.value.chars() {
                consumed += ch.len_utf8();
                match state {
                    0 if ch == '[' => state = 1,
                    1 if ch == '!' => state = 2,
                    0 | 1 => return None,
                    _ if ch == ']' => {
                        let name = std::str::from_utf8(&name[..name_len]).ok()?;
                        return Some((CalloutKind::from_name(name)?, consumed));
                    }
                    _ if ch.is_whitespace() => {
                        trailing_name_whitespace |= name_len != 0;
                    }
                    _ => {
                        if trailing_name_whitespace || !ch.is_ascii() || name_len == name.len() {
                            return None;
                        }
                        name[name_len] = ch as u8;
                        name_len += 1;
                    }
                }
            }
        }

        None
    }

    pub(in crate::html::renderer) fn render_callout_block_quote<'a>(
        &mut self,
        block_quote: &BlockQuote<'a>,
    ) -> bool {
        let Some(Node::Paragraph(first_paragraph)) = block_quote.children.first() else {
            return false;
        };
        let Some((kind, consumed_chars)) = Self::detect_callout(first_paragraph) else {
            return false;
        };

        self.write("<blockquote class=\"ox-callout ox-callout--");
        self.write(kind.class_name());
        self.write("\">\n");
        self.write("<p class=\"ox-callout-title\">");
        self.write(kind.label());
        self.write("</p>\n");

        self.render_paragraph_with_skipped_text_prefix(first_paragraph, consumed_chars);

        for child in block_quote.children.iter().skip(1) {
            self.render_node(child);
        }

        self.write("</blockquote>\n");
        true
    }
}
