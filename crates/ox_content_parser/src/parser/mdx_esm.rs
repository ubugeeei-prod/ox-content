//! Module-level MDX `import` / `export` (`MdxjsEsm`).
//!
//! Recognition is keyword-based (`import` / `export` plus a boundary). The
//! statement is closed by a naive brace / paren / bracket / string / comment
//! scan — not a JavaScript parser. Source is stored and never evaluated.
//!
//! Limits: regex literals, ASI edge cases, and `${}` inside templates are not
//! fully handled. Import resolution and hydration are left to framework plugins.

use ox_content_ast::{MdxjsEsm, Node, Span};

use super::Parser;

#[inline]
pub(super) fn looks_like_esm(bytes: &[u8], at: usize) -> bool {
    keyword_then_boundary(bytes, at, b"import") || keyword_then_boundary(bytes, at, b"export")
}

fn keyword_then_boundary(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    let rest = bytes.get(at..).unwrap_or(&[]);
    if !rest.starts_with(keyword) {
        return false;
    }
    match rest.get(keyword.len()) {
        None => true,
        Some(b) => is_keyword_boundary(*b),
    }
}

fn is_keyword_boundary(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'*' | b'\'' | b'"' | b'`' | b'/')
}

#[derive(Clone, Copy)]
enum Scan {
    Normal,
    Squote,
    Dquote,
    Template,
    LineComment,
    BlockComment,
}

/// Returns the exclusive end of the ESM construct, including a trailing newline.
fn scan_esm_statement(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut brace = 0u32;
    let mut paren = 0u32;
    let mut bracket = 0u32;
    let mut state = Scan::Normal;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            Scan::Normal => match b {
                b'{' => {
                    brace += 1;
                    i += 1;
                }
                b'}' => {
                    brace = brace.saturating_sub(1);
                    i += 1;
                }
                b'(' => {
                    paren += 1;
                    i += 1;
                }
                b')' => {
                    paren = paren.saturating_sub(1);
                    i += 1;
                }
                b'[' => {
                    bracket += 1;
                    i += 1;
                }
                b']' => {
                    bracket = bracket.saturating_sub(1);
                    i += 1;
                }
                b'\'' => {
                    state = Scan::Squote;
                    i += 1;
                }
                b'"' => {
                    state = Scan::Dquote;
                    i += 1;
                }
                b'`' => {
                    state = Scan::Template;
                    i += 1;
                }
                b'/' => match bytes.get(i + 1) {
                    Some(b'/') => {
                        state = Scan::LineComment;
                        i += 2;
                    }
                    Some(b'*') => {
                        state = Scan::BlockComment;
                        i += 2;
                    }
                    _ => i += 1,
                },
                b';' if brace == 0 && paren == 0 && bracket == 0 => {
                    return after_trailing_line_ws(bytes, i + 1);
                }
                b'\n' if brace == 0 && paren == 0 && bracket == 0 => return i + 1,
                _ => i += 1,
            },
            Scan::Squote => {
                if b == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if b == b'\'' {
                    state = Scan::Normal;
                    i += 1;
                } else {
                    i += 1;
                }
            }
            Scan::Dquote => {
                if b == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if b == b'"' {
                    state = Scan::Normal;
                    i += 1;
                } else {
                    i += 1;
                }
            }
            Scan::Template => {
                if b == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if b == b'`' {
                    state = Scan::Normal;
                    i += 1;
                } else {
                    i += 1;
                }
            }
            Scan::LineComment => {
                if b == b'\n' {
                    if brace == 0 && paren == 0 && bracket == 0 {
                        return i + 1;
                    }
                    state = Scan::Normal;
                }
                i += 1;
            }
            Scan::BlockComment => {
                if b == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    state = Scan::Normal;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
    }
    bytes.len()
}

fn after_trailing_line_ws(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t' | b'\r') {
        cursor += 1;
    }
    if cursor < bytes.len() && bytes[cursor] == b'\n' { cursor + 1 } else { cursor }
}

impl<'a> Parser<'a> {
    /// Parses a module-level `import` / `export` starting at the current line.
    ///
    /// On mismatch the cursor is left unchanged so paragraph dispatch can run.
    /// Unbalanced braces and hostile strings store source and do not panic.
    pub(super) fn try_parse_mdxjs_esm(
        &mut self,
        start: usize,
        trimmed_start: usize,
    ) -> Option<Node<'a>> {
        if !self.options.mdx || !looks_like_esm(self.source.as_bytes(), trimmed_start) {
            return None;
        }
        let end = scan_esm_statement(self.source, trimmed_start);
        self.position = end;
        Some(Node::MdxjsEsm(MdxjsEsm {
            value: self.source[trimmed_start..end].trim_end(),
            span: Span::new(start as u32, end as u32),
        }))
    }
}
