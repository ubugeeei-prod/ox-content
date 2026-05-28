use ox_content_ast::{Html, Span};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_html(
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> Option<(Html<'a>, usize)> {
        let bytes = content.as_bytes();

        if content[pos..].starts_with("<!--") {
            let end = content[pos + 4..].find("-->").map(|found| pos + 4 + found + 3)?;
            return Some((Self::inline_html_node(content, pos, end, offset), end));
        }

        let closing = bytes.get(pos + 1) == Some(&b'/');
        let tag_start = if closing { pos + 2 } else { pos + 1 };
        if !bytes.get(tag_start).is_some_and(u8::is_ascii_alphabetic) {
            return None;
        }

        let tag_end = Self::inline_tag_name_end(bytes, tag_start);
        let mut cursor = tag_end;

        if closing {
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            return (bytes.get(cursor) == Some(&b'>')).then(|| {
                let end = cursor + 1;
                (Self::inline_html_node(content, pos, end, offset), end)
            });
        }

        if !matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'/' | b'>')) {
            return None;
        }

        Self::find_inline_html_end(bytes, cursor)
            .map(|end| (Self::inline_html_node(content, pos, end, offset), end))
    }

    fn inline_tag_name_end(bytes: &[u8], start: usize) -> usize {
        let mut end = start + 1;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
            end += 1;
        }
        end
    }

    fn find_inline_html_end(bytes: &[u8], mut cursor: usize) -> Option<usize> {
        let mut quote = None;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if let Some(quote_byte) = quote {
                if byte == quote_byte {
                    quote = None;
                }
            } else if matches!(byte, b'"' | b'\'') {
                quote = Some(byte);
            } else if byte == b'>' {
                return Some(cursor + 1);
            } else if byte == b'\n' {
                return None;
            }
            cursor += 1;
        }
        None
    }

    fn inline_html_node(content: &'a str, start: usize, end: usize, offset: usize) -> Html<'a> {
        Html {
            value: &content[start..end],
            span: Span::new((offset + start) as u32, (offset + end) as u32),
        }
    }
}
