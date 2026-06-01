use memchr::memchr;
use ox_content_ast::{Html, Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

/// Classification of an HTML block opener on the current line.
///
/// `parse_block` already has the current line and its trimmed view in hand.
/// Carrying this compact value into `parse_html_block` avoids reparsing that
/// same line after taking `&mut self`, while still preserving the exact block
/// kind decisions needed by the Markdown HTML block rules implemented here.
#[derive(Clone, Copy)]
pub(super) enum HtmlBlockStart {
    /// An HTML comment block beginning with `<!--`.
    Comment,
    /// A type-1 raw HTML block whose closing tag can appear after blank lines.
    Type1(Type1HtmlBlockTag),
    /// A regular supported HTML block that ends before the next blank line.
    Other,
}

/// Supported type-1 HTML block tags.
///
/// These are stored as an enum instead of a borrowed `&str` so
/// `parse_html_block` can keep mutating the parser cursor without holding an
/// immutable borrow into `self.source`.
#[derive(Clone, Copy)]
pub(super) enum Type1HtmlBlockTag {
    Pre,
    Script,
    Style,
    Textarea,
}

impl Type1HtmlBlockTag {
    fn closing_name(self) -> &'static [u8] {
        match self {
            Self::Pre => b"pre",
            Self::Script => b"script",
            Self::Style => b"style",
            Self::Textarea => b"textarea",
        }
    }
}

impl<'a> Parser<'a> {
    /// Classifies a trimmed line as a supported HTML block opener.
    ///
    /// The caller passes `trimmed` because `parse_block` and
    /// `line_starts_block` have already paid for `trim_start()`. Returning the
    /// full block kind lets the parser reuse that dispatch work in the actual
    /// parse step.
    pub(super) fn parse_html_block_start(trimmed: &str) -> Option<HtmlBlockStart> {
        if trimmed.starts_with("<!--") {
            return Some(HtmlBlockStart::Comment);
        }

        let tag_name = Self::parse_html_block_tag_name_from_trimmed(trimmed)?;
        Self::html_block_start_for_tag(tag_name)
    }

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

    /// Returns the raw tag name from a line already known to begin with `<`.
    ///
    /// This intentionally does not check whether the tag is supported. Keeping
    /// syntax extraction separate from support classification lets
    /// `html_block_start_for_tag` return the exact `HtmlBlockStart` variant in
    /// one pass.
    fn parse_html_block_tag_name_from_trimmed(trimmed: &str) -> Option<&str> {
        let after_open = trimmed.strip_prefix('<')?;
        let after_slash = after_open.strip_prefix('/').unwrap_or(after_open);
        let mut tag_len = 0;

        for byte in after_slash.as_bytes() {
            if byte.is_ascii_alphanumeric() || *byte == b'-' {
                tag_len += 1;
            } else {
                break;
            }
        }

        if tag_len == 0 {
            return None;
        }

        let tag_name = &after_slash[..tag_len];
        let next = after_slash.as_bytes().get(tag_len).copied();

        if let Some(byte) = next {
            if !matches!(byte, b' ' | b'\t' | b'>' | b'/') {
                return None;
            }
        }

        Some(tag_name)
    }

    /// Maps a parsed tag name to the supported HTML block kind.
    ///
    /// The previous implementation walked a fixed slice of tag strings for
    /// every `<...>` opener. Length bucketing keeps this allocation-free and
    /// trims comparisons on generated API docs with many HTML blocks.
    fn html_block_start_for_tag(tag_name: &str) -> Option<HtmlBlockStart> {
        let other = HtmlBlockStart::Other;
        match tag_name.len() {
            1 if tag_name.eq_ignore_ascii_case("p") => Some(other),
            2 if tag_name.eq_ignore_ascii_case("ol")
                || tag_name.eq_ignore_ascii_case("td")
                || tag_name.eq_ignore_ascii_case("th")
                || tag_name.eq_ignore_ascii_case("tr")
                || tag_name.eq_ignore_ascii_case("ul") =>
            {
                Some(other)
            }
            3 if tag_name.eq_ignore_ascii_case("pre") => {
                Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Pre))
            }
            3 if tag_name.eq_ignore_ascii_case("div") || tag_name.eq_ignore_ascii_case("nav") => {
                Some(other)
            }
            4 if tag_name.eq_ignore_ascii_case("main") => Some(other),
            5 if tag_name.eq_ignore_ascii_case("style") => {
                Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Style))
            }
            5 if tag_name.eq_ignore_ascii_case("aside")
                || tag_name.eq_ignore_ascii_case("table")
                || tag_name.eq_ignore_ascii_case("tbody")
                || tag_name.eq_ignore_ascii_case("tfoot")
                || tag_name.eq_ignore_ascii_case("thead") =>
            {
                Some(other)
            }
            6 if tag_name.eq_ignore_ascii_case("script") => {
                Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Script))
            }
            6 if tag_name.eq_ignore_ascii_case("dialog")
                || tag_name.eq_ignore_ascii_case("figure")
                || tag_name.eq_ignore_ascii_case("footer")
                || tag_name.eq_ignore_ascii_case("header") =>
            {
                Some(other)
            }
            7 if tag_name.eq_ignore_ascii_case("article")
                || tag_name.eq_ignore_ascii_case("details")
                || tag_name.eq_ignore_ascii_case("section")
                || tag_name.eq_ignore_ascii_case("summary") =>
            {
                Some(other)
            }
            8 if tag_name.eq_ignore_ascii_case("textarea") => {
                Some(HtmlBlockStart::Type1(Type1HtmlBlockTag::Textarea))
            }
            10 if tag_name.eq_ignore_ascii_case("blockquote")
                || tag_name.eq_ignore_ascii_case("figcaption") =>
            {
                Some(other)
            }
            _ => None,
        }
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

/// Returns true when `haystack` contains a case-insensitive `</tag` opener.
///
/// Type-1 HTML blocks close on the first line containing their closing tag.
/// Searching for `<` with `memchr` skips the common case of long text/code
/// lines that contain no tag-looking byte at all.
pub(super) fn ascii_contains_closing_tag(haystack: &str, tag: &[u8]) -> bool {
    let bytes = haystack.as_bytes();
    if bytes.len() < tag.len() + 2 {
        return false;
    }
    let mut search_start = 0;
    while let Some(off) = memchr(b'<', &bytes[search_start..]) {
        let i = search_start + off;
        if i + tag.len() + 2 <= bytes.len() && bytes[i + 1] == b'/' {
            let candidate = &bytes[i + 2..i + 2 + tag.len()];
            if candidate.eq_ignore_ascii_case(tag) {
                return true;
            }
        }
        search_start = i + 1;
    }
    false
}
