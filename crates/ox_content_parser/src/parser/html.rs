use memchr::memchr;
use ox_content_ast::{Html, Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

mod scan;
mod start;

pub(super) use scan::ascii_contains_closing_tag;
pub(super) use start::HtmlBlockStart;

impl<'a> Parser<'a> {
    /// Parses an HTML block previously classified by `parse_html_block_start`.
    ///
    /// `block_start` is trusted to match the current line. This is why the
    /// function no longer rechecks the opener: the outer dispatcher owns that
    /// responsibility, and this function only advances `self.position` to the
    /// end of the block.
    pub(super) fn parse_html_block(
        &mut self,
        start: usize,
        block_start: HtmlBlockStart,
    ) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_html_block");

        match block_start {
            HtmlBlockStart::Comment => loop {
                let consumed = self.consume_line();
                if consumed.contains("-->") || self.is_at_end() {
                    break;
                }
            },
            HtmlBlockStart::Terminated(terminator) => loop {
                let consumed = self.consume_line();
                if consumed.contains(terminator) || self.is_at_end() {
                    break;
                }
            },
            HtmlBlockStart::Type1(tag) => {
                // Type 1 blocks (`<pre>`, `<script>`, `<style>`, `<textarea>`)
                // close on the first line containing `</tag`. The block tag
                // was classified by `parse_block`, so we avoid reparsing the
                // first line here and only scan the body.
                let tag_bytes = tag.closing_name();
                loop {
                    let consumed = self.consume_line();
                    if ascii_contains_closing_tag(consumed, tag_bytes) || self.is_at_end() {
                        break;
                    }
                }
            }
            HtmlBlockStart::Other => {
                self.consume_line();
                self.advance_html_block_until_blank();
            }
        }

        let span = Span::new(start as u32, self.position as u32);
        let value = &self.source[start..self.position];
        Ok(Some(Node::Html(Html { value, span })))
    }

    /// Advances through a regular HTML block until the next blank line.
    ///
    /// The cursor must stop *before* that blank line so the outer block parser
    /// can consume it through `skip_blank_lines`. This mirrors the old
    /// `consume_line` + rollback behavior but avoids constructing a line slice
    /// and rescanning it for every nonblank line.
    fn advance_html_block_until_blank(&mut self) {
        let bytes = self.source.as_bytes();
        let mut pos = self.position;

        while pos < bytes.len() {
            let line_start = pos;
            let mut scan = pos;

            while scan < bytes.len() {
                match bytes[scan] {
                    b'\n' => {
                        self.position = line_start;
                        return;
                    }
                    b' ' | b'\t' | b'\r' => scan += 1,
                    _ => break,
                }
            }

            if scan >= bytes.len() {
                self.position = line_start;
                return;
            }

            pos = memchr(b'\n', &bytes[scan..]).map_or(bytes.len(), |off| scan + off + 1);
        }

        self.position = pos;
    }
}
