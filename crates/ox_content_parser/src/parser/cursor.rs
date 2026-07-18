use memchr::{memchr, memchr2};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Returns the remaining source.
    pub(super) fn remaining(&self) -> &'a str {
        &self.source[self.position..]
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
        if b < 0x80 {
            Some(b as char)
        } else {
            self.source[pos..].chars().next()
        }
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
            } else {
                self.position = line_start;
                return;
            }
        }
    }

    pub(super) fn current_line(&self) -> &'a str {
        let bytes = self.source.as_bytes();
        let end = memchr(b'\n', &bytes[self.position..])
            .map_or(self.source.len(), |off| self.position + off);
        &self.source[self.position..end]
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
        let bytes = self.source.as_bytes();
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };

        // A line indented four or more columns cannot start any block, so
        // it can never interrupt a paragraph either (lazy continuation).
        if self.line_indent_width(line_start, trimmed_start) >= 4 {
            return false;
        }

        let starts_block = match bytes[trimmed_start] {
            b'#' => self.try_parse_heading_start(line_start, trimmed_start),
            b'-' | b'*' => {
                let line = self.line_at(line_start);
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_thematic_break_line(line) || Self::try_parse_list_line(trimmed)
            }
            b'_' => Self::try_parse_thematic_break_line(self.line_at(line_start)),
            b'>' => true,
            b'`' | b'~' => {
                let line = self.line_at(line_start);
                let trimmed = &line[trimmed_start - line_start..];
                Self::try_parse_fenced_code_at(line, trimmed)
            }
            b'<' => {
                let line = self.line_at(line_start);
                Self::parse_html_block_start(&line[trimmed_start - line_start..]).is_some()
            }
            b'+' | b'0'..=b'9' => {
                let line = self.line_at(line_start);
                Self::try_parse_list_line(&line[trimmed_start - line_start..])
            }
            _ => false,
        };

        starts_block
            || (self.options.tables
                && self.line_contains_byte(line_start, b'|')
                && self.try_parse_table())
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

    /// Checks whether `needle` appears before the end of the current line.
    ///
    /// `memchr2` searches for either `needle` or `\n` in one pass. This is
    /// used as a guard for rare syntax families such as tables: the common
    /// no-marker case stops at the newline and avoids constructing the line
    /// slice or running the full parser for that feature.
    pub(super) fn line_contains_byte(&self, line_start: usize, needle: u8) -> bool {
        let bytes = self.source.as_bytes();
        match memchr2(needle, b'\n', &bytes[line_start..]) {
            Some(off) => bytes[line_start + off] == needle,
            None => false,
        }
    }
}
