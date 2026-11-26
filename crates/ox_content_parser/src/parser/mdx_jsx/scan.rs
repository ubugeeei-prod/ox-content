//! JSX tag scanning: fragments, member names, named attrs, and spreads.

use ox_content_allocator::Vec;
use ox_content_ast::{
    MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxAttributeValueExpression,
    MdxJsxExpressionAttribute, Span,
};

use super::super::line_scan::{is_line_ending_byte, line_terminator_end};
use super::lex::Lex;

/// Opening tag accepted by this slice.
pub(super) struct JsxOpen<'a> {
    pub name: Option<&'a str>,
    pub self_closing: bool,
    pub end: usize,
}

#[inline]
pub(super) fn looks_like_jsx_open(bytes: &[u8], at: usize) -> bool {
    bytes.get(at) == Some(&b'<')
        && match bytes.get(at + 1) {
            Some(b'>') => true,
            Some(byte) => byte.is_ascii_uppercase(),
            None => false,
        }
}

pub(super) fn only_ws_until_eol(bytes: &[u8], mut cursor: usize) -> bool {
    while cursor < bytes.len() {
        match bytes[cursor] {
            b' ' | b'\t' => cursor += 1,
            byte if is_line_ending_byte(byte) => return true,
            _ => return false,
        }
    }
    true
}

pub(super) fn after_trailing_line_ws(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
        cursor += 1;
    }
    if cursor < bytes.len() && is_line_ending_byte(bytes[cursor]) {
        line_terminator_end(bytes, cursor)
    } else {
        cursor
    }
}

/// `lex` must scan `source`.
pub(super) fn scan_jsx_open<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<JsxOpen<'a>> {
    let bytes = source.as_bytes();
    if !looks_like_jsx_open(bytes, start) {
        return None;
    }
    if bytes.get(start + 1) == Some(&b'>') {
        return Some(JsxOpen { name: None, self_closing: false, end: start + 2 });
    }
    let name_start = start + 1;
    let name_end = scan_jsx_name(bytes, name_start)?;
    let name = &source[name_start..name_end];
    let (self_closing, end) = scan_attributes(lex, source, name_end, offset, attributes)?;
    Some(JsxOpen { name: Some(name), self_closing, end })
}

fn scan_jsx_name(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes.get(start)?.is_ascii_uppercase() {
        return None;
    }
    scan_member_name(bytes, start)
}

fn scan_attributes<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    mut cursor: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<(bool, usize)> {
    let bytes = source.as_bytes();
    loop {
        let had_ws = skip_ws(bytes, &mut cursor);
        match bytes.get(cursor)? {
            b'/' if bytes.get(cursor + 1) == Some(&b'>') => return Some((true, cursor + 2)),
            b'>' => return Some((false, cursor + 1)),
            b'{' if had_ws => {
                cursor = push_expression_attribute(lex, source, cursor, offset, attributes)?;
            }
            _ if !had_ws => return None,
            _ => cursor = push_named_attribute(lex, source, cursor, offset, attributes)?,
        }
    }
}

fn push_expression_attribute<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<usize> {
    let end = lex.skip_braces(start)?;
    attributes.push(MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
        value: &source[start + 1..end - 1],
        span: Span::new((offset + start) as u32, (offset + end) as u32),
    }));
    Some(end)
}

fn push_named_attribute<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    start: usize,
    offset: usize,
    attributes: &mut Vec<'a, MdxJsxAttributeEntry<'a>>,
) -> Option<usize> {
    let bytes = source.as_bytes();
    let name_end = scan_attr_name(bytes, start)?;
    let name = &source[start..name_end];
    let mut cursor = name_end;
    skip_ws(bytes, &mut cursor);
    if bytes.get(cursor) != Some(&b'=') {
        attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
            name,
            value: None,
            span: Span::new((offset + start) as u32, (offset + name_end) as u32),
        }));
        return Some(name_end);
    }
    cursor += 1;
    skip_ws(bytes, &mut cursor);
    let (value, value_end) = scan_attr_value(lex, source, cursor, offset)?;
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name,
        value: Some(value),
        span: Span::new((offset + start) as u32, (offset + value_end) as u32),
    }));
    Some(value_end)
}

fn scan_attr_value<'a>(
    lex: &mut Lex<'_>,
    source: &'a str,
    start: usize,
    offset: usize,
) -> Option<(MdxJsxAttributeValue<'a>, usize)> {
    match *source.as_bytes().get(start)? {
        b'"' | b'\'' => {
            let end = lex.skip_quoted(start)?;
            Some((MdxJsxAttributeValue::Literal(&source[start + 1..end - 1]), end))
        }
        b'{' => {
            let end = lex.skip_braces(start)?;
            Some((
                MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
                    value: &source[start + 1..end - 1],
                    span: Span::new((offset + start) as u32, (offset + end) as u32),
                }),
                end,
            ))
        }
        _ => None,
    }
}

pub(super) fn scan_member_name(bytes: &[u8], start: usize) -> Option<usize> {
    let mut end = scan_ident(bytes, start)?;
    while bytes.get(end) == Some(&b'.') {
        end = scan_ident(bytes, end + 1)?;
    }
    Some(end)
}

fn scan_ident(bytes: &[u8], start: usize) -> Option<usize> {
    let first = *bytes.get(start)?;
    if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b'$')) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'$'))
    {
        end += 1;
    }
    Some(end)
}

pub(super) fn scan_attr_name(bytes: &[u8], start: usize) -> Option<usize> {
    if !bytes.get(start).is_some_and(|byte| is_attr_name_start(*byte)) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric()
            || matches!(bytes[end], b'_' | b'$' | b'-' | b':' | b'.'))
    {
        end += 1;
    }
    Some(end)
}

#[inline]
pub(super) fn is_attr_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

pub(super) fn skip_ws(bytes: &[u8], cursor: &mut usize) -> bool {
    let start = *cursor;
    while *cursor < bytes.len() && matches!(bytes[*cursor], b' ' | b'\t' | b'\n' | b'\r') {
        *cursor += 1;
    }
    *cursor > start
}
