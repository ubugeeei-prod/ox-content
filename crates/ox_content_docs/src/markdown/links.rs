use std::borrow::Cow;
use std::rc::Rc;
use std::sync::OnceLock;

use rustc_hash::FxHashMap;

use super::options::MarkdownDocsOptions;
use super::paths::doc_page_href_from;
use super::regex_cache::{cached_regex, RegexCache};
use crate::string_builder::{join2, join3, join5};

#[derive(Debug, Clone)]
pub(super) struct SymbolLocation {
    // `Rc<str>` because `build_symbol_map` shares one module name across every
    // entry in a module and one file name across an entry and all its members;
    // cloning the location into the map is then a refcount bump, not a heap copy.
    pub(super) module_name: Rc<str>,
    pub(super) file_name: Rc<str>,
    pub(super) anchor: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct MarkdownLinkContext<'a> {
    pub(super) options: &'a MarkdownDocsOptions,
    pub(super) current_file_name: &'a str,
    pub(super) current_module_name: &'a str,
    pub(super) symbol_map: &'a FxHashMap<String, Vec<SymbolLocation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsdocInlineLinkKind {
    Link,
    LinkCode,
    LinkPlain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JsdocInlineLink<'a> {
    kind: JsdocInlineLinkKind,
    target: &'a str,
    label: Option<&'a str>,
}

pub(super) fn format_symbol_href(
    context: &MarkdownLinkContext<'_>,
    location: &SymbolLocation,
) -> String {
    if location.file_name.as_ref() == context.current_file_name {
        if let Some(anchor) = location.anchor.as_deref().filter(|anchor| !anchor.is_empty()) {
            join2("#", anchor)
        } else {
            doc_page_href_from(
                context.options,
                context.current_file_name,
                &location.file_name,
                None,
            )
        }
    } else {
        doc_page_href_from(
            context.options,
            context.current_file_name,
            &location.file_name,
            location.anchor.as_deref(),
        )
    }
}

pub(super) fn resolve_symbol_location<'a>(
    symbol_name: &str,
    context: &'a MarkdownLinkContext<'_>,
) -> Option<&'a SymbolLocation> {
    let locations = context.symbol_map.get(symbol_name)?;
    locations
        .iter()
        .find(|location| location.module_name.as_ref() == context.current_module_name)
        .or_else(|| {
            locations
                .iter()
                .find(|location| location.file_name.as_ref() == context.current_file_name)
        })
        .or_else(|| locations.first())
}

fn resolve_jsdoc_link_target(
    target: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Option<String> {
    let target = target.trim();
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.to_string());
    }

    let context = context?;
    resolve_symbol_location(target, context).map(|location| format_symbol_href(context, location))
}

fn parse_jsdoc_inline_link_body(body: &str) -> Option<(&str, Option<&str>)> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let (target, label) =
        body.split_once('|').map_or((body, None), |(target, label)| (target, Some(label)));
    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    Some((target, label.map(str::trim).filter(|label| !label.is_empty())))
}

fn parse_jsdoc_inline_link_at(text: &str, start: usize) -> Option<(JsdocInlineLink<'_>, usize)> {
    let after_open = text.get(start + 2..)?;
    let (kind, tag_len) = if after_open.starts_with("linkcode") {
        (JsdocInlineLinkKind::LinkCode, "linkcode".len())
    } else if after_open.starts_with("linkplain") {
        (JsdocInlineLinkKind::LinkPlain, "linkplain".len())
    } else if after_open.starts_with("link") {
        (JsdocInlineLinkKind::Link, "link".len())
    } else {
        return None;
    };

    let body_start = start + 2 + tag_len;
    if !text
        .get(body_start..)
        .and_then(|value| value.chars().next())
        .is_some_and(|value| value.is_whitespace() || value == '}')
    {
        return None;
    }

    let body_end = body_start + text.get(body_start..)?.find('}')?;
    let body = text.get(body_start..body_end)?;
    let (target, label) = parse_jsdoc_inline_link_body(body)?;

    Some((JsdocInlineLink { kind, target, label }, body_end + 1))
}

fn render_jsdoc_inline_link(
    link: &JsdocInlineLink<'_>,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let label = link.label.unwrap_or(link.target).trim();
    let label = if label.is_empty() { link.target.trim() } else { label };
    let label = if link.kind == JsdocInlineLinkKind::LinkCode {
        join3("`", label.trim_matches('`'), "`")
    } else {
        label.to_string()
    };

    if let Some(href) = resolve_jsdoc_link_target(link.target, context) {
        join5("[", &label, "](", &href, ")")
    } else {
        label
    }
}

fn convert_jsdoc_inline_links<'a>(
    text: &'a str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut cursor = 0;

    while let Some(start_offset) = text[cursor..].find("{@") {
        let start = cursor + start_offset;
        let Some((link, end)) = parse_jsdoc_inline_link_at(text, start) else {
            result.push_str(&text[cursor..start + 2]);
            cursor = start + 2;
            continue;
        };

        result.push_str(&text[cursor..start]);
        result.push_str(&render_jsdoc_inline_link(&link, context));
        cursor = end;
    }

    if cursor == 0 {
        return Cow::Borrowed(text);
    }

    result.push_str(&text[cursor..]);
    Cow::Owned(result)
}

pub(super) fn process_doc_text<'a>(
    text: &'a str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Cow<'a, str> {
    // Resolve `[Symbol]` references first, then `{@link}` inline tags. Both
    // passes borrow the input untouched when there is nothing to rewrite, so a
    // description with no links allocates nothing.
    match context {
        Some(context) => match convert_symbol_links(text, context) {
            Cow::Borrowed(borrowed) => convert_jsdoc_inline_links(borrowed, Some(context)),
            Cow::Owned(owned) => {
                Cow::Owned(convert_jsdoc_inline_links(&owned, Some(context)).into_owned())
            }
        },
        None => convert_jsdoc_inline_links(text, None),
    }
}

fn convert_symbol_links<'a>(text: &'a str, context: &MarkdownLinkContext<'_>) -> Cow<'a, str> {
    static SYMBOL_RE: RegexCache = OnceLock::new();

    let Some(symbol_re) = cached_regex(&SYMBOL_RE, r"\[([A-Z_]\w*)\]") else {
        return Cow::Borrowed(text);
    };
    let mut result = String::new();
    let mut last_index = 0;

    for captures in symbol_re.captures_iter(text) {
        let Some(mat) = captures.get(0) else {
            continue;
        };

        if text[mat.end()..].starts_with('(') {
            continue;
        }

        let symbol_name = captures.get(1).map_or("", |value| value.as_str());
        let Some(location) = resolve_symbol_location(symbol_name, context) else {
            continue;
        };

        result.push_str(&text[last_index..mat.start()]);
        result.push('[');
        result.push_str(symbol_name);
        result.push_str("](");
        result.push_str(&format_symbol_href(context, location));
        result.push(')');
        last_index = mat.end();
    }

    if last_index == 0 {
        return Cow::Borrowed(text);
    }

    result.push_str(&text[last_index..]);
    Cow::Owned(result)
}
