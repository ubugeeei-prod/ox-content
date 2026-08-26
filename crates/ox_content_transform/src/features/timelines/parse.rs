use super::ResolvedTimelineOptions;
use super::date::parse_date_and_title;
use super::meta::{
    ItemMeta, parse_item_meta, push_diagnostic, split_meta_tokens, strip_trailing_meta, unquote,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TimelineOpen {
    pub(super) indent: usize,
    pub(super) colon_count: usize,
    pub(super) caption: Option<String>,
    pub(super) ordered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TimelineDate {
    pub(super) text: String,
    pub(super) datetime: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TimelineItem {
    pub(super) date: Option<TimelineDate>,
    pub(super) title: String,
    pub(super) label: Option<String>,
    pub(super) status: Option<String>,
    pub(super) href: Option<String>,
    pub(super) body: String,
}

struct RawItem {
    line: usize,
    header: String,
    body: String,
}

struct ItemMarker<'a> {
    indent: usize,
    rest: &'a str,
}

pub(super) fn parse_timeline_open(line: &str, default_ordered: bool) -> Option<TimelineOpen> {
    if super::super::segments::is_indented_code_line(line) {
        return None;
    }
    let indent = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();
    let colon_count = trimmed.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    let mut parts = trimmed[colon_count..].trim_start().splitn(2, char::is_whitespace);
    if parts.next()? != "timeline" {
        return None;
    }
    let meta = parts.next().unwrap_or_default().trim();
    let (caption, ordered) = parse_open_meta(meta, default_ordered);
    Some(TimelineOpen { indent, colon_count, caption, ordered })
}

pub(super) fn parse_timeline_items(
    body: &str,
    options: &ResolvedTimelineOptions,
) -> (Vec<TimelineItem>, Vec<String>) {
    let mut diagnostics = Vec::new();
    let mut items = Vec::new();
    let mut current: Option<RawItem> = None;
    let mut item_indent = 0usize;

    for (index, line_with_end) in body.split_inclusive('\n').enumerate() {
        let (line, ending) = split_ending(line_with_end);
        if let Some(marker) = parse_item_marker(line) {
            if let Some(item) = current.take() {
                push_parsed_item(item, options, &mut items, &mut diagnostics);
            }
            item_indent = marker.indent;
            current = Some(RawItem {
                line: index + 1,
                header: marker.rest.trim().to_string(),
                body: String::new(),
            });
            continue;
        }
        if let Some(item) = current.as_mut() {
            item.body.push_str(strip_item_indent(line, item_indent));
            item.body.push_str(ending);
        } else if !line.trim().is_empty() {
            push_diagnostic(
                options.unknown_meta,
                format!("Timeline line {} must start with a list item.", index + 1),
                &mut diagnostics,
            );
        }
    }

    if let Some(item) = current.take() {
        push_parsed_item(item, options, &mut items, &mut diagnostics);
    }
    (items, diagnostics)
}

fn push_parsed_item(
    raw: RawItem,
    options: &ResolvedTimelineOptions,
    items: &mut Vec<TimelineItem>,
    diagnostics: &mut Vec<String>,
) {
    let (header, meta_source) = strip_trailing_meta(&raw.header);
    let mut meta = ItemMeta::default();
    if let Some(source) = meta_source {
        parse_item_meta(source, raw.line, options, diagnostics, &mut meta);
    }
    let (date, title) = parse_date_and_title(header.trim(), raw.line, options, diagnostics);
    items.push(TimelineItem {
        date,
        title: title.trim().to_string(),
        label: meta.label,
        status: meta.status,
        href: meta.href,
        body: raw.body.trim_end().to_string(),
    });
}

fn parse_open_meta(meta: &str, default_ordered: bool) -> (Option<String>, bool) {
    if meta.is_empty() {
        return (None, default_ordered);
    }
    let mut caption = None;
    let mut ordered = default_ordered;
    for token in split_meta_tokens(meta) {
        if token == "unordered" || token == "ul" {
            ordered = false;
            continue;
        }
        if token == "ordered" || token == "ol" {
            ordered = true;
            continue;
        }
        let Some((key, value)) = token.split_once('=') else {
            caption.get_or_insert_with(|| unquote(&token).to_string());
            continue;
        };
        match key {
            "caption" | "title" => caption = Some(unquote(value).to_string()),
            "ordered" => ordered = value != "false",
            _ => {}
        }
    }
    (caption, ordered)
}

fn parse_item_marker(line: &str) -> Option<ItemMarker<'_>> {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 4 && bytes[indent] == b' ' {
        indent += 1;
    }
    if indent >= 4 {
        return None;
    }
    let rest = &line[indent..];
    if let Some(rest) = rest.strip_prefix("- ").or_else(|| rest.strip_prefix("* ")) {
        return Some(ItemMarker { indent, rest });
    }
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0
        && rest.as_bytes().get(digits) == Some(&b'.')
        && rest[digits + 1..].starts_with(' ')
    {
        return Some(ItemMarker { indent, rest: &rest[digits + 2..] });
    }
    None
}

fn strip_item_indent(line: &str, item_indent: usize) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < item_indent + 2 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}

fn split_ending(line_with_end: &str) -> (&str, &str) {
    if let Some(line) = line_with_end.strip_suffix("\r\n") {
        (line, "\n")
    } else if let Some(line) = line_with_end.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_end.strip_suffix('\r').unwrap_or(line_with_end), "")
    }
}
