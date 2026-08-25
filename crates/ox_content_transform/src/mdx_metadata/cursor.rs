pub(super) struct Cursor<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    pub(super) fn is_eof(&self) -> bool {
        self.i >= self.src.len()
    }

    pub(super) fn rest(&self) -> &'a str {
        self.src.get(self.i..).unwrap_or("")
    }

    pub(super) fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    pub(super) fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += ch.len_utf8();
        Some(ch)
    }

    pub(super) fn skip_ws_and_comments(&mut self) {
        loop {
            let start = self.i;
            while matches!(self.peek(), Some(c) if c.is_whitespace()) {
                self.bump();
            }
            if self.rest().starts_with("//") {
                self.i += 2;
                while let Some(c) = self.peek() {
                    self.bump();
                    if c == '\n' {
                        break;
                    }
                }
            } else if self.rest().starts_with("/*") {
                self.i += 2;
                while !self.is_eof() && !self.rest().starts_with("*/") {
                    self.bump();
                }
                if self.rest().starts_with("*/") {
                    self.i += 2;
                }
            }
            if self.i == start {
                break;
            }
        }
    }

    pub(super) fn eat(&mut self, expected: char) -> bool {
        self.skip_ws_and_comments();
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(super) fn eat_keyword(&mut self, keyword: &str) -> bool {
        self.skip_ws_and_comments();
        let rest = self.rest();
        if !rest.starts_with(keyword) {
            return false;
        }
        let after = rest.get(keyword.len()..).and_then(|s| s.chars().next());
        if after.is_some_and(is_ident_continue) {
            return false;
        }
        self.i += keyword.len();
        true
    }

    pub(super) fn eat_ident(&mut self) -> Option<String> {
        self.skip_ws_and_comments();
        let rest = self.rest();
        let mut chars = rest.chars();
        let first = chars.next()?;
        if !is_ident_start(first) {
            return None;
        }
        let mut len = first.len_utf8();
        for ch in chars {
            if !is_ident_continue(ch) {
                break;
            }
            len += ch.len_utf8();
        }
        let ident = rest[..len].to_string();
        self.i += len;
        Some(ident)
    }

    pub(super) fn eat_string(&mut self) -> Option<String> {
        self.skip_ws_and_comments();
        let Some(quote @ ('\'' | '"')) = self.peek() else {
            return None;
        };
        self.bump();
        let mut out = String::new();
        while let Some(ch) = self.bump() {
            match ch {
                '\\' => {
                    if let Some(escaped) = self.bump() {
                        out.push(escaped);
                    }
                }
                c if c == quote => return Some(out),
                c => out.push(c),
            }
        }
        Some(out)
    }

    pub(super) fn eat_imported_name(&mut self) -> Option<String> {
        if let Some(ident) = self.eat_ident() {
            return Some(ident);
        }
        self.eat_string()
    }

    pub(super) fn skip_balanced(&mut self, open: char, close: char) {
        self.skip_ws_and_comments();
        if self.peek() != Some(open) {
            return;
        }
        self.bump();
        let mut depth = 1u32;
        while let Some(ch) = self.peek() {
            if ch == '\'' || ch == '"' {
                let _ = self.eat_string();
                continue;
            }
            self.bump();
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
        }
    }

    pub(super) fn skip_until_statement_end(&mut self) {
        let mut brace = 0u32;
        let mut paren = 0u32;
        let mut bracket = 0u32;
        while !self.is_eof() {
            self.skip_ws_and_comments();
            match self.peek() {
                None => break,
                Some('\'') | Some('"') => {
                    let _ = self.eat_string();
                }
                Some('{') => {
                    brace += 1;
                    self.bump();
                }
                Some('}') => {
                    brace = brace.saturating_sub(1);
                    self.bump();
                }
                Some('(') => {
                    paren += 1;
                    self.bump();
                }
                Some(')') => {
                    paren = paren.saturating_sub(1);
                    self.bump();
                }
                Some('[') => {
                    bracket += 1;
                    self.bump();
                }
                Some(']') => {
                    bracket = bracket.saturating_sub(1);
                    self.bump();
                }
                Some(';') if brace == 0 && paren == 0 && bracket == 0 => {
                    self.bump();
                    break;
                }
                Some('\n') if brace == 0 && paren == 0 && bracket == 0 => break,
                _ => {
                    self.bump();
                }
            }
        }
    }

    pub(super) fn consume_type_keyword(&mut self) {
        self.i += 4;
    }
}

pub(super) fn is_ident_start(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphabetic()
}

pub(super) fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}
