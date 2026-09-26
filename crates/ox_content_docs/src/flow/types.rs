//! Rewrites inside type annotations: maybe types, variance, bounds, and
//! indexers.

use super::{CODE, LITERAL, Rewriter, is_ident, is_ident_start};

impl Rewriter<'_> {
    /// `%checks` / `%checks(expr)` predicates.
    pub(super) fn predicate(&mut self, i: usize) {
        let end = i + 7;
        if !self.src[i + 1..].starts_with(b"checks")
            || self.src.get(end).is_some_and(|&b| is_ident(b))
        {
            return;
        }
        self.blank(i, end);
        if let Some(open) = self.next_sig(end)
            && self.src[open] == b'('
            && let Some(close) = self.find_close(open)
        {
            self.blank(open, close + 1);
        }
    }

    /// `?Type` maybe types. A `?` directly after these tokens can never be a
    /// ternary or optional chain.
    pub(super) fn maybe_type(&mut self, i: usize) {
        if matches!(self.code_byte(i + 1), Some(b'.' | b'?' | b':' | b'=')) {
            return;
        }
        let Some(prev) = self.prev_sig(i) else { return };
        let allowed = match self.sig_byte(prev) {
            b':' | b'<' | b',' | b'(' | b'|' | b'&' | b'=' | b'[' => true,
            b'>' => prev > 0 && self.src[prev - 1] == b'=',
            _ => false,
        };
        if allowed {
            self.blank(i, i + 1);
        }
    }

    /// `+prop` / `-prop` variance sigils in object types, classes, and type
    /// parameters. Dropping a unary sign in value position still parses.
    pub(super) fn variance(&mut self, i: usize) {
        let sign = self.src[i];
        if matches!(self.src.get(i + 1), Some(&next) if next == sign || next == b'=') {
            return;
        }
        let Some(&next) = self.src.get(i + 1) else { return };
        let prev = self.prev_sig(i);
        let after_separator = prev.is_none_or(|prev| {
            matches!(self.sig_byte(prev), b',' | b';' | b'<')
                || self.after_object_open(prev)
                || self.word_before(prev + 1) == Some(b"static")
        });
        if !(after_separator || self.starts_line(i)) {
            return;
        }
        if next == b'[' {
            self.blank(i, i + 1);
            return;
        }
        let name = if next == b'#' { i + 2 } else { i + 1 };
        let name_end = match self.kind.get(name) {
            // `+'quoted-key': T`
            Some(&LITERAL) => {
                (name..self.src.len()).find(|&index| self.kind[index] != LITERAL).unwrap_or(name)
            }
            Some(&CODE) if is_ident_start(self.src[name]) => self.word_end(name),
            _ => return,
        };
        let Some(follow) = self.next_sig(name_end) else { return };
        let in_type_params = prev.is_some_and(|prev| matches!(self.sig_byte(prev), b'<' | b','));
        let is_member = match self.src[follow] {
            b':' => true,
            b'?' => self.next_sig(follow + 1).is_some_and(|colon| self.src[colon] == b':'),
            b',' | b'>' | b'=' => in_type_params,
            _ => false,
        };
        if is_member {
            self.blank(i, i + 1);
        }
    }

    /// `{ [K in Keys]: V, }` — TypeScript mapped types take no `,`.
    pub(super) fn mapped_type_separator(&mut self, open: usize) {
        let is_member = self.prev_sig(open).is_some_and(|prev| {
            matches!(self.sig_byte(prev), b'{' | b',' | b';' | b'+' | b'-' | b'|')
        });
        let Some(close) = self.find_close(open).filter(|_| is_member) else { return };
        let Some(key) = self.next_word(open + 1) else {
            self.unnamed_indexer(open, close);
            return;
        };
        if self.next_word(key.1).is_none_or(|(start, end)| &self.src[start..end] != b"in") {
            self.unnamed_indexer(open, close);
            return;
        }
        let Some(comma) = self.depth_zero_in(open, self.src.len(), b",;}", true) else { return };
        if self.src[comma] == b',' {
            self.out[comma] = b';';
        }
    }

    /// `{[keyof T]: V}` — Flow indexers may omit the key name. TypeScript
    /// would read the brackets as a computed key, so make it the identifier
    /// `k`; docs still show the original key type.
    pub(super) fn unnamed_indexer(&mut self, open: usize, close: usize) {
        let Some(first) = self.next_sig(open + 1).filter(|&first| first < close) else { return };
        let named = self.kind[first] == CODE
            && is_ident_start(self.src[first])
            && self
                .next_sig(self.word_end(first))
                .is_some_and(|after| after == close || matches!(self.src[after], b':' | b'?'));
        if !named && self.next_sig(close + 1).is_some_and(|colon| self.src[colon] == b':') {
            self.blank(open + 1, close);
            self.out[open + 1] = b'k';
        }
    }

    /// `*` existential types become the type reference `_`.
    pub(super) fn existential(&mut self, i: usize) {
        let Some(prev) = self.prev_sig(i) else { return };
        if !matches!(self.sig_byte(prev), b'<' | b',' | b':' | b'(' | b'=' | b'|' | b'&') {
            return;
        }
        if self.next_sig(i + 1).is_some_and(|next| {
            matches!(self.src[next], b'>' | b',' | b')' | b'|' | b'&' | b';' | b']' | b'}' | b'=')
        }) {
            self.out[i] = b'_';
        }
    }

    /// `<T: Bound, +U>` — Flow bounds use `:` where TypeScript uses `extends`.
    pub(super) fn type_parameter_bounds(&mut self, open: usize) {
        let after_name = (open > 0 && self.kind[open - 1] == CODE && is_ident(self.src[open - 1]))
            || self.prev_sig(open).and_then(|prev| self.word_before(prev + 1)) == Some(b"function");
        if !after_name && !self.types[open] {
            return;
        }
        let Some(close) = self.find_angle_close(open) else { return };

        let mut segment = open + 1;
        while segment < close {
            let segment_end = self.angle_segment_end(segment, close);
            let Some(mut name) = self.next_sig(segment).filter(|&name| name < segment_end) else {
                break;
            };
            if matches!(self.src[name], b'+' | b'-') {
                self.blank(name, name + 1);
                name += 1;
            }
            if is_ident_start(self.src[name]) {
                let name_end = self.word_end(name);
                if let Some(colon) = self.next_sig(name_end).filter(|&colon| colon < segment_end)
                    && self.src[colon] == b':'
                {
                    let bound_end = self.angle_bound_end(colon, segment_end);
                    self.blank(colon, bound_end);
                }
            }
            segment = segment_end + 1;
        }
    }
}
