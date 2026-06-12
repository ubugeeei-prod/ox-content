use crate::error::{I18nError, I18nResult};
use crate::mf2::lexer::Token;

use super::Parser;

impl Parser {
    pub(super) fn check(&self, expected: &Token) -> bool {
        self.pos < self.tokens.len()
            && std::mem::discriminant(&self.tokens[self.pos].token)
                == std::mem::discriminant(expected)
    }

    pub(super) fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(super) fn expect(&mut self, expected: &Token) -> I18nResult<()> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            let found = if self.is_at_end() {
                "end of input".to_string()
            } else {
                format!("{:?}", self.tokens[self.pos].token)
            };
            Err(self.error(&format!("expected {expected:?}, found {found}")))
        }
    }

    pub(super) fn expect_variable(&mut self) -> I18nResult<String> {
        self.try_consume_variable().ok_or_else(|| self.error("expected variable ($name)"))
    }

    pub(super) fn try_consume_variable(&mut self) -> Option<String> {
        if let Some(Token::Variable(name)) = self.tokens.get(self.pos).map(|t| &t.token) {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    pub(super) fn try_consume_name(&mut self) -> Option<String> {
        if let Some(Token::Name(name)) = self.tokens.get(self.pos).map(|t| &t.token) {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    pub(super) fn try_consume_number(&mut self) -> Option<String> {
        if let Some(Token::Number(num)) = self.tokens.get(self.pos).map(|t| &t.token) {
            let num = num.clone();
            self.advance();
            Some(num)
        } else {
            None
        }
    }

    pub(super) fn try_consume_function(&mut self) -> Option<String> {
        if let Some(Token::Function(name)) = self.tokens.get(self.pos).map(|t| &t.token) {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    pub(super) fn try_consume_quoted_literal(&mut self) -> Option<String> {
        if let Some(Token::QuotedLiteral(val)) = self.tokens.get(self.pos).map(|t| &t.token) {
            let val = val.clone();
            self.advance();
            Some(val)
        } else {
            None
        }
    }

    pub(super) fn skip_newlines(&mut self) {
        while self.pos < self.tokens.len() && self.tokens[self.pos].token == Token::Newline {
            self.advance();
        }
    }

    pub(super) fn error(&self, message: &str) -> I18nError {
        let offset = self.tokens.get(self.pos).map_or(0, |t| t.span.start);
        I18nError::Mf2Parse { offset, message: message.to_string() }
    }
}
