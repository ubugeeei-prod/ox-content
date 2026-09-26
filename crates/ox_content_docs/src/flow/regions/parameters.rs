//! Parameter and return type annotations.

use super::super::{CODE, Rewriter, is_expression_keyword};

/// Words before `(` that make it a control-flow head rather than parameters.
const CONTROL_KEYWORDS: &[&[u8]] =
    &[b"if", b"for", b"while", b"switch", b"catch", b"with", b"return", b"typeof", b"await"];

enum Parameter {
    /// `name?: T` — the index of the `?`.
    Optional(usize),
    /// `name: T` without a default.
    Required,
    Other,
}

impl Rewriter<'_> {
    /// Parameter annotations and the return type of a parameter list.
    pub(super) fn mark_parameter_list(&mut self, open: usize) {
        let Some(close) = self.find_close(open) else { return };
        let Some(follow) = self.next_sig(close + 1) else { return };
        let is_parameters =
            self.src[follow..].starts_with(b"=>") || matches!(self.src[follow], b'{' | b':');
        if !is_parameters || self.kind[follow] != CODE {
            return;
        }
        let head = self.prev_sig(open).and_then(|prev| self.word_before(prev + 1));
        if head.is_some_and(|word| CONTROL_KEYWORDS.contains(&word)) {
            return;
        }
        let is_constructor = head == Some(b"constructor");
        let is_setter = self
            .prev_sig(open)
            .and_then(|prev| self.prev_word_start(prev + 1))
            .and_then(|name| self.prev_sig(name))
            .and_then(|prev| self.word_before(prev + 1))
            == Some(b"set");
        // Named functions and methods; an arrow's return type ends at `=>`.
        let named = head.is_some_and(|word| word != b"async" && !is_expression_keyword(word));

        let mut optional = Vec::new();
        let mut start = open + 1;
        while start < close {
            let end = self.depth_zero_comma(start, close).unwrap_or(close);
            match self.mark_parameter(start, end) {
                Parameter::Optional(question) => optional.push(question),
                // Flow allows required parameters after optional ones.
                Parameter::Required => {
                    for question in std::mem::take(&mut optional) {
                        self.blank(question, question + 1);
                    }
                }
                Parameter::Other => {}
            }
            start = end + 1;
        }

        // `cond ? call(x) : other` — only declarations have return types.
        if self.src[follow] == b':' && (!named || self.is_declaration_name(open)) {
            let return_end = self.return_type_end(follow + 1, named);
            if is_constructor || is_setter {
                // TypeScript rejects `constructor(): void`.
                self.blank(follow, return_end);
            } else {
                self.mark(follow + 1, return_end);
            }
        }
    }

    fn mark_parameter(&mut self, start: usize, end: usize) -> Parameter {
        let rest = self
            .next_sig(start)
            .is_some_and(|first| first < end && self.src[first..].starts_with(b"..."));
        let mut depth = 0usize;
        for index in start..end {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth = depth.saturating_sub(1),
                b'=' if depth == 0 => return Parameter::Other,
                b':' if depth == 0 => {
                    let annotation_end = self.annotation_end(index + 1, b"=,)").min(end);
                    self.mark(index + 1, annotation_end);
                    let has_default = self.src.get(annotation_end) == Some(&b'=');
                    let question =
                        self.prev_sig(index).filter(|&question| self.src[question] == b'?');
                    if rest && let Some(question) = question {
                        // `...rest?: T[]`
                        self.blank(question, question + 1);
                        return Parameter::Other;
                    }
                    return match question {
                        // `base?: T = x` — TypeScript rejects `?` with a default.
                        Some(question) if has_default => {
                            self.blank(question, question + 1);
                            Parameter::Other
                        }
                        Some(question) => Parameter::Optional(question),
                        None if has_default || rest => Parameter::Other,
                        None => Parameter::Required,
                    };
                }
                _ => {}
            }
        }
        Parameter::Other
    }

    /// End of a return type: the body `{`, `;`, or (for arrows) `=>`.
    fn return_type_end(&self, from: usize, named: bool) -> usize {
        let mut depth = 0usize;
        for index in from..self.src.len() {
            if self.kind[index] != CODE {
                continue;
            }
            match self.src[index] {
                b'{' if depth == 0
                    && self.prev_sig(index).is_some_and(|prev| {
                        !matches!(self.sig_byte(prev), b':' | b'|' | b'&' | b'<' | b',' | b'(')
                    }) =>
                {
                    return index;
                }
                b'=' if !named && depth == 0 && self.src.get(index + 1) == Some(&b'>') => {
                    return index;
                }
                b';' if depth == 0 => return index,
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    if depth == 0 {
                        return index;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        self.src.len()
    }

    /// Whether the name before the parameter list at `open` is declared there
    /// (a function or method) rather than called.
    fn is_declaration_name(&self, open: usize) -> bool {
        let Some(name_end) = self.prev_sig(open) else { return false };
        let name_end = if self.src[name_end] == b'>' {
            // `name<T>(`: step over the type parameters.
            let mut depth = 0usize;
            let Some(angle) = (0..=name_end).rev().find(|&index| {
                match self.src[index] {
                    b'>' if self.kind[index] == CODE => depth += 1,
                    b'<' if self.kind[index] == CODE => depth -= 1,
                    _ => {}
                }
                depth == 0
            }) else {
                return false;
            };
            match self.prev_sig(angle) {
                Some(end) => end,
                None => return false,
            }
        } else {
            name_end
        };
        let Some(name_start) = self.prev_word_start(name_end + 1) else { return false };
        if self.src[name_start..=name_end] == *b"function" {
            return true;
        }
        let Some(before) = self.prev_sig(name_start) else { return true };
        matches!(self.sig_byte(before), b'{' | b';' | b'}' | b',' | b'*' | b'#' | b'+' | b'-')
            || self.starts_line(name_start)
            || self.word_before(before + 1).is_some_and(|word| {
                matches!(
                    word,
                    b"function"
                        | b"static"
                        | b"get"
                        | b"set"
                        | b"async"
                        | b"declare"
                        | b"export"
                        | b"default"
                        | b"public"
                        | b"private"
                        | b"protected"
                        | b"override"
                )
            })
    }

    fn prev_word_start(&self, end: usize) -> Option<usize> {
        self.word_before(end).map(|word| end - word.len())
    }
}
