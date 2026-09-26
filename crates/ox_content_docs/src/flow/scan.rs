//! Classifies every source byte as code, comment, or literal.

use super::{CODE, COMMENT, LITERAL, is_expression_keyword, is_ident};

/// Marks every byte as code, comment, or literal (strings, template text, and
/// regular expressions).
pub(super) fn classify(src: &[u8]) -> Vec<u8> {
    let mut kind = Vec::new();
    kind.resize(src.len(), CODE);
    // One entry per open `{`; `true` marks a template `${` substitution.
    let mut braces: Vec<bool> = Vec::new();
    // Last significant code byte and the word it ends, for regex detection.
    let mut last: Option<usize> = None;
    let mut i = 0;

    while i < src.len() {
        let byte = src[i];
        match byte {
            b'/' if src.get(i + 1) == Some(&b'/') => {
                let end = src[i..].iter().position(|&b| b == b'\n').map_or(src.len(), |n| i + n);
                kind[i..end].fill(COMMENT);
                i = end;
                continue;
            }
            b'/' if src.get(i + 1) == Some(&b'*') => {
                let end = find(src, i + 2, b"*/").map_or(src.len(), |n| n + 2);
                kind[i..end].fill(COMMENT);
                i = end;
                continue;
            }
            b'\'' | b'"' => {
                let end = string_end(src, i);
                kind[i..end].fill(LITERAL);
                last = Some(end - 1);
                i = end;
                continue;
            }
            b'`' => {
                let (end, opened) = template_end(src, i + 1);
                kind[i..end].fill(LITERAL);
                if opened {
                    braces.push(true);
                }
                last = Some(end - 1);
                i = end;
                continue;
            }
            b'}' if braces.last() == Some(&true) => {
                braces.pop();
                let (end, opened) = template_end(src, i + 1);
                kind[i..end].fill(LITERAL);
                if opened {
                    braces.push(true);
                }
                last = Some(end - 1);
                i = end;
                continue;
            }
            b'/' if regex_allowed(src, &kind, last) => {
                let end = regex_end(src, i);
                kind[i..end].fill(LITERAL);
                last = Some(end - 1);
                i = end;
                continue;
            }
            b'{' => braces.push(false),
            b'}' => {
                braces.pop();
            }
            _ => {}
        }
        if !byte.is_ascii_whitespace() {
            last = Some(i);
        }
        i += 1;
    }
    kind
}

pub(super) fn find(src: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    src.get(from..)?.windows(needle.len()).position(|window| window == needle).map(|n| from + n)
}

pub(super) fn string_end(src: &[u8], open: usize) -> usize {
    let quote = src[open];
    let mut i = open + 1;
    while i < src.len() {
        match src[i] {
            b'\\' => i += 2,
            b'\n' => return i,
            byte if byte == quote => return i + 1,
            _ => i += 1,
        }
    }
    src.len()
}

/// Scans template text from `from`; returns the end and whether it stopped at
/// a `${` substitution (included in the literal).
pub(super) fn template_end(src: &[u8], from: usize) -> (usize, bool) {
    let mut i = from;
    while i < src.len() {
        match src[i] {
            b'\\' => i += 2,
            b'`' => return (i + 1, false),
            b'$' if src.get(i + 1) == Some(&b'{') => return (i + 2, true),
            _ => i += 1,
        }
    }
    (src.len(), false)
}

pub(super) fn regex_allowed(src: &[u8], kind: &[u8], last: Option<usize>) -> bool {
    let Some(last) = last else { return true };
    if kind[last] != CODE {
        return false;
    }
    match src[last] {
        b')' | b']' | b'}' => false,
        byte if is_ident(byte) => {
            let mut start = last + 1;
            while start > 0 && is_ident(src[start - 1]) {
                start -= 1;
            }
            is_expression_keyword(&src[start..=last])
        }
        _ => true,
    }
}

pub(super) fn regex_end(src: &[u8], open: usize) -> usize {
    let mut i = open + 1;
    let mut in_class = false;
    while i < src.len() {
        match src[i] {
            b'\\' => i += 1,
            b'\n' => return i,
            b'[' => in_class = true,
            b']' => in_class = false,
            b'/' if !in_class => {
                i += 1;
                while i < src.len() && is_ident(src[i]) {
                    i += 1;
                }
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    src.len()
}
