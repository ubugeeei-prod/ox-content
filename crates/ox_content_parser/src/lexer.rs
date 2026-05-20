//! Lexer for Markdown tokenization.
//!
//! This module provides token definitions for Markdown parsing.
//! Currently used for testing and will be integrated with the parser.

#![allow(dead_code)]

use logos::Logos;

/// Markdown tokens.
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    /// ATX heading prefix (# to ######).
    #[regex(r"#{1,6}[ \t]+", |lex| lex.slice().trim_end().len())]
    HeadingPrefix(usize),

    /// Thematic break (---, ***, ___).
    #[regex(r"(\*[ \t]*){3,}|(-[ \t]*){3,}|(_[ \t]*){3,}")]
    ThematicBreak,

    /// Block quote prefix.
    #[token(">")]
    BlockQuotePrefix,

    /// Unordered list marker.
    #[regex(r"[-*+][ \t]+")]
    UnorderedListMarker,

    /// Ordered list marker.
    #[regex(r"[0-9]+\.[ \t]+", |lex| {
        let s = lex.slice();
        s[..s.len()-2].parse::<u32>().ok()
    })]
    OrderedListMarker(Option<u32>),

    /// Fenced code block delimiter.
    #[regex(r"```[^\n]*|~~~[^\n]*", allow_greedy = true)]
    FencedCodeDelimiter,

    /// Emphasis marker (*).
    #[token("*")]
    Asterisk,

    /// Emphasis marker (_).
    #[token("_")]
    Underscore,

    /// Strong emphasis marker (**).
    #[token("**")]
    DoubleAsterisk,

    /// Strong emphasis marker (__).
    #[token("__")]
    DoubleUnderscore,

    /// Strikethrough marker (~~).
    #[token("~~")]
    Strikethrough,

    /// Inline code delimiter.
    #[token("`")]
    Backtick,

    /// Double inline code delimiter.
    #[token("``")]
    DoubleBacktick,

    /// Link start.
    #[token("[")]
    LeftBracket,

    /// Link middle.
    #[token("](")]
    LinkMiddle,

    /// Right bracket.
    #[token("]")]
    RightBracket,

    /// Left parenthesis.
    #[token("(")]
    LeftParen,

    /// Right parenthesis.
    #[token(")")]
    RightParen,

    /// Image start.
    #[token("![")]
    ImageStart,

    /// Newline.
    #[token("\n")]
    Newline,

    /// Hard line break (two spaces + newline or backslash + newline).
    #[regex(r"  \n|\\n")]
    HardBreak,

    /// Escape sequence.
    #[regex(r"\\[\\`*_\{\}\[\]()#+\-.!~]")]
    Escape,

    /// HTML tag.
    #[regex(r"<[^>]+>")]
    HtmlTag,

    /// Autolink.
    #[regex(r"<[a-zA-Z][a-zA-Z0-9+.-]*:[^\s>]+>")]
    Autolink,

    /// Pipe for tables.
    #[token("|")]
    Pipe,

    /// Whitespace (space or tab).
    #[regex(r"[ \t]+")]
    Whitespace,

    /// Plain text (single character fallback).
    #[regex(r"[^\n\[\]()!*_`~\\#>|\- \t]+")]
    Text,
}

/// Lexer state for tracking context.
#[derive(Debug, Clone, Default)]
pub struct LexerState {
    /// Current line number (1-indexed).
    pub line: u32,
    /// Current column number (1-indexed).
    pub column: u32,
    /// Whether we're at the start of a line.
    pub at_line_start: bool,
    /// Current indentation level.
    pub indent: u32,
}

impl LexerState {
    /// Creates a new lexer state.
    #[must_use]
    pub fn new() -> Self {
        Self { line: 1, column: 1, at_line_start: true, indent: 0 }
    }

    /// Updates state after processing a token.
    pub fn advance(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                self.indent = 0;
            } else {
                if self.at_line_start {
                    if ch == ' ' {
                        self.indent += 1;
                    } else if ch == '\t' {
                        self.indent += 4;
                    } else {
                        self.at_line_start = false;
                    }
                }
                self.column += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_prefix() {
        let mut lexer = Token::lexer("# ");
        assert_eq!(lexer.next(), Some(Ok(Token::HeadingPrefix(1))));
    }

    #[test]
    fn test_thematic_break() {
        let mut lexer = Token::lexer("---");
        assert_eq!(lexer.next(), Some(Ok(Token::ThematicBreak)));
    }

    #[test]
    fn test_ordered_list() {
        let mut lexer = Token::lexer("1. ");
        assert_eq!(lexer.next(), Some(Ok(Token::OrderedListMarker(Some(1)))));
    }
}
