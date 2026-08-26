use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Token {
    Literal(Value),
    Identifier(String),
    Eq,
    Ne,
    In,
    And,
    Or,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    End,
}

pub(super) fn lex(source: &str) -> Result<Vec<Token>, String> {
    Lexer { source, cursor: 0 }.lex()
}

impl Token {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Token::Literal(_) => "literal",
            Token::Identifier(_) => "identifier",
            Token::Eq => "==",
            Token::Ne => "!=",
            Token::In => "in",
            Token::And => "and",
            Token::Or => "or",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBracket => "[",
            Token::RBracket => "]",
            Token::Comma => ",",
            Token::End => "end of expression",
        }
    }
}

struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    fn lex(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.cursor < self.source.len() {
            let byte = self.source.as_bytes()[self.cursor];
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => self.cursor += 1,
                b'=' if self.consume(b"==") => tokens.push(Token::Eq),
                b'!' if self.consume(b"!=") => tokens.push(Token::Ne),
                b'(' => self.single(&mut tokens, Token::LParen),
                b')' => self.single(&mut tokens, Token::RParen),
                b'[' => self.single(&mut tokens, Token::LBracket),
                b']' => self.single(&mut tokens, Token::RBracket),
                b',' => self.single(&mut tokens, Token::Comma),
                b'\'' | b'"' => tokens.push(Token::Literal(self.string(byte)?)),
                b'-' | b'0'..=b'9' => tokens.push(Token::Literal(self.number()?)),
                _ if is_identifier_start(byte) => tokens.push(self.identifier()),
                _ => return Err(format!("unexpected character `{}`", byte as char)),
            }
        }
        tokens.push(Token::End);
        Ok(tokens)
    }

    fn single(&mut self, tokens: &mut Vec<Token>, token: Token) {
        self.cursor += 1;
        tokens.push(token);
    }

    fn consume(&mut self, expected: &[u8]) -> bool {
        if self.source.as_bytes()[self.cursor..].starts_with(expected) {
            self.cursor += expected.len();
            return true;
        }
        false
    }

    fn identifier(&mut self) -> Token {
        let start = self.cursor;
        self.cursor += 1;
        while self
            .source
            .as_bytes()
            .get(self.cursor)
            .is_some_and(|byte| is_identifier_continue(*byte))
        {
            self.cursor += 1;
        }
        match &self.source[start..self.cursor] {
            "true" => Token::Literal(Value::Bool(true)),
            "false" => Token::Literal(Value::Bool(false)),
            "null" => Token::Literal(Value::Null),
            "in" => Token::In,
            "and" => Token::And,
            "or" => Token::Or,
            value => Token::Identifier(value.to_string()),
        }
    }

    fn string(&mut self, quote: u8) -> Result<Value, String> {
        self.cursor += 1;
        let mut out = String::new();
        while self.cursor < self.source.len() {
            let byte = self.source.as_bytes()[self.cursor];
            if byte == quote {
                self.cursor += 1;
                return Ok(Value::String(out));
            }
            if byte == b'\\' {
                self.cursor += 1;
                out.push(self.escape()?);
                continue;
            }
            let Some(ch) = self.source[self.cursor..].chars().next() else {
                return Err("unterminated string literal".to_string());
            };
            out.push(ch);
            self.cursor += ch.len_utf8();
        }
        Err("unterminated string literal".to_string())
    }

    fn escape(&mut self) -> Result<char, String> {
        let Some(byte) = self.source.as_bytes().get(self.cursor).copied() else {
            return Err("unterminated string escape".to_string());
        };
        self.cursor += 1;
        Ok(match byte {
            b'\\' => '\\',
            b'\'' => '\'',
            b'"' => '"',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            _ => return Err(format!("unsupported escape `\\{}`", byte as char)),
        })
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.cursor;
        if self.source.as_bytes()[self.cursor] == b'-' {
            self.cursor += 1;
        }
        while self.source.as_bytes().get(self.cursor).is_some_and(u8::is_ascii_digit) {
            self.cursor += 1;
        }
        if self.source.as_bytes().get(self.cursor) == Some(&b'.') {
            self.cursor += 1;
            while self.source.as_bytes().get(self.cursor).is_some_and(u8::is_ascii_digit) {
                self.cursor += 1;
            }
        }
        let raw = &self.source[start..self.cursor];
        let value: Value =
            serde_json::from_str(raw).map_err(|_| format!("invalid number `{raw}`"))?;
        value.is_number().then_some(value).ok_or_else(|| format!("invalid number `{raw}`"))
    }
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
}
