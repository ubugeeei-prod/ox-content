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
        &self,
        paragraph: &Paragraph<'a>,
        mut skip_chars: usize,
    ) -> String {
        let mut renderer = HtmlRenderer::with_options(self.options.clone());
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
                    renderer.write_escaped(value);
                }
                _ => {
                    before_body = false;
                    renderer.render_node(child);
                }
            }
        }

        renderer.output
    }

    fn detect_callout<'a>(paragraph: &Paragraph<'a>) -> Option<(CalloutKind, usize)> {
        // Fast bail: a callout marker is `[!KIND]...` so the very first
        // text byte must be `[`. The previous version unconditionally
        // allocated a `String prefix` and pushed Text values into it
        // before checking — pure waste for the overwhelmingly common
        // case of a regular block quote.
        let mut iter = paragraph.children.iter();
        let Node::Text(first_text) = iter.next()? else {
            return None;
        };
        if first_text.value.as_bytes().first() != Some(&b'[') {
            return None;
        }

        // The first Text node almost always contains the entire marker
        // (parsers don't split `[!NOTE]` across multiple Text nodes
        // unless inline markup interleaves). Try in-place first, and
        // only fall back to the concatenating slow path if the marker
        // straddles nodes.
        if let Some((kind, remainder)) = CalloutKind::parse_marker(first_text.value) {
            let consumed = first_text.value.len().saturating_sub(remainder.len());
            return Some((kind, consumed));
        }

        let mut prefix = String::from(first_text.value);
        for child in iter {
            let Node::Text(text) = child else {
                return None;
            };
            prefix.push_str(text.value);
            if let Some((kind, remainder)) = CalloutKind::parse_marker(&prefix) {
                let consumed = prefix.len().saturating_sub(remainder.len());
                return Some((kind, consumed));
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

        let paragraph_body =
            self.render_paragraph_with_skipped_text_prefix(first_paragraph, consumed_chars);
        if !paragraph_body.trim().is_empty() {
            self.write("<p>");
            self.write(&paragraph_body);
            self.write("</p>\n");
        }

        for child in block_quote.children.iter().skip(1) {
            self.render_node(child);
        }

        self.write("</blockquote>\n");
        true
    }
}
