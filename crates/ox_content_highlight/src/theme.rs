//! Mapping from tree-sitter capture names to the renderer's CSS variables.
//!
//! The emitted colors are `--octc-shiki-*` custom properties with a baked-in
//! GitHub Dark fallback, the same contract the previous highlighter used. That
//! is what lets one build serve both light and dark: the HTML is generated
//! once and the properties resolve per color scheme. Keeping the variable
//! names means every `@ox-content/theme-color-*` package keeps working.

/// Capture names requested from every grammar, in priority order.
///
/// Tree-sitter resolves a capture to the last matching name, so more specific
/// names must come after the general ones they refine.
pub const CAPTURE_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "function.method",
    "keyword",
    "label",
    "module",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "text.emphasis",
    "text.literal",
    "text.reference",
    "text.strong",
    "text.title",
    "text.uri",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

/// One CSS variable: its `--octc-shiki-` suffix and the fallback color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub name: &'static str,
    pub fallback: &'static str,
}

pub const FOREGROUND: Token = Token { name: "foreground", fallback: "#e6edf3" };
pub const BACKGROUND: Token = Token { name: "background", fallback: "#0d1117" };

const COMMENT: Token = Token { name: "token-comment", fallback: "#8b949e" };
const CONSTANT: Token = Token { name: "token-constant", fallback: "#79c0ff" };
const FUNCTION: Token = Token { name: "token-function", fallback: "#d2a8ff" };
const KEYWORD: Token = Token { name: "token-keyword", fallback: "#ff7b72" };
const PARAMETER: Token = Token { name: "token-parameter", fallback: "#ffa657" };
const PUNCTUATION: Token = Token { name: "token-punctuation", fallback: "#c9d1d9" };
const STRING: Token = Token { name: "token-string", fallback: "#a5d6ff" };
const LINK: Token = Token { name: "token-link", fallback: "#a5d6ff" };
const STRING_EXPRESSION: Token = Token { name: "token-string-expression", fallback: "#a5d6ff" };

/// The variable a capture paints with, or `None` to leave it at the
/// foreground color.
pub fn token_for(capture: &str) -> Option<Token> {
    Some(match capture {
        "comment" => COMMENT,
        "constant" | "constant.builtin" | "number" | "type" | "type.builtin" => CONSTANT,
        "constructor" | "function" | "function.builtin" | "function.method" => FUNCTION,
        "keyword" | "operator" | "label" => KEYWORD,
        "variable.parameter" => PARAMETER,
        "punctuation" | "punctuation.bracket" | "punctuation.delimiter" | "punctuation.special" => {
            PUNCTUATION
        }
        "string" | "string.escape" | "string.special" => STRING,
        "embedded" => STRING_EXPRESSION,
        "attribute" | "property" | "tag" | "module" => CONSTANT,
        // Markdown's grammars speak nvim-treesitter's capture vocabulary
        // rather than the names every other grammar here uses.
        "text.title" => FUNCTION,
        "text.literal" => STRING,
        "text.uri" | "text.reference" => LINK,
        "text.emphasis" | "text.strong" => CONSTANT,
        _ => return None,
    })
}

/// Writes `color:var(--octc-shiki-NAME, FALLBACK)` into `out`.
pub fn push_color(out: &mut String, token: Token) {
    out.push_str("color:var(--octc-shiki-");
    out.push_str(token.name);
    out.push_str(", ");
    out.push_str(token.fallback);
    out.push(')');
}
