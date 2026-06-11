use super::attr_tokens::{write_attrs, ParsedAttrs};

pub(super) fn transform_attribute_syntax(html: &str) -> Option<String> {
    let bytes = html.as_bytes();
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'{', &bytes[cursor..]) else {
            break;
        };
        let attr_start = cursor + relative;
        let Some(attr_end) = find_attr_block_end(html, attr_start) else {
            cursor = attr_start + 1;
            continue;
        };
        let attrs = &html[attr_start + 1..attr_end];
        let Some(parsed) = ParsedAttrs::parse(attrs) else {
            cursor = attr_start + 1;
            continue;
        };

        if try_apply_attrs_inside_element(html, &mut out, cursor, attr_start, attr_end, &parsed) {
            changed = true;
            cursor = attr_end + 1;
            continue;
        }

        if try_apply_attrs_to_previous_element(html, &mut out, cursor, attr_start, &parsed)
            .is_some()
        {
            changed = true;
            cursor = attr_end + 1;
            continue;
        }
        cursor = attr_start + 1;
    }

    if !changed {
        return None;
    }
    out.push_str(&html[cursor..]);
    Some(out)
}

fn try_apply_attrs_inside_element(
    html: &str,
    out: &mut String,
    cursor: usize,
    attr_start: usize,
    attr_end: usize,
    attrs: &ParsedAttrs,
) -> bool {
    let close_start = attr_end + 1;
    if !html[close_start..].starts_with("</") {
        return false;
    }
    let Some(close_name_end) = html[close_start + 2..].find('>') else {
        return false;
    };
    let tag_name =
        html[close_start + 2..close_start + 2 + close_name_end].trim().to_ascii_lowercase();
    if !is_attr_target_tag(&tag_name) {
        return false;
    }
    let open_marker = format!("<{tag_name}");
    let Some(open_start) = html[..attr_start].rfind(&open_marker) else {
        return false;
    };
    let Some(open_end) = scan_tag_end(html, open_start) else {
        return false;
    };
    if open_end > attr_start {
        return false;
    }

    let text_end = html[..attr_start].trim_end().len();
    out.push_str(&html[cursor..open_start]);
    out.push_str(&html[open_start..open_end - 1]);
    write_attrs(out, attrs);
    out.push('>');
    out.push_str(&html[open_end..text_end]);
    true
}

fn try_apply_attrs_to_previous_element(
    html: &str,
    out: &mut String,
    cursor: usize,
    attr_start: usize,
    attrs: &ParsedAttrs,
) -> Option<usize> {
    let before = &html[..attr_start];
    let trimmed_end = before.trim_end().len();
    if trimmed_end == 0 || trimmed_end > attr_start {
        return None;
    }
    let whitespace = &html[trimmed_end..attr_start];

    if let Some((tag_start, tag_end, close_end)) = find_previous_wrapped_element(html, trimmed_end)
    {
        out.push_str(&html[cursor..tag_start]);
        out.push_str(&html[tag_start..tag_end]);
        write_attrs(out, attrs);
        out.push_str(&html[tag_end..trimmed_end]);
        out.push_str(whitespace);
        return Some(close_end);
    }

    if let Some((tag_start, tag_end)) = find_previous_void_element(html, trimmed_end) {
        out.push_str(&html[cursor..tag_start]);
        out.push_str(&html[tag_start..tag_end]);
        write_attrs(out, attrs);
        out.push_str(&html[tag_end..trimmed_end]);
        out.push_str(whitespace);
        return Some(tag_end);
    }

    None
}

fn find_previous_wrapped_element(html: &str, end: usize) -> Option<(usize, usize, usize)> {
    let close_start = html[..end].rfind("</")?;
    if close_start + 2 >= end {
        return None;
    }
    let close_name_end = html[close_start + 2..end].find('>')? + close_start + 2;
    let tag_name = html[close_start + 2..close_name_end].trim().to_ascii_lowercase();
    if tag_name.is_empty() || !is_attr_target_tag(&tag_name) {
        return None;
    }
    let open_marker = format!("<{tag_name}");
    let open_start = html[..close_start].rfind(&open_marker)?;
    let open_end = scan_tag_end(html, open_start)?;
    Some((open_start, open_end - 1, close_name_end + 1))
}

fn find_previous_void_element(html: &str, end: usize) -> Option<(usize, usize)> {
    let open_start = html[..end].rfind('<')?;
    let open_end = scan_tag_end(html, open_start)?;
    if open_end != end {
        return None;
    }
    let name_start = open_start + 1;
    let mut name_end = name_start;
    let bytes = html.as_bytes();
    while name_end < bytes.len()
        && !bytes[name_end].is_ascii_whitespace()
        && bytes[name_end] != b'>'
        && bytes[name_end] != b'/'
    {
        name_end += 1;
    }
    let tag_name = html[name_start..name_end].to_ascii_lowercase();
    if matches!(tag_name.as_str(), "img" | "br" | "hr" | "input") {
        Some((open_start, open_end - 1))
    } else {
        None
    }
}

fn is_attr_target_tag(tag: &str) -> bool {
    matches!(
        tag,
        "h1" | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "a"
            | "img"
            | "code"
            | "pre"
            | "p"
            | "div"
            | "span"
            | "blockquote"
            | "table"
            | "tr"
            | "th"
            | "td"
            | "ul"
            | "ol"
            | "li"
    )
}

fn scan_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut i = start;
    let mut quote = None;
    while i < bytes.len() {
        match quote {
            Some(q) if bytes[i] == q => quote = None,
            Some(_) => {}
            None if bytes[i] == b'"' || bytes[i] == b'\'' => quote = Some(bytes[i]),
            None if bytes[i] == b'>' => return Some(i + 1),
            None => {}
        }
        i += 1;
    }
    None
}

fn find_attr_block_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'}' => return Some(i),
            b'\n' | b'\r' | b'<' | b'>' => return None,
            _ => i += 1,
        }
    }
    None
}
