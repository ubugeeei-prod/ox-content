use memchr::memchr;

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

    pub(super) fn line_starts_block(&self) -> bool {
        let line = self.current_line();
        let trimmed = line.trim_start();
        let first = trimmed.as_bytes().first().copied();

        // NB: dispatch arms must use the same checks as `parse_block`,
        // which test at `self.position` (the un-trimmed offset). Using
        // `is_atx_heading_prefix(trimmed.as_bytes())` here would diverge
        // from `parse_block`'s `Some(b'#') if self.try_parse_heading_at()`
        // check on indented inputs like `" # heading"`:
        // `line_starts_block` would say "yes, a heading", `parse_block`
        // would say "no" (since the byte at position 0 is a space) and
        // fall through to `parse_paragraph`, which would immediately break
        // with no content — `parse_block` then returns `Ok(None)` without
        // advancing position, and the outer `parse()` loop spins forever
        // on the same offset. We route every arm through the same
        // line-cached helpers as `parse_block` so the two dispatchers stay
        // in lock-step and we only `memchr` + `trim_start` the current
        // line once per call.
        let starts_block = match first {
            Some(b'#') => self.try_parse_heading_at(trimmed),
            Some(b'-' | b'*') => {
                Self::try_parse_thematic_break_line(line) || Self::try_parse_list_line(trimmed)
            }
            Some(b'_') => Self::try_parse_thematic_break_line(line),
            Some(b'>') => true,
            Some(b'`' | b'~') => Self::try_parse_fenced_code_at(line, trimmed),
            Some(b'<') => self.try_parse_html_block(),
            Some(b'+' | b'0'..=b'9') => Self::try_parse_list_line(trimmed),
            _ => false,
        };

        starts_block
            || (self.options.tables
                && memchr(b'|', line.as_bytes()).is_some()
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
}
