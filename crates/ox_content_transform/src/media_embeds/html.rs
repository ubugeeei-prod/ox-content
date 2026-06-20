use crate::html_scan::find_ci;

pub(super) struct ComponentElement<'a> {
    pub(super) span: (usize, usize),
    attrs: Vec<(&'a str, &'a str)>,
    pub(super) body: &'a str,
}

pub(super) fn find_component<'a>(
    html: &'a str,
    from: usize,
    open: &str,
    name: &str,
) -> Option<ComponentElement<'a>> {
    let bytes = html.as_bytes();
    let mut search = from;
    loop {
        let tag_start = find_ci(html, search, open)?;
        let after_name = tag_start + open.len();
        let boundary = bytes.get(after_name).copied();
        if !matches!(boundary, Some(b) if b == b'>' || b == b'/' || b.is_ascii_whitespace()) {
            search = after_name;
            continue;
        }

        let start_tag = scan_start_tag(html, tag_start)?;
        let attr_end = if start_tag.self_closing { start_tag.inner_end } else { start_tag.tag_end };
        let attrs = parse_attrs(&html[after_name..attr_end]);
        if start_tag.self_closing {
            return Some(ComponentElement { span: (tag_start, start_tag.end), attrs, body: "" });
        }
        let close = format!("</{name}>");
        let close_start = find_ci(html, start_tag.end, &close).unwrap_or(start_tag.end);
        let span_end =
            if close_start == start_tag.end { start_tag.end } else { close_start + close.len() };
        return Some(ComponentElement {
            span: (tag_start, span_end),
            attrs,
            body: &html[start_tag.end..close_start],
        });
    }
}

pub(super) fn attr<'a>(element: &'a ComponentElement<'_>, name: &str) -> Option<&'a str> {
    element
        .attrs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
        .filter(|value| !value.is_empty())
}

struct StartTag {
    inner_end: usize,
    tag_end: usize,
    end: usize,
    self_closing: bool,
}

fn scan_start_tag(html: &str, start: usize) -> Option<StartTag> {
    let bytes = html.as_bytes();
    let mut cursor = start;
    let mut quote = None;
    while cursor < bytes.len() {
        match quote {
            Some(q) if bytes[cursor] == q => quote = None,
            Some(_) => {}
            None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => quote = Some(bytes[cursor]),
            None if bytes[cursor] == b'>' => {
                let self_closing = cursor > start && bytes[cursor - 1] == b'/';
                return Some(StartTag {
                    inner_end: if self_closing { cursor - 1 } else { cursor },
                    tag_end: cursor,
                    end: cursor + 1,
                    self_closing,
                });
            }
            None => {}
        }
        cursor += 1;
    }
    None
}

fn parse_attrs(inner: &str) -> Vec<(&str, &str)> {
    let bytes = inner.as_bytes();
    let mut attrs = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] == b'/' {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
        {
            cursor += 1;
        }
        let name = &inner[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let value = if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
                let quote = bytes[cursor];
                cursor += 1;
                let value_start = cursor;
                while cursor < bytes.len() && bytes[cursor] != quote {
                    cursor += 1;
                }
                let value = &inner[value_start..cursor];
                if cursor < bytes.len() {
                    cursor += 1;
                }
                value
            } else {
                let value_start = cursor;
                while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                &inner[value_start..cursor]
            }
        } else {
            ""
        };
        attrs.push((name, value));
    }
    attrs
}
