use super::{Scanner, SpannedToken, Token};

impl<'a> Scanner<'a> {
    pub(super) fn scan_variable_token(&mut self) -> Result<(), usize> {
        if self.peek() != Some(b'$') {
            return Err(self.pos);
        }
        let start = self.pos;
        self.pos += 1;
        let name_start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            self.pos += 1;
        }
        if self.pos == name_start {
            return Err(self.pos);
        }
        let name = self.source[name_start..self.pos].to_string();
        self.tokens.push(SpannedToken { token: Token::Variable(name), span: start..self.pos });
        Ok(())
    }

    pub(super) fn scan_function_token(&mut self) -> Result<(), usize> {
        if self.peek() != Some(b':') {
            return Err(self.pos);
        }
        let start = self.pos;
        self.pos += 1;
        let name_start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            self.pos += 1;
        }
        if self.pos == name_start {
            return Err(self.pos);
        }
        let name = self.source[name_start..self.pos].to_string();
        self.tokens.push(SpannedToken { token: Token::Function(name), span: start..self.pos });
        Ok(())
    }

    pub(super) fn scan_name(&mut self) -> Result<(), usize> {
        let start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.pos);
        }
        let name = self.source[start..self.pos].to_string();
        self.tokens.push(SpannedToken { token: Token::Name(name), span: start..self.pos });
        Ok(())
    }

    pub(super) fn scan_number(&mut self) -> Result<(), usize> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.peek() == Some(b'.') && self.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if self.pos == start || (self.pos == start + 1 && self.bytes[start] == b'-') {
            return Err(self.pos);
        }
        let num = self.source[start..self.pos].to_string();
        self.tokens.push(SpannedToken { token: Token::Number(num), span: start..self.pos });
        Ok(())
    }

    pub(super) fn scan_quoted_literal(&mut self) -> Result<(), usize> {
        if self.peek() != Some(b'|') {
            return Err(self.pos);
        }
        let start = self.pos;
        self.pos += 1;
        let content_start = self.pos;
        while !self.is_at_end() && self.peek() != Some(b'|') {
            self.pos += 1;
        }
        if self.is_at_end() {
            return Err(self.pos);
        }
        let content = self.source[content_start..self.pos].to_string();
        self.pos += 1;
        self.tokens
            .push(SpannedToken { token: Token::QuotedLiteral(content), span: start..self.pos });
        Ok(())
    }

    pub(super) fn scan_char_token(&mut self, ch: u8, token: Token) -> Result<(), usize> {
        if self.peek() != Some(ch) {
            return Err(self.pos);
        }
        self.emit(token, 1);
        Ok(())
    }

    pub(super) fn emit(&mut self, token: Token, len: usize) {
        let start = self.pos;
        self.pos += len;
        self.tokens.push(SpannedToken { token, span: start..self.pos });
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    pub(super) fn starts_with(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
            && self.bytes.get(self.pos + s.len()).is_none_or(|b| !b.is_ascii_alphanumeric())
    }

    pub(super) fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|b| b == b' ' || b == b'\t') {
            self.pos += 1;
        }
    }

    pub(super) fn skip_whitespace_and_newlines_with_tokens(&mut self) {
        loop {
            if self.peek().is_some_and(|b| b == b' ' || b == b'\t') {
                self.pos += 1;
            } else if self.peek() == Some(b'\n') {
                self.emit(Token::Newline, 1);
            } else if self.peek() == Some(b'\r') {
                if self.peek_at(1) == Some(b'\n') {
                    self.emit(Token::Newline, 2);
                } else {
                    self.emit(Token::Newline, 1);
                }
            } else {
                break;
            }
        }
    }
}
