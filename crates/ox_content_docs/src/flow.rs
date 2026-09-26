//! Flow type annotation support.
//!
//! oxc does not parse [Flow](https://flow.org). Flow's annotation syntax is
//! close enough to TypeScript that rewriting the Flow-only constructs lets the
//! TypeScript parser read the file. Every rewrite preserves byte length (Flow
//! syntax is blanked with spaces or swapped for an equally long TypeScript
//! spelling), so all spans of the parsed program line up with the original
//! source. Extraction then slices signatures, types, and JSDoc from the
//! untouched Flow text, and the generated docs keep Flow's own notation.
//!
//! Rewrites are only ever applied to ASCII bytes, so the output stays valid
//! UTF-8. Where a heuristic could misfire in value position, the rewrite is
//! chosen so the result still parses (for example, dropping a unary `+`).

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

mod cursor;
mod declarations;
mod parens;
mod regions;
mod scan;
#[cfg(test)]
mod tests;
mod types;

use self::scan::classify;

/// Parses `source`, reading it as Flow when the file is a Flow file.
///
/// Non-Flow files are parsed unchanged with `source_type`.
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    path: &str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    if !is_flow_source(path, source) {
        return Parser::new(allocator, source, source_type).parse();
    }

    let mut text = to_typescript(source);
    // Flow files may contain JSX, but TSX rejects `<T>(x) => x` arrow generics,
    // so prefer plain TypeScript.
    let mut source_type = SourceType::ts();
    let mut ret = Parser::new(allocator, allocator.alloc_str(&text), source_type).parse();
    if !ret.diagnostics.is_empty() {
        let jsx = Parser::new(allocator, allocator.alloc_str(&text), SourceType::tsx()).parse();
        if jsx.diagnostics.len() < ret.diagnostics.len() {
            source_type = SourceType::tsx();
            ret = jsx;
        }
    }

    // Syntax the rewrite cannot express (`hook`, `match`, ...) would otherwise
    // lose the whole file. Blank the offending top-level statement and retry,
    // so the rest of the module is still documented.
    for _ in 0..MAX_RECOVERED_STATEMENTS {
        let Some(offset) = ret.diagnostics.first().and_then(|diagnostic| {
            diagnostic.labels.as_ref().first().map(|label| label.offset() as usize)
        }) else {
            break;
        };
        if !blank_top_level_statement(&mut text, offset) {
            break;
        }
        ret = Parser::new(allocator, allocator.alloc_str(&text), source_type).parse();
    }
    ret
}

const MAX_RECOVERED_STATEMENTS: usize = 16;

/// Blanks the top-level statement around `offset`, keeping line breaks.
/// Statements are found by indentation: a top-level statement starts at column
/// zero. Returns `false` when nothing was left to blank.
fn blank_top_level_statement(text: &mut String, offset: usize) -> bool {
    let bytes = text.as_bytes();
    let offset = offset.min(bytes.len());
    let starts_statement = |line: usize| {
        bytes.get(line).is_some_and(|&byte| is_ident_start(byte) || matches!(byte, b'@' | b'/'))
    };
    let line_start =
        |index: usize| bytes[..index].iter().rposition(|&b| b == b'\n').map_or(0, |n| n + 1);

    let mut start = line_start(offset);
    while start > 0 && !starts_statement(start) {
        start = line_start(start - 1);
    }
    let mut end = bytes.len();
    let mut cursor = offset;
    while let Some(newline) = bytes[cursor..].iter().position(|&b| b == b'\n') {
        cursor += newline + 1;
        if starts_statement(cursor) {
            end = cursor;
            break;
        }
    }
    if bytes[start..end].iter().all(u8::is_ascii_whitespace) {
        return false;
    }
    let blanked: String =
        text[start..end].bytes().map(|b| if b == b'\n' { '\n' } else { ' ' }).collect();
    text.replace_range(start..end, &blanked);
    true
}

/// Whether the file should be read as Flow: `*.js.flow` declaration files, or
/// JavaScript files carrying an `@flow` / `@noflow` pragma in their leading
/// comments.
pub fn is_flow_source(path: &str, source: &str) -> bool {
    match Path::new(path).extension().and_then(|extension| extension.to_str()) {
        Some("flow") => true,
        Some("js" | "jsx" | "mjs" | "cjs") => has_flow_pragma(source),
        _ => false,
    }
}

fn has_flow_pragma(source: &str) -> bool {
    let mut rest = match source.strip_prefix("#!") {
        Some(after) => after.find('\n').map_or("", |newline| &after[newline..]),
        None => source,
    };
    loop {
        rest = rest.trim_start();
        if let Some(body) = rest.strip_prefix("//") {
            let end = body.find('\n').unwrap_or(body.len());
            if is_flow_pragma(&body[..end]) {
                return true;
            }
            rest = &body[end..];
        } else if let Some(body) = rest.strip_prefix("/*") {
            let Some(end) = body.find("*/") else { return false };
            if is_flow_pragma(&body[..end]) {
                return true;
            }
            rest = &body[end + 2..];
        } else {
            return false;
        }
    }
}

fn is_flow_pragma(comment: &str) -> bool {
    comment
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '@'))
        .any(|word| word == "@flow" || word == "@noflow")
}

/// Rewrites Flow-only syntax into equally long TypeScript-compatible syntax.
pub fn to_typescript(source: &str) -> String {
    let mut rewriter = Rewriter::new(source.as_bytes());
    rewriter.run();
    String::from_utf8(rewriter.out).unwrap_or_else(|_| source.to_string())
}

const CODE: u8 = 0;
const COMMENT: u8 = 1;
const LITERAL: u8 = 2;

/// Keywords after which `(` starts an expression and `/` starts a regex.
const EXPRESSION_KEYWORDS: &[&[u8]] = &[
    b"return",
    b"typeof",
    b"await",
    b"yield",
    b"case",
    b"in",
    b"of",
    b"void",
    b"delete",
    b"throw",
    b"else",
    b"do",
    b"instanceof",
    b"new",
    b"default",
];

/// Type keywords that cannot name a TypeScript parameter.
const RESERVED_TYPE_WORDS: &[&[u8]] =
    &[b"void", b"null", b"true", b"false", b"typeof", b"this", b"new", b"function", b"class"];

struct Rewriter<'s> {
    src: &'s [u8],
    kind: Vec<u8>,
    out: Vec<u8>,
    /// Bytes in type position (see [`regions`]).
    types: Vec<bool>,
}

impl<'s> Rewriter<'s> {
    fn new(src: &'s [u8]) -> Self {
        let mut types = Vec::new();
        types.resize(src.len(), false);
        let mut rewriter = Self { src, kind: classify(src), out: src.to_vec(), types };
        rewriter.mark_type_regions();
        rewriter
    }

    fn run(&mut self) {
        let mut i = 0;
        while i < self.src.len() {
            // Skip non-code and bytes an earlier rewrite already replaced.
            if self.kind[i] != CODE || self.out[i] != self.src[i] {
                i += 1;
                continue;
            }
            match self.src[i] {
                b'{' if self.code_byte(i + 1) == Some(b'|') => self.blank(i + 1, i + 2),
                b'|' if self.code_byte(i + 1) == Some(b'}') => self.blank(i, i + 1),
                b'%' => self.predicate(i),
                // `@@iterator(): Iterator<T>;` members of library definitions.
                b'@' if self.src[i..].starts_with(b"@@") && self.types[i] => {
                    let end = self.depth_zero_in(i, self.src.len(), b";\n}", false);
                    self.blank(i, end.unwrap_or(self.src.len()));
                }
                // `T?.['key']` optional indexed access types.
                b'?' if self.types[i] && self.src[i..].starts_with(b"?.[") => self.blank(i, i + 2),
                b'?' => self.maybe_type(i),
                b'+' | b'-' => self.variance(i),
                b'*' => self.existential(i),
                // `Foo<>` — Flow's explicit "use the defaults".
                b'<' if self.code_byte(i + 1) == Some(b'>')
                    && i > 0
                    && is_ident(self.src[i - 1]) =>
                {
                    self.blank(i, i + 2);
                }
                b'<' => self.type_parameter_bounds(i),
                b'[' if self.types[i] => self.mapped_type_separator(i),
                // `T => U` — Flow's unparenthesized single-parameter function
                // type becomes the equally long union `T  | U`.
                b'=' if self.types[i]
                    && self.code_byte(i + 1) == Some(b'>')
                    && self.prev_sig(i).is_some_and(|prev| self.sig_byte(prev) != b')')
                    && !self.next_sig(i + 2).is_some_and(|next| {
                        self.src[next] == b'{' && self.code_byte(next + 1) != Some(b'|')
                    }) =>
                {
                    self.out[i] = b' ';
                    self.out[i + 1] = b'|';
                }
                b'(' => self.parenthesized(i),
                b'.' if self.src[i..].starts_with(b"...") => {
                    self.object_type_spread(i);
                    i += 3;
                    continue;
                }
                byte if is_ident_start(byte) && !self.continues_word(i) => {
                    let end = self.word_end(i);
                    self.keyword(i, end);
                    i = end;
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') || byte >= 0x80
}

fn is_ident(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}

fn is_expression_keyword(word: &[u8]) -> bool {
    EXPRESSION_KEYWORDS.contains(&word)
}
