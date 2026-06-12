use super::{Scanner, SpannedToken, Token};

impl<'a> Scanner<'a> {
    pub(super) fn scan_message(&mut self) -> Result<(), usize> {
        self.skip_whitespace_and_newlines_with_tokens();

        if self.peek() == Some(b'.') {
            self.scan_complex_message()?;
        } else {
            self.scan_simple_pattern()?;
        }
        Ok(())
    }

    fn scan_complex_message(&mut self) -> Result<(), usize> {
        loop {
            self.skip_whitespace_and_newlines_with_tokens();
            if self.is_at_end() {
                break;
            }
            if self.starts_with(".input") {
                self.emit(Token::DotInput, 6);
                self.skip_whitespace();
                self.scan_expression()?;
            } else if self.starts_with(".local") {
                self.emit(Token::DotLocal, 6);
                self.skip_whitespace();
                self.scan_variable_token()?;
                self.skip_whitespace();
                self.scan_char_token(b'=', Token::Equals)?;
                self.skip_whitespace();
                self.scan_expression()?;
            } else if self.starts_with(".match") {
                self.emit(Token::DotMatch, 6);
                self.skip_whitespace();
                while !self.is_at_end() && self.peek() != Some(b'\n') && self.peek() != Some(b'\r')
                {
                    if self.peek() == Some(b'$') {
                        self.scan_variable_token()?;
                        self.skip_whitespace();
                    } else {
                        break;
                    }
                }
                self.skip_whitespace_and_newlines_with_tokens();
                while !self.is_at_end() {
                    self.scan_variant()?;
                    self.skip_whitespace_and_newlines_with_tokens();
                }
            } else {
                if self.peek() == Some(b'{') && self.peek_at(1) == Some(b'{') {
                    self.scan_quoted_pattern()?;
                }
                break;
            }
        }
        Ok(())
    }

    fn scan_variant(&mut self) -> Result<(), usize> {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b'{') && self.peek_at(1) == Some(b'{') {
                break;
            }
            if self.is_at_end() {
                return Ok(());
            }
            if self.peek() == Some(b'*') {
                self.emit(Token::Star, 1);
            } else if self.peek().is_some_and(|b| b.is_ascii_alphabetic() || b == b'_') {
                self.scan_name()?;
            } else if self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'-') {
                self.scan_number()?;
            } else {
                return Err(self.pos);
            }
        }
        self.scan_quoted_pattern()?;
        Ok(())
    }

    fn scan_quoted_pattern(&mut self) -> Result<(), usize> {
        if self.peek() != Some(b'{') || self.peek_at(1) != Some(b'{') {
            return Err(self.pos);
        }
        self.emit(Token::DoubleOpenBrace, 2);

        let mut text_start = self.pos;
        while !self.is_at_end() {
            if self.peek() == Some(b'}') && self.peek_at(1) == Some(b'}') {
                self.push_text_token(text_start);
                self.emit(Token::DoubleCloseBrace, 2);
                return Ok(());
            } else if self.peek() == Some(b'{') {
                self.push_text_token(text_start);
                self.scan_expression()?;
                text_start = self.pos;
            } else {
                self.pos += 1;
            }
        }
        Err(self.pos)
    }

    fn scan_simple_pattern(&mut self) -> Result<(), usize> {
        let mut text_start = self.pos;
        while !self.is_at_end() {
            if self.peek() == Some(b'{') {
                self.push_text_token(text_start);
                self.scan_expression()?;
                text_start = self.pos;
            } else {
                self.pos += 1;
            }
        }
        self.push_text_token(text_start);
        Ok(())
    }

    fn scan_expression(&mut self) -> Result<(), usize> {
        if self.peek() != Some(b'{') {
            return Err(self.pos);
        }
        self.emit(Token::OpenBrace, 1);
        self.skip_whitespace();

        if self.peek() == Some(b'$') {
            self.scan_variable_token()?;
        } else if self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'-') {
            self.scan_number()?;
        } else if self.peek() == Some(b'|') {
            self.scan_quoted_literal()?;
        }

        self.skip_whitespace();

        if self.peek() == Some(b':') {
            self.scan_function_token()?;
            self.skip_whitespace();
            while !self.is_at_end() && self.peek() != Some(b'}') {
                if self.peek().is_some_and(|b| b.is_ascii_alphabetic() || b == b'_') {
                    self.scan_option()?;
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }

        self.skip_whitespace();
        if self.peek() != Some(b'}') {
            return Err(self.pos);
        }
        self.emit(Token::CloseBrace, 1);
        Ok(())
    }

    fn scan_option(&mut self) -> Result<(), usize> {
        self.scan_name()?;
        self.skip_whitespace();
        if self.peek() == Some(b'=') {
            self.emit(Token::Equals, 1);
            self.skip_whitespace();
            if self.peek() == Some(b'$') {
                self.scan_variable_token()?;
            } else if self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'-') {
                self.scan_number()?;
            } else if self.peek() == Some(b'|') {
                self.scan_quoted_literal()?;
            } else if self.peek().is_some_and(|b| b.is_ascii_alphabetic() || b == b'_') {
                self.scan_name()?;
            } else {
                return Err(self.pos);
            }
        }
        Ok(())
    }

    fn push_text_token(&mut self, text_start: usize) {
        if self.pos > text_start {
            let text = self.source[text_start..self.pos].to_string();
            self.tokens.push(SpannedToken { token: Token::Text(text), span: text_start..self.pos });
        }
    }
}
