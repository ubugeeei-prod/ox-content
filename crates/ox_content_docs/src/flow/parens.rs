//! Parenthesized groups — casts and parameter lists — and object type
//! spreads.

use super::{CODE, RESERVED_TYPE_WORDS, Rewriter, is_ident_start};

impl Rewriter<'_> {
    /// Handles `(expr: Type)` casts and unnamed function type parameters.
    pub(super) fn parenthesized(&mut self, open: usize) {
        let Some(close) = self.find_close(open) else { return };
        let follow = self.next_sig(close + 1);
        let followed_by_arrow = follow.is_some_and(|f| self.src[f..].starts_with(b"=>"));
        // Read the output: an enclosing cast may already have blanked the `:`.
        let followed_by_colon = follow.is_some_and(|f| self.out[f] == b':');

        if followed_by_arrow || followed_by_colon {
            self.unnamed_parameters(open, close);
            return;
        }
        if self.types[open]
            || follow.is_some_and(|f| self.src[f] == b'{')
            || !self.starts_expression(open)
        {
            return;
        }

        // `(value: Type)` → `(value      )`
        let mut depth = 0usize;
        let mut j = open + 1;
        while j < close {
            if self.kind[j] == CODE {
                match self.src[j] {
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' => depth = depth.saturating_sub(1),
                    // A ternary: the `:` belongs to it.
                    b'?' if depth == 0 && !matches!(self.src.get(j + 1), Some(b'.' | b'?')) => {
                        return;
                    }
                    b':' if depth == 0 => {
                        if !self.is_arrow_return_colon(open, j, close) {
                            self.blank(j, close);
                        }
                        return;
                    }
                    _ => {}
                }
            }
            j += 1;
        }
    }

    /// `(() : T => ...)` / `(async (x): T => ...)` — the `:` after an arrow's
    /// parameter list, not a cast.
    pub(super) fn is_arrow_return_colon(
        &self,
        group: usize,
        colon: usize,
        group_close: usize,
    ) -> bool {
        if !self.depth_zero_arrow(colon + 1, group_close) {
            return false;
        }
        let Some(close) = self.prev_sig(colon).filter(|&close| self.src[close] == b')') else {
            return false;
        };
        let Some(params) = self.matching_open(close) else { return false };
        let Some(before) = self.prev_sig(params) else { return false };
        before == group
            || self.src[before] == b'>'
            || self.word_before(before + 1) == Some(b"async")
    }

    /// Flow allows function type parameters without names — `(string, ?Foo)
    /// => void`. Any parameter that is not already a valid TypeScript
    /// parameter becomes a one-letter name, which is also a valid expression
    /// should the group turn out not to be a parameter list.
    pub(super) fn unnamed_parameters(&mut self, open: usize, close: usize) {
        // `(): ((x: T) => U) => body` — a parenthesized return type, not a
        // parameter list.
        if self.prev_sig(open).is_some_and(|prev| {
            self.sig_byte(prev) == b':'
                && self.prev_sig(prev).is_some_and(|before| self.src[before] == b')')
        }) && self.depth_zero_comma(open + 1, close).is_none()
            && self.depth_zero_arrow(open + 1, close)
        {
            return;
        }
        // `(): ?(A | B) => body` — Flow ends an arrow's return type at `=>`.
        let mut before = self.prev_sig(open);
        if before.is_some_and(|prev| self.src[prev] == b'?') {
            before = before.and_then(|prev| self.prev_sig(prev));
        }
        if let Some(colon) = before.filter(|&prev| self.sig_byte(prev) == b':')
            && self.prev_sig(colon).is_some_and(|prev| self.src[prev] == b')')
            && self.src[self.next_sig(close + 1).unwrap_or(close)..].starts_with(b"=>")
        {
            return;
        }
        let mut index = 0u8;
        let mut start = open + 1;
        while start < close {
            let end = self.depth_zero_comma(start, close).unwrap_or(close);
            if let Some(first) = self.next_sig(start).filter(|&first| first < end)
                && !self.is_typescript_parameter(first, end)
            {
                let name = if self.src[first..].starts_with(b"...") { first + 3 } else { first };
                let last = self.prev_sig(end).unwrap_or(name);
                if name <= last {
                    self.blank(name, last + 1);
                    self.out[name] = b'a' + index % 26;
                }
            }
            index = index.wrapping_add(1);
            start = end + 1;
        }
    }

    pub(super) fn is_typescript_parameter(&self, first: usize, end: usize) -> bool {
        if matches!(self.sig_byte(first), b'{' | b'[') {
            // A destructuring pattern — unless it is an unnamed object or
            // tuple type in a function type: `({a?: T, ...}) => void`.
            return !self.types[first]
                || self.find_close(first).is_some_and(|close| {
                    self.next_sig(close + 1)
                        .is_some_and(|after| after < end && matches!(self.src[after], b':' | b'='))
                });
        }
        let name = if self.src[first..].starts_with(b"...") { first + 3 } else { first };
        if self.kind[name] != CODE || !is_ident_start(self.src[name]) {
            return false;
        }
        let name_end = self.word_end(name);
        let Some(after) = self.next_sig(name_end).filter(|&after| after < end) else {
            // `(void) => T` — a keyword type, not a parameter name.
            return !RESERVED_TYPE_WORDS.contains(&&self.src[name..name_end]);
        };
        match self.src[after] {
            b':' => true,
            b'?' => self.code_byte(after + 1) == Some(b':'),
            b'=' => !matches!(self.src.get(after + 1), Some(b'=' | b'>')),
            _ => false,
        }
    }

    /// `{...A, b: T}` object type spreads. Removing a spread element from an
    /// object literal, array, or call in value position still parses.
    pub(super) fn object_type_spread(&mut self, dots: usize) {
        let Some(prev) = self.prev_sig(dots) else { return };
        // `{...}` / `{a: T, ...}` explicitly inexact objects.
        if matches!(self.sig_byte(prev), b'{' | b',' | b';' | b'|' | b'[')
            && let Some(next) = self.next_sig(dots + 3)
        {
            match self.sig_byte(next) {
                b'}' | b']' => return self.blank(dots, dots + 3),
                b'|' if self.code_byte(next + 1) == Some(b'}') => {
                    return self.blank(dots, dots + 3);
                }
                b',' | b';' => return self.blank(dots, next + 1),
                _ => {}
            }
        }
        let allowed = match self.sig_byte(prev) {
            b',' | b';' => true,
            b'|' => prev > 0 && self.src[prev - 1] == b'{',
            b'{' => !self.is_jsx_attribute_brace(prev),
            _ => false,
        };
        if !allowed {
            return;
        }
        if self.types[dots] {
            // Rest parameters of function types stay.
            if self.enclosing_open(dots).is_none_or(|open| self.src[open] != b'(') {
                self.blank_spread_to_separator(dots);
            }
            return;
        }
        let Some(operand) = self.next_sig(dots + 3) else { return };
        let Some(operand_end) = self.type_operand_end(operand) else { return };
        let Some(follow) = self.next_sig(operand_end) else { return };
        match self.src[follow] {
            b',' | b';' => self.blank(dots, follow + 1),
            b'}' => self.blank(dots, operand_end),
            b'|' if self.code_byte(follow + 1) == Some(b'}') => self.blank(dots, operand_end),
            // `...A | B,` — spread of a union.
            b'|' | b'&' => self.blank_spread_to_separator(dots),
            _ => {}
        }
    }

    /// Blanks `...Operand` through its `,`/`;`, or up to the closing `}`/`|}`.
    pub(super) fn blank_spread_to_separator(&mut self, dots: usize) {
        let Some(end) = self.depth_zero_in(dots + 3, self.src.len(), b",;}", true) else {
            return;
        };
        match self.src[end] {
            b'}' if self.src[end - 1] == b'|' => self.blank(dots, end - 1),
            b'}' => self.blank(dots, end),
            _ => self.blank(dots, end + 1),
        }
    }
}
