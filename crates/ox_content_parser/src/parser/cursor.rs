use memchr::{memchr, memchr2};

use super::Parser;
#[allow(unused_imports)]
use crate::profile_span_detail;

/// What [`Parser::probe_line`] learned about the current line.
pub(super) struct BlockProbe {
    /// Whether the line begins a block-level construct.
    pub starts_block: bool,
    /// Offset of the line's newline, or `source.len()` for an unterminated
    /// final line — but only when the table scan ran all the way to it.
    /// `None` means the caller has to find the line end itself.
    pub line_end: Option<usize>,
}

impl<'a> Parser<'a> {
    pub(super) fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Peeks at the current character.
    #[inline]
    pub(super) fn peek(&self) -> Option<char> {
        // ASCII fast path: most parser hot loops only inspect ASCII bytes
        // (`#`, `\``, `~`, `>`, digits, whitespace), so avoid the cost of
        // constructing a `Chars` iterator and decoding UTF-8 when the
        // current byte is plain ASCII.
        let bytes = self.source.as_bytes();
        let pos = self.position;
        let &b = bytes.get(pos)?;
        if b < 0x80 { Some(b as char) } else { self.source[pos..].chars().next() }
    }

    /// Advances by one character.
    #[inline]
    pub(super) fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Skips whitespace characters.
    pub(super) fn skip_whitespace(&mut self) {
        let bytes = self.source.as_bytes();
        let mut pos = self.position;
        while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
            pos += 1;
        }
        self.position = pos;
    }

    /// Skips blank lines.
    pub(super) fn skip_blank_lines(&mut self) {
        let bytes = self.source.as_bytes();
        let mut pos = self.position;
        loop {
            let line_start = pos;
            // Skip spaces/tabs.
            while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == b'\n' {
                pos += 1;
            } else if pos >= bytes.len() {
                // Spaces/tabs running to end of input with no newline are
                // a blank final line: consume them. Rewinding instead
                // would park the position on whitespace that no block
                // parser consumes, and the block loop — which runs until
                // `is_at_end` — would spin forever.
                self.position = pos;
                return;
            } else {
                // Content follows the indentation; rewind so the caller
                // still sees the leading whitespace (indented code and
                // list markers depend on it).
                self.position = line_start;
                return;
            }
        }
    }

    /// Returns true when the current line begins a block-level construct.
    ///
    /// This is the paragraph-continuation counterpart of `parse_block`'s
    /// first-byte dispatcher. Paragraph parsing calls it for each following
    /// line, so it deliberately avoids `current_line().trim_start()` unless
    /// the leading marker byte makes a block parse plausible. Keeping this
    /// byte-dispatch table aligned with `parse_block` preserves Markdown
    /// behavior while avoiding repeated full-line scans on ordinary prose.
    pub(super) fn line_starts_block(&self) -> bool {
        let line_start = self.position;
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };
        self.probe_line(line_start, trimmed_start).starts_block
    }

    /// [`Self::line_starts_block`] for a caller that has already found the
    /// line's first non-whitespace byte, and that also wants to know where
    /// the line ends.
    ///
    /// The table check scans the line with `memchr2(b'|', b'\n')`. When it
    /// comes back with the newline — the answer for ordinary prose — it has
    /// incidentally answered "where does this line end", which is the very
    /// next thing paragraph parsing asks. Returning that offset lets the
    /// paragraph loop consume the line without a second scan over the same
    /// bytes.
    pub(super) fn probe_line(&self, line_start: usize, trimmed_start: usize) -> BlockProbe {
        self.probe_line_inner(line_start, trimmed_start, self.options.tables)
    }

    /// [`Self::probe_line`] without the table check.
    ///
    /// A table's own body rows are the one place the check is pure waste:
    /// consecutive rows always belong to the same table, yet every one of
    /// them re-ran the two-line header/delimiter probe — peeking both lines,
    /// counting the header's cells, and validating a delimiter row that is
    /// really the next data row — only to answer "no".
    pub(super) fn probe_line_without_table(
        &self,
        line_start: usize,
        trimmed_start: usize,
    ) -> BlockProbe {
        self.probe_line_inner(line_start, trimmed_start, false)
    }

    fn probe_line_inner(
        &self,
        line_start: usize,
        trimmed_start: usize,
        check_tables: bool,
    ) -> BlockProbe {
        profile_span_detail!("parser::line_starts_block");
        let bytes = self.source.as_bytes();

        // A line indented four or more columns cannot start any block, so
        // it can never interrupt a paragraph either (lazy continuation).
        if self.line_indent_width(line_start, trimmed_start) >= 4 {
            return BlockProbe { starts_block: false, line_end: None };
        }

        let starts_block = match bytes[trimmed_start] {
            b'#' => self.try_parse_heading_start(line_start, trimmed_start),
            b'-' | b'*' => {
                let line = self.line_at(line_start);
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_thematic_break_line(line) || Self::try_parse_list_interrupt(trimmed)
            }
            b'_' => Self::try_parse_thematic_break_line(self.line_at(line_start)),
            b'>' => true,
            b'`' | b'~' => {
                let line = self.line_at(line_start);
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_fenced_code_at(line, trimmed)
            }
            b'{' => {
                // `looks_like_flow_expression` walks to the end of the
                // source to report that nothing closed, so a run of lines
                // starting with an unclosed `{` would pay one walk each.
                self.options.mdx
                    && self.has_closer_from(self.source, trimmed_start + 1, b'}')
                    && super::mdx_jsx::looks_like_flow_expression(self.source, trimmed_start)
            }
            b'<' => {
                let bytes = self.source.as_bytes();
                if self.options.mdx && super::mdx_jsx::looks_like_jsx_open(bytes, trimmed_start) {
                    true
                } else {
                    let line = self.line_at(line_start);
                    Self::parse_html_block_start(&line[trimmed_start - line_start..]).is_some()
                }
            }
            b'+' | b'0'..=b'9' => {
                let line = self.line_at(line_start);
                Self::try_parse_list_interrupt(&line[trimmed_start - line_start..])
            }
            b'i' | b'e' => self.options.mdx && super::mdx_esm::looks_like_esm(bytes, trimmed_start),
            _ => false,
        };

        if starts_block {
            return BlockProbe { starts_block: true, line_end: None };
        }
        if !check_tables {
            return BlockProbe { starts_block: false, line_end: None };
        }
        match memchr2(b'|', b'\n', &bytes[line_start..]) {
            Some(off) if bytes[line_start + off] == b'|' => {
                BlockProbe { starts_block: self.try_parse_table(), line_end: None }
            }
            Some(off) => BlockProbe { starts_block: false, line_end: Some(line_start + off) },
            None => BlockProbe { starts_block: false, line_end: Some(self.source.len()) },
        }
    }

    pub(super) fn line_at(&self, line_start: usize) -> &'a str {
        let bytes = self.source.as_bytes();
        let end =
            memchr(b'\n', &bytes[line_start..]).map_or(self.source.len(), |off| line_start + off);
        &self.source[line_start..end]
    }

    pub(super) fn next_line_start(&self, line_start: usize) -> usize {
        let bytes = self.source.as_bytes();
        match memchr(b'\n', &bytes[line_start..]) {
            Some(off) => line_start + off + 1,
            None => self.source.len(),
        }
    }

    pub(super) fn consume_line(&mut self) -> &'a str {
        let start = self.position;
        let bytes = self.source.as_bytes();
        if let Some(off) = memchr(b'\n', &bytes[start..]) {
            let line_end = start + off;
            self.position = line_end + 1;
            &self.source[start..line_end]
        } else {
            self.position = self.source.len();
            &self.source[start..]
        }
    }

    /// Finds the first non-space, non-tab byte before the next newline.
    ///
    /// The parser only treats ASCII space and tab as indentation in these
    /// block-level dispatchers. Returning the byte offset, rather than a
    /// trimmed `&str`, lets callers inspect the discriminator byte first and
    /// defer `line_at()` until they know a full line slice is needed.
    pub(super) fn first_non_whitespace_in_line(&self, line_start: usize) -> Option<usize> {
        let bytes = self.source.as_bytes();
        let mut cursor = line_start;

        while cursor < bytes.len() {
            match bytes[cursor] {
                b' ' | b'\t' => cursor += 1,
                b'\n' => return None,
                _ => return Some(cursor),
            }
        }

        None
    }
}
