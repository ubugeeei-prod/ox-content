pub(super) fn find_previous_wrapped_element(
    html: &str,
    end: usize,
) -> Option<(usize, usize, usize, String)> {
    let close_start = html[..end].rfind("</")?;
    if close_start + 2 >= end {
        return None;
    }
    let close_name_end = html[close_start + 2..end].find('>')? + close_start + 2;
    if close_name_end + 1 != end {
        return None;
    }
    let tag_name = html[close_start + 2..close_name_end].trim().to_ascii_lowercase();
    if tag_name.is_empty() || !is_attr_target_tag(&tag_name) {
        return None;
    }
    let open_marker = format!("<{tag_name}");
    let open_start = html[..close_start].rfind(&open_marker)?;
    let open_end = scan_tag_end(html, open_start)?;
    Some((open_start, open_end - 1, close_name_end + 1, tag_name))
}

pub(super) fn find_previous_void_element(html: &str, end: usize) -> Option<(usize, usize, String)> {
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
        Some((open_start, open_end - 1, tag_name))
    } else {
        None
    }
}

pub(super) fn is_adjacent_inline_attr_target_tag(tag: &str) -> bool {
    matches!(tag, "a" | "img" | "code" | "span")
}

pub(super) fn is_attr_target_tag(tag: &str) -> bool {
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

pub(super) fn scan_tag_end(html: &str, start: usize) -> Option<usize> {
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
