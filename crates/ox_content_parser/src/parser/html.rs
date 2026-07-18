use memchr::{memchr, memmem};
use ox_content_ast::{Html, Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

mod start;

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
            // Types 2-5 close on the first line CONTAINING their
            // terminator, so one whole-block substring search replaces the
            // per-line `contains` loop. The opener line participates in
            // the search (a one-line `<!-- ... -->` closes immediately),
            // exactly like the old first `consume_line` iteration did.
            HtmlBlockStart::Comment => self.advance_html_block_past(b"-->"),
            HtmlBlockStart::Terminated(terminator) => {
                self.advance_html_block_past(terminator.as_bytes());
            }
            HtmlBlockStart::Type1(tag) => {
                // Type 1 blocks (`<pre>`, `<script>`, `<style>`,
                // `<textarea>`) close on the first line containing
                // `</tag`, searched case-insensitively across the whole
                // remaining source in one scan.
                let bytes = self.source.as_bytes();
                self.position = match find_closing_tag(bytes, self.position, tag.closing_name()) {
                    Some(at) => end_of_line_after(bytes, at),
                    None => bytes.len(),
                };
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

    /// Advances just past the line that contains `needle`, or to EOF when
    /// the remaining source never contains it.
    fn advance_html_block_past(&mut self, needle: &[u8]) {
        let bytes = self.source.as_bytes();
        self.position = match memmem::find(&bytes[self.position..], needle) {
            Some(off) => end_of_line_after(bytes, self.position + off + needle.len()),
            None => bytes.len(),
        };
    }

    /// Advances through a regular HTML block until the next blank line.
    ///
    /// The cursor must stop *before* that blank line so the outer block
    /// parser can consume it through `skip_blank_lines`. A single memchr
    /// newline iterator walks the block; each line costs one first-byte
    /// check unless it actually starts with whitespace.
    fn advance_html_block_until_blank(&mut self) {
        let bytes = self.source.as_bytes();
        let start = self.position;
        let mut line_start = start;

        for newline_off in memchr::memchr_iter(b'\n', &bytes[start..]) {
            let newline = start + newline_off;
            if is_blank_line(&bytes[line_start..newline]) {
                self.position = line_start;
                return;
            }
            line_start = newline + 1;
        }

        // Trailing line without a newline: a whitespace-only remainder
        // stays unconsumed, anything else belongs to the block.
        if line_start >= bytes.len() || is_blank_line(&bytes[line_start..]) {
            self.position = line_start.min(bytes.len());
        } else {
            self.position = bytes.len();
        }
    }
}

/// Position of the first case-insensitive `</tag` at or after `from`.
///
/// Type-1 HTML blocks close on the first line containing their closing
/// tag; searching for `<` with `memchr` skips the common case of long
/// text/code runs that contain no tag-looking byte at all.
pub(in crate::parser) fn find_closing_tag(bytes: &[u8], from: usize, tag: &[u8]) -> Option<usize> {
    let mut search = from;
    while let Some(off) = memchr(b'<', &bytes[search..]) {
        let at = search + off;
        if at + tag.len() + 2 <= bytes.len()
            && bytes[at + 1] == b'/'
            && bytes[at + 2..at + 2 + tag.len()].eq_ignore_ascii_case(tag)
        {
            return Some(at);
        }
        search = at + 1;
    }
    None
}

/// True when `line` holds only spaces, tabs, and carriage returns
/// (including the empty line).
#[inline]
fn is_blank_line(line: &[u8]) -> bool {
    line.iter().all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
}

/// Byte offset just past the newline of the line containing `at` (or EOF).
#[inline]
fn end_of_line_after(bytes: &[u8], at: usize) -> usize {
    memchr(b'\n', &bytes[at..]).map_or(bytes.len(), |off| at + off + 1)
}
