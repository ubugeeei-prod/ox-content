use crate::error::I18nResult;
use crate::mf2::ast::{
    Annotation, ComplexBody, ComplexMessage, Declaration, Expression, FunctionOption,
    InputDeclaration, LocalDeclaration, Matcher, Message, Operand, OptionValue, Pattern,
    PatternPart, Variant, VariantKey,
};
use crate::mf2::lexer::Token;

use super::Parser;

impl Parser {
    pub fn parse(&mut self) -> I18nResult<Message> {
        self.skip_newlines();

        if self.check(&Token::DotInput)
            || self.check(&Token::DotLocal)
            || self.check(&Token::DotMatch)
        {
            Ok(Message::Complex(self.parse_complex_message()?))
        } else {
            Ok(Message::Simple(self.parse_simple_pattern()?))
        }
    }

    fn parse_complex_message(&mut self) -> I18nResult<ComplexMessage> {
        let mut declarations = Vec::new();

        while self.check(&Token::DotInput) || self.check(&Token::DotLocal) {
            declarations.push(self.parse_declaration()?);
            self.skip_newlines();
        }

        let body = if self.check(&Token::DotMatch) {
            ComplexBody::Matcher(self.parse_matcher()?)
        } else if self.check(&Token::DoubleOpenBrace) {
            ComplexBody::QuotedPattern(self.parse_quoted_pattern()?)
        } else {
            return Err(self.error("expected .match or quoted pattern {{...}}"));
        };

        Ok(ComplexMessage { declarations, body })
    }

    fn parse_declaration(&mut self) -> I18nResult<Declaration> {
        if self.check(&Token::DotInput) {
            self.advance();
            Ok(Declaration::Input(self.parse_input_declaration()?))
        } else {
            self.advance();
            Ok(Declaration::Local(self.parse_local_declaration()?))
        }
    }

    fn parse_input_declaration(&mut self) -> I18nResult<InputDeclaration> {
        self.expect(&Token::OpenBrace)?;
        let variable = self.expect_variable()?;
        let annotation = self.try_parse_annotation()?;
        self.expect(&Token::CloseBrace)?;
        Ok(InputDeclaration { variable, annotation })
    }

    fn parse_local_declaration(&mut self) -> I18nResult<LocalDeclaration> {
        let variable = self.expect_variable()?;
        self.expect(&Token::Equals)?;
        self.expect(&Token::OpenBrace)?;
        let expression = self.parse_expression_body()?;
        self.expect(&Token::CloseBrace)?;
        Ok(LocalDeclaration { variable, expression })
    }

    fn parse_matcher(&mut self) -> I18nResult<Matcher> {
        self.advance();

        let mut selectors = Vec::new();
        while let Some(var) = self.try_consume_variable() {
            selectors.push(var);
        }
        if selectors.is_empty() {
            return Err(self.error("expected at least one selector variable after .match"));
        }

        self.skip_newlines();

        let mut variants = Vec::new();
        while !self.is_at_end() {
            if let Some(variant) = self.try_parse_variant(selectors.len())? {
                variants.push(variant);
                self.skip_newlines();
            } else {
                break;
            }
        }

        if variants.is_empty() {
            return Err(self.error("expected at least one variant"));
        }

        Ok(Matcher { selectors, variants })
    }

    fn try_parse_variant(&mut self, selector_count: usize) -> I18nResult<Option<Variant>> {
        let mut keys = Vec::new();

        for _ in 0..selector_count {
            if let Some(key) = self.try_parse_variant_key() {
                keys.push(key);
            } else if keys.is_empty() {
                return Ok(None);
            } else {
                return Err(self.error(&format!(
                    "expected {selector_count} variant keys, found {}",
                    keys.len()
                )));
            }
        }

        let pattern = self.parse_quoted_pattern()?;
        Ok(Some(Variant { keys, pattern }))
    }

    fn try_parse_variant_key(&mut self) -> Option<VariantKey> {
        if self.check(&Token::Star) {
            self.advance();
            Some(VariantKey::Wildcard)
        } else if let Some(name) = self.try_consume_name() {
            Some(VariantKey::Literal(name))
        } else {
            self.try_consume_number().map(VariantKey::Literal)
        }
    }

    fn parse_quoted_pattern(&mut self) -> I18nResult<Pattern> {
        self.expect(&Token::DoubleOpenBrace)?;
        let pattern = self.parse_pattern_until(true)?;
        self.expect(&Token::DoubleCloseBrace)?;
        Ok(pattern)
    }

    fn parse_simple_pattern(&mut self) -> I18nResult<Pattern> {
        self.parse_pattern_until(false)
    }

    fn parse_pattern_until(&mut self, in_quoted: bool) -> I18nResult<Pattern> {
        let mut parts = Vec::new();

        while !self.is_at_end() {
            if in_quoted && self.check(&Token::DoubleCloseBrace) {
                break;
            }

            match &self.tokens[self.pos].token {
                Token::Text(text) => {
                    parts.push(PatternPart::Text(text.clone()));
                    self.advance();
                }
                Token::OpenBrace => {
                    let expr = self.parse_expression()?;
                    parts.push(PatternPart::Expression(expr));
                }
                _ => break,
            }
        }

        Ok(Pattern { parts })
    }

    fn parse_expression(&mut self) -> I18nResult<Expression> {
        self.expect(&Token::OpenBrace)?;
        let expression = self.parse_expression_body()?;
        self.expect(&Token::CloseBrace)?;
        Ok(expression)
    }

    fn parse_expression_body(&mut self) -> I18nResult<Expression> {
        let operand = self.try_parse_operand();
        let annotation = self.try_parse_annotation()?;

        if operand.is_none() && annotation.is_none() {
            return Err(self.error("expected variable, literal, or function in expression"));
        }

        Ok(Expression { operand, annotation })
    }

    fn try_parse_operand(&mut self) -> Option<Operand> {
        if let Some(var) = self.try_consume_variable() {
            Some(Operand::Variable(var))
        } else if let Some(num) = self.try_consume_number() {
            Some(Operand::Literal(num))
        } else {
            self.try_consume_quoted_literal().map(Operand::Literal)
        }
    }

    fn try_parse_annotation(&mut self) -> I18nResult<Option<Annotation>> {
        if let Some(function) = self.try_consume_function() {
            let mut options = Vec::new();
            while let Some(opt) = self.try_parse_function_option()? {
                options.push(opt);
            }
            Ok(Some(Annotation { function, options }))
        } else {
            Ok(None)
        }
    }

    fn try_parse_function_option(&mut self) -> I18nResult<Option<FunctionOption>> {
        if self.pos + 1 < self.tokens.len() {
            if let Token::Name(ref name) = self.tokens[self.pos].token {
                if self.tokens[self.pos + 1].token == Token::Equals {
                    let name = name.clone();
                    self.advance();
                    self.advance();
                    let value = self.parse_option_value()?;
                    return Ok(Some(FunctionOption { name, value }));
                }
            }
        }
        Ok(None)
    }

    fn parse_option_value(&mut self) -> I18nResult<OptionValue> {
        if let Some(var) = self.try_consume_variable() {
            Ok(OptionValue::Variable(var))
        } else if let Some(num) = self.try_consume_number() {
            Ok(OptionValue::Literal(num))
        } else if let Some(name) = self.try_consume_name() {
            Ok(OptionValue::Literal(name))
        } else if let Some(lit) = self.try_consume_quoted_literal() {
            Ok(OptionValue::Literal(lit))
        } else {
            Err(self.error("expected option value"))
        }
    }
}
