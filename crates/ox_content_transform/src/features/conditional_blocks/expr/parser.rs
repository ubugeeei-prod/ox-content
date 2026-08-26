use super::{Expr, lexer::Token};

pub(super) fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
    let mut parser = Parser { tokens, cursor: 0 };
    let expression = parser.parse_expression()?;
    parser.expect_end()?;
    Ok(expression)
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.bump();
            expr = Expr::Or(Box::new(expr), Box::new(self.parse_and()?));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        while matches!(self.peek(), Token::And) {
            self.bump();
            expr = Expr::And(Box::new(expr), Box::new(self.parse_comparison()?));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;
        match self.peek() {
            Token::Eq => {
                self.bump();
                Ok(Expr::Equal(Box::new(left), Box::new(self.parse_primary()?)))
            }
            Token::Ne => {
                self.bump();
                Ok(Expr::NotEqual(Box::new(left), Box::new(self.parse_primary()?)))
            }
            Token::In => {
                self.bump();
                Ok(Expr::In(Box::new(left), Box::new(self.parse_primary()?)))
            }
            _ => Ok(left),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Token::Literal(value) => Ok(Expr::Literal(value)),
            Token::Identifier(name) => Ok(Expr::Variable(name)),
            Token::LParen => {
                let expr = self.parse_expression()?;
                self.expect(Token::RParen, "expected `)` after grouped expression")?;
                Ok(expr)
            }
            Token::LBracket => self.parse_array(),
            Token::End => Err("expected an expression".to_string()),
            token => Err(format!("unexpected token `{}`", token.name())),
        }
    }

    fn parse_array(&mut self) -> Result<Expr, String> {
        let mut values = Vec::new();
        if matches!(self.peek(), Token::RBracket) {
            self.bump();
            return Ok(Expr::Array(values));
        }
        loop {
            values.push(self.parse_expression()?);
            match self.peek() {
                Token::Comma => {
                    self.bump();
                    if matches!(self.peek(), Token::RBracket) {
                        return Err("expected an array value after `,`".to_string());
                    }
                }
                Token::RBracket => {
                    self.bump();
                    return Ok(Expr::Array(values));
                }
                _ => return Err("expected `,` or `]` in array literal".to_string()),
            }
        }
    }

    fn expect(&mut self, expected: Token, message: &str) -> Result<(), String> {
        if self.peek() == expected {
            self.bump();
            Ok(())
        } else {
            Err(message.to_string())
        }
    }

    fn expect_end(&self) -> Result<(), String> {
        if matches!(self.peek(), Token::End) {
            Ok(())
        } else {
            Err(format!("unexpected token `{}`", self.peek().name()))
        }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.cursor).cloned().unwrap_or(Token::End)
    }

    fn bump(&mut self) -> Token {
        let token = self.peek();
        if !matches!(token, Token::End) {
            self.cursor += 1;
        }
        token
    }
}
