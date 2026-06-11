#[derive(Debug)]
pub(super) struct ParsedTag<'a> {
    pub(super) name: String,
    pub(super) closing: bool,
    pub(super) self_closing: bool,
    pub(super) attrs: Vec<ParsedAttr<'a>>,
}

#[derive(Debug)]
pub(super) struct ParsedAttr<'a> {
    pub(super) name: String,
    pub(super) value: Option<&'a str>,
}

impl<'a> ParsedTag<'a> {
    pub(super) fn parse(raw: &'a str) -> Option<Self> {
        let raw = raw.trim();
        let closing = raw.starts_with('/');
        let raw = raw.strip_prefix('/').unwrap_or(raw).trim_start();
        let bytes = raw.as_bytes();
        let mut cursor = 0usize;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'/'
            && bytes[cursor] != b'>'
        {
            cursor += 1;
        }
        if cursor == 0 {
            return None;
        }
        let name = raw[..cursor].to_ascii_lowercase();
        if !is_tag_name(&name) {
            return None;
        }
        let attr_text = raw[cursor..].trim();
        let self_closing = attr_text.ends_with('/');
        let attr_text = attr_text.trim_end_matches('/').trim();
        Some(Self { name, closing, self_closing, attrs: parse_attrs(attr_text) })
    }
}

fn parse_attrs(mut raw: &str) -> Vec<ParsedAttr<'_>> {
    let mut attrs = Vec::new();
    while !raw.is_empty() {
        raw = raw.trim_start();
        if raw.is_empty() {
            break;
        }
        let bytes = raw.as_bytes();
        let mut name_end = 0usize;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && bytes[name_end] != b'='
            && bytes[name_end] != b'/'
        {
            name_end += 1;
        }
        if name_end == 0 {
            break;
        }
        let name = raw[..name_end].to_ascii_lowercase();
        raw = &raw[name_end..];
        raw = raw.trim_start();
        let mut value = None;
        if raw.starts_with('=') {
            raw = raw[1..].trim_start();
            if let Some(quote) =
                raw.as_bytes().first().copied().filter(|b| *b == b'"' || *b == b'\'')
            {
                let value_start = 1usize;
                if let Some(end) = raw[value_start..].bytes().position(|byte| byte == quote) {
                    let value_end = value_start + end;
                    value = Some(&raw[value_start..value_end]);
                    raw = &raw[value_end + 1..];
                } else {
                    value = Some(&raw[value_start..]);
                    raw = "";
                }
            } else {
                let value_end = raw
                    .bytes()
                    .position(|byte| byte.is_ascii_whitespace() || byte == b'/')
                    .unwrap_or(raw.len());
                value = Some(&raw[..value_end]);
                raw = &raw[value_end..];
            }
        }
        attrs.push(ParsedAttr { name, value });
    }
    attrs
}

pub(super) fn scan_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut cursor = start;
    let mut quote = None;
    while cursor < bytes.len() {
        match quote {
            Some(q) if bytes[cursor] == q => quote = None,
            Some(_) => {}
            None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => quote = Some(bytes[cursor]),
            None if bytes[cursor] == b'>' => return Some(cursor + 1),
            None => {}
        }
        cursor += 1;
    }
    None
}

pub(super) fn find_ci(haystack: &str, from: usize, needle: &str) -> Option<usize> {
    let hay = haystack.as_bytes();
    let pat = needle.as_bytes();
    if pat.is_empty() || from > hay.len() || hay.len() - from < pat.len() {
        return None;
    }
    let last_start = hay.len() - pat.len();
    let rest = &pat[1..];
    let mut base = from;
    while base <= last_start {
        let rel = memchr::memchr(pat[0], &hay[base..=last_start])?;
        let index = base + rel;
        if hay[index + 1..index + pat.len()].eq_ignore_ascii_case(rest) {
            return Some(index);
        }
        base = index + 1;
    }
    None
}

fn is_tag_name(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(super) fn is_attr_name(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_'))
}
