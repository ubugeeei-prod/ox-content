mod scan;
#[cfg(test)]
mod tests;
mod tokens;

/// MF2 tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `.input` keyword
    DotInput,
    /// `.local` keyword
    DotLocal,
    /// `.match` keyword
    DotMatch,
    /// Variable reference `$name`
    Variable(String),
    /// Function name `:name`
    Function(String),
    /// Identifier / name
    Name(String),
    /// Numeric literal
    Number(String),
    /// Quoted literal `|...|`
    QuotedLiteral(String),
    /// Literal text (in pattern context)
    Text(String),
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `{{`
    DoubleOpenBrace,
    /// `}}`
    DoubleCloseBrace,
    /// `=`
    Equals,
    /// `*` wildcard
    Star,
    /// Newline
    Newline,
}

/// A positioned token with its byte offset span.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

/// Context-aware MF2 tokenizer.
///
/// MF2 has two modes:
/// - **Pattern mode**: text is literal until `{` or `}}`.
/// - **Expression mode** (inside `{ }`): variables, functions, options.
/// - **Declaration mode**: `.input`, `.local`, `.match` and their arguments.
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, usize> {
    let mut scanner = Scanner::new(source);
    scanner.scan_message()?;
    Ok(scanner.tokens)
}

struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    tokens: Vec<SpannedToken>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, bytes: source.as_bytes(), pos: 0, tokens: Vec::new() }
    }
}
