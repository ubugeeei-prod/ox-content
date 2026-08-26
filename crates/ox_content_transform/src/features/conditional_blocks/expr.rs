use rustc_hash::FxHashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::BuildHasher;

mod eval;
mod lexer;
mod parser;

pub(super) struct EvalContext<'a, S: BuildHasher> {
    pub(super) config: &'a FxHashMap<String, Value>,
    pub(super) frontmatter: &'a HashMap<String, Value, S>,
}

pub(super) fn evaluate<S: BuildHasher>(
    expression: &str,
    context: &EvalContext<'_, S>,
) -> Result<bool, String> {
    eval::eval_bool(&parser::parse(lexer::lex(expression)?)?, context)
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Expr {
    Literal(Value),
    Variable(String),
    Array(Vec<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}
