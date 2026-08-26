use super::attr_tokens::{ParsedAttrs, strip_quoted_attr, write_open_tag_attrs};
use super::escape_html_attr;

mod targets;
#[cfg(test)]
mod tests;

use self::targets::{
    find_previous_void_element, find_previous_wrapped_element, is_adjacent_inline_attr_target_tag,
    is_attr_target_tag, scan_tag_end,
};

const HEADER_ANCHOR_OPEN: &str = "<a class=\"header-anchor\"";

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

        if let Some(next) =
            try_apply_attrs_inside_element(html, &mut out, cursor, attr_start, attr_end, &parsed)
        {
            changed = true;
            cursor = next;
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
) -> Option<usize> {
    let close_start = skip_trailing_heading_permalink(html, attr_end + 1);
    if !html[close_start..].starts_with("</") {
        return None;
    }
    let close_name_end = html[close_start + 2..].find('>')?;
    let tag_name =
        html[close_start + 2..close_start + 2 + close_name_end].trim().to_ascii_lowercase();
    if !is_attr_target_tag(&tag_name) {
        return None;
    }
    let open_marker = format!("<{tag_name}");
    let open_start = html[..attr_start].rfind(&open_marker)?;
    let open_end = scan_tag_end(html, open_start)?;
    if open_end > attr_start {
        return None;
    }

    if !is_heading_tag(&tag_name)
        && let Some(next) = try_apply_attrs_to_adjacent_inline_child(
            html, out, cursor, open_end, attr_start, attr_end, attrs,
        )
    {
        return Some(next);
    }

    let text_end = html[..attr_start].trim_end().len();
    let heading_inner = &html[open_end..text_end];
    out.push_str(&html[cursor..open_start]);
    write_open_tag_attrs(out, &html[open_start..open_end - 1], attrs);
    out.push('>');
    out.push_str(heading_inner);
    if is_heading_tag(&tag_name)
        && let Some(id) = attrs.id()
        && close_start > attr_end + 1
    {
        out.push_str(&rewrite_header_anchor(
            &html[attr_end + 1..close_start],
            id,
            &heading_label_text(heading_inner),
        ));
        return Some(close_start);
    }
    Some(attr_end + 1)
}

fn try_apply_attrs_to_adjacent_inline_child(
    html: &str,
    out: &mut String,
    cursor: usize,
    container_open_end: usize,
    attr_start: usize,
    attr_end: usize,
    attrs: &ParsedAttrs,
) -> Option<usize> {
    let before = &html[..attr_start];
    let trimmed_end = before.trim_end().len();
    if trimmed_end == 0 || trimmed_end > attr_start {
        return None;
    }
    let whitespace = &html[trimmed_end..attr_start];

    if let Some((tag_start, tag_end, _close_end, tag_name)) =
        find_previous_wrapped_element(html, trimmed_end)
        && tag_start >= container_open_end
        && is_adjacent_inline_attr_target_tag(&tag_name)
    {
        out.push_str(&html[cursor..tag_start]);
        write_open_tag_attrs(out, &html[tag_start..tag_end], attrs);
        out.push_str(&html[tag_end..trimmed_end]);
        out.push_str(whitespace);
        return Some(attr_end + 1);
    }

    if let Some((tag_start, tag_end, tag_name)) = find_previous_void_element(html, trimmed_end)
        && tag_start >= container_open_end
        && is_adjacent_inline_attr_target_tag(&tag_name)
    {
        out.push_str(&html[cursor..tag_start]);
        write_open_tag_attrs(out, &html[tag_start..tag_end], attrs);
        out.push_str(&html[tag_end..trimmed_end]);
        out.push_str(whitespace);
        return Some(attr_end + 1);
    }

    None
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

    if let Some((tag_start, tag_end, close_end, _tag_name)) =
        find_previous_wrapped_element(html, trimmed_end)
    {
        out.push_str(&html[cursor..tag_start]);
        write_open_tag_attrs(out, &html[tag_start..tag_end], attrs);
        let inner = &html[tag_end..trimmed_end];
        if heading_open_tag(&html[tag_start..tag_end])
            && let Some(id) = attrs.id()
        {
            out.push_str(&rewrite_header_anchor(inner, id, &heading_label_text(inner)));
        } else {
            out.push_str(inner);
        }
        out.push_str(whitespace);
        return Some(close_end);
    }

    if let Some((tag_start, tag_end, _tag_name)) = find_previous_void_element(html, trimmed_end) {
        out.push_str(&html[cursor..tag_start]);
        write_open_tag_attrs(out, &html[tag_start..tag_end], attrs);
        out.push_str(&html[tag_end..trimmed_end]);
        out.push_str(whitespace);
        return Some(tag_end);
    }

    None
}

fn is_heading_tag(tag: &str) -> bool {
    matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

fn heading_open_tag(open: &str) -> bool {
    let name = open.strip_prefix('<').unwrap_or(open);
    is_heading_tag(
        name.split(|ch: char| ch.is_ascii_whitespace() || ch == '>').next().unwrap_or(""),
    )
}

fn skip_trailing_heading_permalink(html: &str, start: usize) -> usize {
    let rest = &html[start..];
    let trimmed = rest.trim_start();
    let skip_ws = rest.len() - trimmed.len();
    if !trimmed.starts_with(HEADER_ANCHOR_OPEN) {
        return start;
    }
    trimmed.find("</a>").map_or(start, |end| start + skip_ws + end + 4)
}

fn rewrite_header_anchor(fragment: &str, id: &str, heading_text: &str) -> String {
    let Some(anchor_at) = fragment.find(HEADER_ANCHOR_OPEN) else {
        return fragment.to_string();
    };
    let after = &fragment[anchor_at..];
    let Some(tag_end) = after.find('>') else {
        return fragment.to_string();
    };
    let open = strip_quoted_attr(&strip_quoted_attr(&after[..tag_end], "href"), "aria-label");
    let mut rewritten = String::with_capacity(fragment.len() + id.len() + heading_text.len() + 32);
    rewritten.push_str(&fragment[..anchor_at]);
    rewritten.push_str(&open);
    rewritten.push_str(" href=\"#");
    escape_html_attr(id, &mut rewritten);
    rewritten.push_str("\" aria-label=\"");
    if heading_text.is_empty() {
        rewritten.push_str("Permalink to this section");
    } else {
        rewritten.push_str("Permalink to &quot;");
        escape_html_attr(heading_text, &mut rewritten);
        rewritten.push_str("&quot;");
    }
    rewritten.push('"');
    rewritten.push_str(&after[tag_end..]);
    rewritten
}

fn heading_label_text(html: &str) -> String {
    let before = html.find(HEADER_ANCHOR_OPEN).map_or(html, |i| &html[..i]);
    visible_text(before)
}

fn visible_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
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
