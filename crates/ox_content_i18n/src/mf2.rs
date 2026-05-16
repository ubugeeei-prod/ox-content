pub mod ast;
pub mod lexer;
pub mod parser;
pub mod validator;

use crate::error::{I18nError, I18nResult};
use ast::Message;

/// Parses an MF2 message string into an AST.
pub fn parse(source: &str) -> I18nResult<Message> {
    let tokens = lexer::tokenize(source).map_err(|offset| I18nError::Mf2Parse {
        offset,
        message: "unexpected character".to_string(),
    })?;
    let mut p = parser::Parser::new(tokens);
    p.parse()
}

/// Parses and validates an MF2 message, returning the AST and any validation errors.
pub fn parse_and_validate(source: &str) -> I18nResult<(Message, Vec<I18nError>)> {
    let message = parse(source)?;
    let errors = validator::validate(&message);
    Ok((message, errors))
}
