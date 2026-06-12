mod grammar;
#[cfg(test)]
mod tests;
mod tokens;

use crate::mf2::lexer::SpannedToken;

/// Recursive descent parser for MF2 messages.
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }
}
