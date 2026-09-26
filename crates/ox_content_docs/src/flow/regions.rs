//! Marks the byte ranges that are type positions.
//!
//! A few Flow rewrites are only safe where the parser expects a type — for
//! example, the unparenthesized function type `T => U` looks exactly like an
//! arrow function. This pass marks type alias bodies, `declare` and
//! `interface` statements, and the annotations of parameters, return types,
//! variables, and class properties.

use super::{CODE, Rewriter, is_ident, is_ident_start};

mod parameters;

/// Words that start a new statement on the next line.
const STATEMENT_KEYWORDS: &[&[u8]] = &[
    b"export",
    b"import",
    b"type",
    b"declare",
    b"const",
    b"let",
    b"var",
    b"function",
    b"class",
    b"opaque",
    b"interface",
    b"module",
    b"component",
    b"hook",
    b"if",
    b"return",
    b"for",
    b"while",
    b"switch",
    b"throw",
    b"try",
    b"async",
    b"enum",
];

impl Rewriter<'_> {
    pub(super) fn mark_type_regions(&mut self) {
        let mut i = 0;
        while i < self.src.len() {
            if self.kind[i] != CODE {
                i += 1;
                continue;
            }
            let byte = self.src[i];
            if byte == b'(' {
                self.mark_parameter_list(i);
            } else if byte == b'<' && i > 0 && is_ident(self.src[i - 1]) {
                // Type parameters and arguments: `f<T: X>()`, `Foo<A => B>`.
                if let Some(close) = self.find_angle_close(i) {
                    self.mark(i + 1, close);
                }
            } else if is_ident_start(byte) && !self.continues_word(i) {
                let end = self.word_end(i);
                match &self.src[i..end] {
                    b"type" | b"interface"
                        if self.at_statement_start(i) && self.next_word(end).is_some() =>
                    {
                        let statement_end = self.statement_end(end);
                        self.mark(end, statement_end);
                    }
                    b"declare" if self.at_statement_start(i) => {
                        let statement_end = self.statement_end(end);
                        self.mark(end, statement_end);
                    }
                    b"const" | b"let" | b"var" => self.mark_variable_annotation(end),
                    b"class" => self.mark_class_body(end),
                    // `value as Type`
                    b"as" => {
                        let type_end = self.annotation_end(end, b";,=");
                        self.mark(end, type_end);
                    }
                    _ => {}
                }
                i = end;
                continue;
            }
            i += 1;
        }
    }

    fn mark(&mut self, start: usize, end: usize) {
        let end = end.min(self.src.len());
        if start < end {
            self.types[start..end].fill(true);
        }
    }

    pub(super) fn at_statement_start(&self, index: usize) -> bool {
        let Some(prev) = self.prev_sig(index) else { return true };
        if matches!(self.sig_byte(prev), b';' | b'{' | b'}') || self.starts_line(index) {
            return true;
        }
        self.word_before(prev + 1)
            .is_some_and(|word| matches!(word, b"export" | b"declare" | b"opaque" | b"default"))
    }

    /// End of the statement whose body starts at `from`: a depth-zero `;`, the
    /// enclosing block's `}`, or a line that starts a new statement.
    fn statement_end(&self, from: usize) -> usize {
        let mut depth = 0usize;
        for index in from..self.src.len() {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    if depth == 0 {
                        return index;
                    }
                    depth -= 1;
                }
                b';' if depth == 0 => return index,
                b'\n'
                    if depth == 0
                        && !self.continues_type_across_line(index)
                        && self.next_line_starts_statement(index) =>
                {
                    return index;
                }
                _ => {}
            }
        }
        self.src.len()
    }

    fn next_line_starts_statement(&self, newline: usize) -> bool {
        self.next_word(newline).is_some_and(|(start, end)| {
            self.starts_line(start) && STATEMENT_KEYWORDS.contains(&&self.src[start..end])
        })
    }

    /// End of an annotation starting at `from`: a depth-zero byte in `stops`
    /// (`=` never matches `=>`), or a line break that does not continue the
    /// type.
    fn annotation_end(&self, from: usize, stops: &[u8]) -> usize {
        let mut depth = 0usize;
        for index in from..self.src.len() {
            if self.kind[index] != CODE {
                continue;
            }
            let byte = self.src[index];
            let arrow = byte == b'=' && self.src.get(index + 1) == Some(&b'>');
            let arrow_head = byte == b'>' && index > 0 && self.src[index - 1] == b'=';
            if depth == 0 && stops.contains(&byte) && !arrow && !arrow_head {
                return index;
            }
            match byte {
                b'(' | b'[' | b'{' | b'<' => depth += 1,
                b'>' if !arrow_head => depth = depth.saturating_sub(1),
                b')' | b']' | b'}' => {
                    if depth == 0 {
                        return index;
                    }
                    depth -= 1;
                }
                b'\n' if depth == 0 && !self.continues_type_across_line(index) => return index,
                _ => {}
            }
        }
        self.src.len()
    }

    fn continues_type_across_line(&self, newline: usize) -> bool {
        let before = self.prev_sig(newline).map(|prev| self.sig_byte(prev));
        let after = self.next_sig(newline).map(|next| self.sig_byte(next));
        matches!(before, Some(b'|' | b'&' | b':' | b',' | b'<' | b'(' | b'=' | b'>' | b'?'))
            || matches!(after, Some(b'|' | b'&' | b'=' | b'.'))
    }

    /// `const name: Type = value`
    fn mark_variable_annotation(&mut self, keyword_end: usize) {
        let Some(name) = self.next_sig(keyword_end) else { return };
        let name_end = match self.src[name] {
            b'{' | b'[' => match self.find_close(name) {
                Some(close) => close + 1,
                None => return,
            },
            byte if is_ident_start(byte) => self.word_end(name),
            _ => return,
        };
        if let Some(colon) = self.next_sig(name_end)
            && self.src[colon] == b':'
            && self.kind[colon] == CODE
        {
            let end = self.annotation_end(colon + 1, b"=;,");
            self.mark(colon + 1, end);
        }
    }

    /// `name: Type` property annotations directly inside a class body.
    fn mark_class_body(&mut self, keyword_end: usize) {
        let Some(open) = self.class_body_open(keyword_end) else { return };
        let Some(close) = self.find_close(open) else { return };
        let mut depth = 0usize;
        let mut index = open + 1;
        while index < close {
            if self.kind[index] == CODE {
                match self.src[index] {
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' => depth = depth.saturating_sub(1),
                    b':' if depth == 0
                        && self.prev_sig(index).is_some_and(|prev| {
                            let byte = self.sig_byte(prev);
                            is_ident(byte) || matches!(byte, b'?' | b']')
                        }) =>
                    {
                        let end = self.annotation_end(index + 1, b";=}").min(close);
                        self.mark(index + 1, end);
                        index = end;
                        continue;
                    }
                    _ => {}
                }
            }
            index += 1;
        }
    }

    /// The `{` opening a class body, skipping type arguments in the heritage.
    fn class_body_open(&self, from: usize) -> Option<usize> {
        let mut angle = 0usize;
        let mut index = from;
        while index < self.src.len() {
            if self.kind[index] == CODE {
                match self.src[index] {
                    b'<' => angle += 1,
                    b'>' => angle = angle.saturating_sub(1),
                    b'{' if angle == 0 => return Some(index),
                    // `extends Base<{ a: T }>` / `extends mixin(Base)`
                    b'{' | b'(' => index = self.find_close(index)?,
                    b';' => return None,
                    _ => {}
                }
            }
            index += 1;
        }
        None
    }
}
