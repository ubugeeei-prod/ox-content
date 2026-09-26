//! Token-level navigation over the classified source.

use super::{CODE, COMMENT, LITERAL, Rewriter, is_expression_keyword, is_ident, is_ident_start};

impl Rewriter<'_> {
    pub(super) fn blank(&mut self, start: usize, end: usize) {
        for index in start..end.min(self.src.len()) {
            // Ranges start and end on ASCII bytes, so every multi-byte
            // character inside is blanked whole.
            if !matches!(self.src[index], b'\n' | b'\r') {
                self.out[index] = b' ';
            }
        }
    }

    pub(super) fn code_byte(&self, index: usize) -> Option<u8> {
        (index < self.src.len() && self.kind[index] == CODE).then(|| self.src[index])
    }

    /// The byte at a significant index, with every literal token reading as `"`.
    pub(super) fn sig_byte(&self, index: usize) -> u8 {
        if self.kind[index] == CODE { self.src[index] } else { b'"' }
    }

    pub(super) fn is_skippable(&self, index: usize) -> bool {
        self.kind[index] == COMMENT
            || (self.kind[index] == CODE && self.src[index].is_ascii_whitespace())
    }

    pub(super) fn next_sig(&self, from: usize) -> Option<usize> {
        (from..self.src.len()).find(|&index| !self.is_skippable(index))
    }

    pub(super) fn prev_sig(&self, before: usize) -> Option<usize> {
        (0..before).rev().find(|&index| !self.is_skippable(index))
    }

    pub(super) fn continues_word(&self, index: usize) -> bool {
        index > 0 && (is_ident(self.src[index - 1]) || self.src[index - 1] == b'.')
    }

    pub(super) fn word_end(&self, start: usize) -> usize {
        let mut end = start;
        while end < self.src.len() && self.kind[end] == CODE && is_ident(self.src[end]) {
            end += 1;
        }
        end
    }

    pub(super) fn next_word(&self, from: usize) -> Option<(usize, usize)> {
        let start = self.next_sig(from)?;
        (self.kind[start] == CODE && is_ident_start(self.src[start]))
            .then(|| (start, self.word_end(start)))
    }

    /// The next identifier-like word of an import clause.
    pub(super) fn next_word_in_statement(&self, from: usize) -> Option<(usize, usize)> {
        let mut index = from;
        while index < self.src.len() {
            // The module specifier ends the import clause.
            if self.kind[index] == LITERAL {
                return None;
            }
            if self.kind[index] == CODE {
                let byte = self.src[index];
                if byte == b';' {
                    return None;
                }
                if is_ident_start(byte) {
                    return Some((index, self.word_end(index)));
                }
            }
            index += 1;
        }
        None
    }

    pub(super) fn starts_line(&self, index: usize) -> bool {
        self.src[..index]
            .iter()
            .rev()
            .take_while(|&&byte| byte != b'\n')
            .all(|&byte| byte == b' ' || byte == b'\t')
    }

    /// Whether `prev` is the `{` or `{|` opening an object type or body.
    pub(super) fn after_object_open(&self, prev: usize) -> bool {
        match self.sig_byte(prev) {
            b'{' => true,
            b'|' => prev > 0 && self.src[prev - 1] == b'{',
            _ => false,
        }
    }

    /// `<Tag {...props}>` — a brace after a tag name, attribute, or another
    /// attribute value.
    pub(super) fn is_jsx_attribute_brace(&self, brace: usize) -> bool {
        let Some(prev) = self.prev_sig(brace) else { return false };
        match self.sig_byte(prev) {
            b'"' | b'}' => true,
            byte if is_ident(byte) => !self.word_before(prev + 1).is_some_and(|word| {
                is_expression_keyword(word) || matches!(word, b"as" | b"extends" | b"satisfies")
            }),
            _ => false,
        }
    }

    pub(super) fn word_before(&self, end: usize) -> Option<&[u8]> {
        let mut start = end;
        while start > 0 && self.kind[start - 1] == CODE && is_ident(self.src[start - 1]) {
            start -= 1;
        }
        (start < end).then(|| &self.src[start..end])
    }

    /// Whether `(` at `open` begins an expression (rather than a call or a
    /// parameter list).
    pub(super) fn starts_expression(&self, open: usize) -> bool {
        let Some(prev) = self.prev_sig(open) else { return true };
        match self.sig_byte(prev) {
            b')' | b']' | b'"' => false,
            b'>' => prev > 0 && self.src[prev - 1] == b'=',
            byte if is_ident(byte) => self.word_before(prev + 1).is_some_and(is_expression_keyword),
            _ => true,
        }
    }

    /// The matching closer for the bracket at `open`.
    pub(super) fn find_close(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for index in open..self.src.len() {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The matching `>` for a type parameter/argument list, ignoring `=>`.
    pub(super) fn find_angle_close(&self, open: usize) -> Option<usize> {
        let mut angle = 0usize;
        let mut bracket = 0usize;
        for index in open..self.src.len() {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b'<' => angle += 1,
                b'>' if index > 0 && self.src[index - 1] == b'=' => {}
                b'>' => {
                    angle -= 1;
                    if angle == 0 {
                        return (bracket == 0).then_some(index);
                    }
                }
                b'(' | b'[' | b'{' => bracket += 1,
                b')' | b']' | b'}' => bracket = bracket.checked_sub(1)?,
                b';' => return None,
                _ => {}
            }
        }
        None
    }

    /// The innermost unclosed bracket before `index`.
    pub(super) fn enclosing_open(&self, index: usize) -> Option<usize> {
        let mut depth = 0usize;
        for open in (0..index).rev() {
            if self.kind[open] != CODE {
                continue;
            }
            match self.src[open] {
                b')' | b']' | b'}' => depth += 1,
                b'(' | b'[' | b'{' if depth == 0 => return Some(open),
                b'(' | b'[' | b'{' => depth -= 1,
                _ => {}
            }
        }
        None
    }

    pub(super) fn matching_open(&self, close: usize) -> Option<usize> {
        let mut depth = 0usize;
        for index in (0..=close).rev() {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b')' | b']' | b'}' => depth += 1,
                b'(' | b'[' | b'{' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// End of a type parameter at `start`: the next depth-zero `,` or `close`.
    pub(super) fn angle_segment_end(&self, start: usize, close: usize) -> usize {
        self.depth_zero_in(start, close, b",", true).unwrap_or(close)
    }

    /// End of a bound starting at `colon`: the default `=` or the segment end.
    pub(super) fn angle_bound_end(&self, colon: usize, segment_end: usize) -> usize {
        self.depth_zero_in(colon + 1, segment_end, b"=", true).unwrap_or(segment_end)
    }

    pub(super) fn depth_zero_arrow(&self, start: usize, end: usize) -> bool {
        self.depth_zero_in(start, end, b">", false)
            .is_some_and(|index| index > 0 && self.src[index - 1] == b'=')
    }

    pub(super) fn depth_zero_comma(&self, start: usize, end: usize) -> Option<usize> {
        self.depth_zero_in(start, end, b",", true)
    }

    pub(super) fn depth_zero_byte(&self, start: usize, targets: &[u8]) -> Option<usize> {
        self.depth_zero_in(start, self.src.len(), targets, false)
    }

    /// First byte in `targets` at bracket depth zero within `start..end`.
    pub(super) fn depth_zero_in(
        &self,
        start: usize,
        end: usize,
        targets: &[u8],
        angles: bool,
    ) -> Option<usize> {
        let mut depth = 0usize;
        for index in start..end {
            if self.kind[index] != CODE {
                continue;
            }
            let byte = self.src[index];
            let arrow = index > 0 && self.src[index - 1] == b'=' && byte == b'>';
            if depth == 0
                && targets.contains(&byte)
                && !(byte == b'=' && self.src.get(index + 1) == Some(&b'>'))
            {
                return Some(index);
            }
            match byte {
                b'(' | b'[' | b'{' => depth += 1,
                b'<' if angles => depth += 1,
                b'>' if angles && !arrow => depth = depth.saturating_sub(1),
                b')' | b']' | b'}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        None
    }

    /// End of a spread operand: a dotted name with type arguments, or an
    /// object type.
    pub(super) fn type_operand_end(&self, start: usize) -> Option<usize> {
        if self.kind[start] != CODE {
            return None;
        }
        if self.src[start] == b'{' {
            return self.find_close(start).map(|close| close + 1);
        }
        if !is_ident_start(self.src[start]) {
            return None;
        }
        let mut end = self.word_end(start);
        while self.code_byte(end) == Some(b'.')
            && self.src.get(end + 1).is_some_and(|&byte| is_ident_start(byte))
        {
            end = self.word_end(end + 1);
        }
        if self.code_byte(end) == Some(b'<') {
            end = self.find_angle_close(end)? + 1;
        }
        Some(end)
    }
}
