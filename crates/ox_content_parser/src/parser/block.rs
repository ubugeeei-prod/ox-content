use memchr::memchr;
use ox_content_ast::{Heading, Node, Paragraph, Span};

use super::Parser;
use crate::error::{ParseError, ParseResult};
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_block");
        self.skip_blank_lines();

        if self.is_at_end() {
            return Ok(None);
        }

        // Check nesting depth
        if self.nesting_depth > self.options.max_nesting_depth {
            return Err(ParseError::NestingTooDeep {
                span: Span::new(self.position as u32, self.position as u32),
                max_depth: self.options.max_nesting_depth,
            });
        }

        let start = self.position;
        let bytes = self.source.as_bytes();
        let Some(trimmed_start) = self.first_non_whitespace_in_line(start) else {
            return Ok(None);
        };

        // Four columns of indentation start an indented code block; no
        // other block construct can begin on such a line. (This runs at
        // block level only — an indented line after an open paragraph is
        // lazy continuation, handled by `parse_paragraph`.)
        if self.line_indent_width(start, trimmed_start) >= 4 {
            return self.parse_indented_code(start);
        }

        // Fast block dispatch.
        //
        // Most documentation lines are plain paragraph text. The old shape
        // built `line` and `trimmed` up front, then tried each block parser in
        // sequence; that meant every paragraph paid for newline search,
        // trimming, and several failed recognizers. Here the first
        // non-whitespace byte is used as a cheap discriminator. Only marker
        // families that can actually begin with that byte materialize the
        // full line slice and run their more expensive syntax checks.
        //
        // Keep this table in sync with `line_starts_block`: paragraph parsing
        // uses that helper to decide when a following line terminates the
        // paragraph, so the two dispatchers must agree on block starts.
        match bytes[trimmed_start] {
            b'#' if self.try_parse_heading_start(start, trimmed_start) => {
                return self.parse_heading(start);
            }
            b'-' | b'*' => {
                let line = self.line_at(start);
                let trimmed = &line[trimmed_start - start..];
                if Self::try_parse_thematic_break_line(line) {
                    return self.parse_thematic_break(start);
                }
                if Self::try_parse_list_line(trimmed) {
                    return self.parse_list(start);
                }
            }
            b'_' if Self::try_parse_thematic_break_line(self.line_at(start)) => {
                return self.parse_thematic_break(start);
            }
            b'>' => return self.parse_block_quote(start),
            b'`' | b'~' => {
                let line = self.line_at(start);
                let trimmed = &line[trimmed_start - start..];
                if Self::try_parse_fenced_code_at(line, trimmed) {
                    return self.parse_fenced_code(start);
                }
            }
            b'<' => {
                let line = self.line_at(start);
                let trimmed = &line[trimmed_start - start..];
                if let Some(html_start) = Self::parse_html_block_start(trimmed) {
                    return self.parse_html_block(start, html_start);
                }
                // Type-7 blocks (a lone complete tag) start blocks but can
                // never interrupt a paragraph, so only this dispatcher —
                // not line_starts_block — recognizes them.
                if Self::is_html_block_type7_line(trimmed) {
                    return self.parse_html_block(start, super::html::HtmlBlockStart::Other);
                }
            }
            b'+' | b'0'..=b'9' => {
                let line = self.line_at(start);
                let trimmed = &line[trimmed_start - start..];
                if Self::try_parse_list_line(trimmed) {
                    return self.parse_list(start);
                }
            }
            _ => {}
        }

        // Table recognition is the one feature that cannot be decided from
        // the first byte because table headers usually look like ordinary
        // paragraph text. Guard the expensive two-line delimiter check with a
        // same-line `|` probe so non-table prose does one memchr2 scan and
        // then falls through to paragraph parsing.
        if self.options.tables && self.line_contains_byte(start, b'|') && self.try_parse_table() {
            return self.parse_table(start);
        }

        // Link reference definitions look like paragraphs but are
        // consumed as their own (non-rendered) nodes.
        if bytes[trimmed_start] == b'[' {
            if let Some(node) = self.try_parse_definition_node() {
                return Ok(Some(node));
            }
        }

        // Default: parse as paragraph
        self.parse_paragraph(start)
    }

    pub(super) fn parse_paragraph(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_paragraph");
        let bytes = self.source.as_bytes();

        // `parse_block` is the sole caller and only reaches here after
        // `skip_blank_lines` + its block dispatch — the very checks
        // `line_starts_block` re-runs — have already classified the current
        // line as a non-blank, non-block paragraph line. So consume the first
        // line unconditionally instead of re-deriving that verdict with
        // another `current_line` memchr + `trim_start` + dispatch (+ table
        // `memchr`). This also removes the infinite-loop hazard the two
        // dispatchers guard against: by always advancing past line one we can
        // never return `Ok(None)` without progress on a non-blank line.
        let mut content_end = if let Some(off) = memchr(b'\n', &bytes[start..]) {
            start + off + 1
        } else {
            self.source.len()
        };
        self.position = content_end;

        loop {
            if self.is_at_end() {
                break;
            }

            // Check for blank line (paragraph end): scan whitespace and
            // peek the next byte. Cheaper than the prior
            // `skip_whitespace` + `peek` + reset dance.
            let line_start = self.position;
            let mut cursor = line_start;
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            if cursor >= bytes.len() || bytes[cursor] == b'\n' {
                break;
            }

            // Setext heading underline: while a paragraph is open this
            // takes precedence over every block start (`Foo\n---` is an
            // h2, not a paragraph followed by a thematic break), so it
            // must be checked before `line_starts_block`.
            if let Some(depth) = self.setext_underline_depth(line_start, cursor) {
                let heading_end = if let Some(off) = memchr(b'\n', &bytes[line_start..]) {
                    line_start + off + 1
                } else {
                    self.source.len()
                };
                self.position = heading_end;
                let content = self.source[start..content_end].trim();
                let children = self.parse_inline(content, start)?;
                return Ok(Some(Node::Heading(Heading {
                    depth,
                    children,
                    span: Span::new(start as u32, heading_end as u32),
                })));
            }

            // Check for block-level element that would end paragraph.
            if self.line_starts_block() {
                break;
            }

            // Consume one line via memchr.
            content_end = if let Some(off) = memchr(b'\n', &bytes[line_start..]) {
                line_start + off + 1
            } else {
                self.source.len()
            };
            self.position = content_end;
        }

        let content = self.source[start..content_end].trim();
        if content.is_empty() {
            return Ok(None);
        }

        let span = Span::new(start as u32, content_end as u32);

        // Parse inline content
        let children = self.parse_inline(content, start)?;

        Ok(Some(Node::Paragraph(Paragraph { children, span })))
    }

    /// Returns the setext heading depth (1 for `=`, 2 for `-`) when the
    /// line starting at `line_start` is a setext underline: at most three
    /// leading spaces, a run of a single marker character, and nothing but
    /// trailing whitespace. `first_non_ws` is the position of the line's
    /// first non-space/tab byte (already computed by the paragraph loop).
    fn setext_underline_depth(&self, line_start: usize, first_non_ws: usize) -> Option<u8> {
        let bytes = self.source.as_bytes();
        // A tab in the indent always reaches column 4+, so spaces only.
        if first_non_ws - line_start > 3
            || bytes[line_start..first_non_ws].iter().any(|&byte| byte != b' ')
        {
            return None;
        }
        let marker = bytes[first_non_ws];
        let depth = match marker {
            b'=' => 1,
            b'-' => 2,
            _ => return None,
        };
        let mut i = first_non_ws;
        while i < bytes.len() && bytes[i] == marker {
            i += 1;
        }
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] != b'\n' {
            return None;
        }
        Some(depth)
    }
}
