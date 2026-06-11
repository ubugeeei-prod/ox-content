#![allow(clippy::redundant_pub_crate)]

use compact_str::CompactString;
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::{
    JsAttrsOptions, JsEditThisPageOptions, JsEmojiShortcodeOptions, JsTransformOptions,
    JsWikiLinkOptions,
};

pub(crate) mod code_blocks;
mod code_imports;
mod emoji;

pub(crate) use code_blocks::{
    extract_code_blocks, extract_docs_tests, lint_code_blocks, CodeBlockDiagnostic,
    ExtractedCodeBlock,
};
use code_imports::CodeImportOptions;

#[derive(Clone, Default)]
pub struct TransformFeatureOptions {
    wiki_links: Option<WikiLinkOptions>,
    emoji_shortcodes: Option<EmojiShortcodeOptions>,
    code_imports: Option<CodeImportOptions>,
    attributes: bool,
    edit_this_page: Option<EditThisPageOptions>,
}

#[derive(Clone)]
struct WikiLinkOptions {
    base_url: String,
}

#[derive(Clone)]
struct EmojiShortcodeOptions {
    custom: FxHashMap<String, String>,
}

#[derive(Clone)]
struct EditThisPageOptions {
    repo_url: String,
    branch: String,
    root_dir: PathBuf,
    source_path: String,
    label: String,
}

pub struct PreprocessResult<'a> {
    pub source: Cow<'a, str>,
    pub errors: Vec<String>,
}

pub struct PostprocessResult {
    pub html: String,
    pub errors: Vec<String>,
}

impl TransformFeatureOptions {
    pub fn from_js(options: &JsTransformOptions) -> Self {
        let wiki_links = resolve_wiki_links(options.wiki_links.as_ref(), options.base_url.as_ref());
        let emoji_shortcodes = resolve_emoji_shortcodes(options.emoji_shortcodes.as_ref());
        let source_path = options.source_path.as_deref().filter(|value| !value.is_empty());
        let code_imports = code_imports::resolve(options.code_imports.as_ref(), source_path);
        let attributes = resolve_attrs(options.attributes.as_ref());
        let edit_this_page = resolve_edit_this_page(
            options.edit_this_page.as_ref(),
            source_path.unwrap_or_default(),
        );

        Self { wiki_links, emoji_shortcodes, code_imports, attributes, edit_this_page }
    }

    pub fn has_preprocess(&self) -> bool {
        self.wiki_links.is_some() || self.emoji_shortcodes.is_some() || self.code_imports.is_some()
    }

    pub fn has_postprocess(&self) -> bool {
        self.attributes || self.edit_this_page.is_some()
    }
}

pub fn preprocess_markdown<'a>(
    source: &'a str,
    options: &TransformFeatureOptions,
) -> PreprocessResult<'a> {
    if !options.has_preprocess() {
        return PreprocessResult { source: Cow::Borrowed(source), errors: Vec::new() };
    }

    let mut current = Cow::Borrowed(source);
    let mut errors = Vec::new();

    if let Some(code_imports) = &options.code_imports {
        if current.contains("<<<") {
            let replaced = code_imports::transform(&current, code_imports, &mut errors);
            current = Cow::Owned(replaced);
        }
    }

    if let Some(wiki_links) = &options.wiki_links {
        if current.contains("[[") {
            let replaced = transform_markdown_text_segments(&current, |segment, out| {
                replace_wiki_links(segment, wiki_links, out);
            });
            if let Some(replaced) = replaced {
                current = Cow::Owned(replaced);
            }
        }
    }

    if let Some(emoji) = &options.emoji_shortcodes {
        if current.contains(':') {
            let replaced = transform_markdown_text_segments(&current, |segment, out| {
                replace_emoji_shortcodes(segment, emoji, out);
            });
            if let Some(replaced) = replaced {
                current = Cow::Owned(replaced);
            }
        }
    }

    PreprocessResult { source: current, errors }
}

pub fn postprocess_html(html: &str, options: &TransformFeatureOptions) -> PostprocessResult {
    if !options.has_postprocess() {
        return PostprocessResult { html: html.to_string(), errors: Vec::new() };
    }

    let mut current = Cow::Borrowed(html);
    let errors = Vec::new();

    if options.attributes && current.contains('{') {
        let transformed = transform_attribute_syntax(&current);
        if let Some(transformed) = transformed {
            current = Cow::Owned(transformed);
        }
    }

    if let Some(edit) = &options.edit_this_page {
        let transformed = append_edit_this_page(&current, edit);
        current = Cow::Owned(transformed);
    }

    PostprocessResult { html: current.into_owned(), errors }
}

fn resolve_wiki_links(
    options: Option<&JsWikiLinkOptions>,
    default_base_url: Option<&String>,
) -> Option<WikiLinkOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(WikiLinkOptions {
        base_url: options
            .base_url
            .clone()
            .or_else(|| default_base_url.cloned())
            .unwrap_or_else(|| "/".to_string()),
    })
}

fn resolve_emoji_shortcodes(
    options: Option<&JsEmojiShortcodeOptions>,
) -> Option<EmojiShortcodeOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(EmojiShortcodeOptions {
        custom: options.custom.clone().unwrap_or_default().into_iter().collect(),
    })
}

fn resolve_attrs(options: Option<&JsAttrsOptions>) -> bool {
    options.is_some_and(|options| options.enabled != Some(false))
}

fn resolve_edit_this_page(
    options: Option<&JsEditThisPageOptions>,
    source_path: &str,
) -> Option<EditThisPageOptions> {
    let options = options?;
    if options.enabled == Some(false) || source_path.is_empty() {
        return None;
    }
    let repo_url = options.repo_url.as_deref()?.trim_end_matches('/').to_string();
    if repo_url.is_empty() {
        return None;
    }

    Some(EditThisPageOptions {
        repo_url,
        branch: options.branch.clone().unwrap_or_else(|| "main".to_string()),
        root_dir: options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
            || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from,
        ),
        source_path: source_path.to_string(),
        label: options.label.clone().unwrap_or_else(|| "Edit this page".to_string()),
    })
}

fn transform_markdown_text_segments(
    source: &str,
    mut transform: impl FnMut(&str, &mut String),
) -> Option<String> {
    let mut out = String::with_capacity(source.len());
    let mut changed = false;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = match line_with_end.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line_with_end, ""),
        };

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
                fence_char = b'\0';
                fence_len = 0;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        let before_len = out.len();
        transform_inline_code_segments(line, &mut out, &mut transform);
        let appended = &out[before_len..];
        if appended != line {
            changed = true;
        }
        out.push_str(ending);
    }

    if changed {
        Some(out)
    } else {
        None
    }
}

fn transform_inline_code_segments(
    line: &str,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'`', &bytes[cursor..]) else {
            transform(&line[cursor..], out);
            return;
        };
        let tick_start = cursor + relative;
        transform(&line[cursor..tick_start], out);
        let tick_count = count_repeated_byte(bytes, tick_start, b'`');
        let code_start = tick_start + tick_count;
        if let Some(close) = find_closing_backticks(bytes, code_start, tick_count) {
            out.push_str(&line[tick_start..close + tick_count]);
            cursor = close + tick_count;
        } else {
            out.push_str(&line[tick_start..]);
            return;
        }
    }
}

fn replace_wiki_links(segment: &str, options: &WikiLinkOptions, out: &mut String) {
    let mut cursor = 0usize;
    while let Some(relative) = segment[cursor..].find("[[") {
        let start = cursor + relative;
        let embed = start > 0 && segment.as_bytes()[start - 1] == b'!';
        let literal_start = if embed { start - 1 } else { start };
        out.push_str(&segment[cursor..literal_start]);
        let inner_start = start + 2;
        let Some(close_relative) = segment[inner_start..].find("]]") else {
            out.push_str(&segment[literal_start..]);
            return;
        };
        let close = inner_start + close_relative;
        let inner = segment[inner_start..close].trim();
        if inner.is_empty() {
            out.push_str(&segment[literal_start..close + 2]);
            cursor = close + 2;
            continue;
        }

        let (target, label) = inner
            .split_once('|')
            .map_or((inner, None), |(target, label)| (target.trim(), Some(label.trim())));
        let target = target.trim();
        let label =
            label.filter(|value| !value.is_empty()).unwrap_or_else(|| default_wiki_label(target));
        let url = wiki_target_to_url(target, &options.base_url);
        if embed {
            out.push_str("![");
        } else {
            out.push('[');
        }
        escape_markdown_link_text(label, out);
        out.push_str("](");
        out.push_str(&url);
        out.push(')');
        cursor = close + 2;
    }
    out.push_str(&segment[cursor..]);
}

fn replace_emoji_shortcodes(segment: &str, options: &EmojiShortcodeOptions, out: &mut String) {
    let bytes = segment.as_bytes();
    let mut cursor = 0usize;
    while let Some(relative) = memchr::memchr(b':', &bytes[cursor..]) {
        let start = cursor + relative;
        out.push_str(&segment[cursor..start]);
        let name_start = start + 1;
        let mut name_end = name_start;
        while name_end < bytes.len() && emoji::is_shortcode_byte(bytes[name_end]) {
            name_end += 1;
        }
        if name_end == name_start || bytes.get(name_end) != Some(&b':') {
            out.push(':');
            cursor = name_start;
            continue;
        }
        let name = &segment[name_start..name_end];
        if let Some(value) = options.custom.get(name) {
            out.push_str(value);
        } else if let Some(value) = emoji::lookup(name) {
            out.push_str(value);
        } else {
            out.push_str(&segment[start..name_end + 1]);
        }
        cursor = name_end + 1;
    }
    out.push_str(&segment[cursor..]);
}

fn transform_attribute_syntax(html: &str) -> Option<String> {
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

#[derive(Default)]
struct ParsedAttrs {
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, String)>,
}

impl ParsedAttrs {
    fn parse(value: &str) -> Option<Self> {
        if !value.contains('#') && !value.contains('.') && !value.contains('=') {
            return None;
        }
        let mut parsed = Self::default();
        for token in split_attr_tokens(value) {
            if let Some(id) = token.strip_prefix('#') {
                if !id.is_empty() {
                    parsed.id = Some(id.to_string());
                }
            } else if let Some(class) = token.strip_prefix('.') {
                if !class.is_empty() {
                    parsed.classes.push(class.to_string());
                }
            } else if let Some((name, raw_value)) = token.split_once('=') {
                let name = name.trim();
                if is_safe_attr_name(name) {
                    parsed.attrs.push((
                        name.to_string(),
                        raw_value.trim_matches(|ch| ch == '"' || ch == '\'').to_string(),
                    ));
                }
            }
        }
        if parsed.id.is_none() && parsed.classes.is_empty() && parsed.attrs.is_empty() {
            None
        } else {
            Some(parsed)
        }
    }
}

fn split_attr_tokens(value: &str) -> Vec<&str> {
    let bytes = value.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let start = cursor;
        let mut quote = None;
        while cursor < bytes.len() {
            match quote {
                Some(q) if bytes[cursor] == q => quote = None,
                Some(_) => {}
                None if bytes[cursor] == b'"' || bytes[cursor] == b'\'' => {
                    quote = Some(bytes[cursor]);
                }
                None if bytes[cursor].is_ascii_whitespace() => break,
                None => {}
            }
            cursor += 1;
        }
        tokens.push(&value[start..cursor]);
    }
    tokens
}

fn write_attrs(out: &mut String, attrs: &ParsedAttrs) {
    if let Some(id) = &attrs.id {
        out.push_str(" id=\"");
        escape_html_attr(id, out);
        out.push('"');
    }
    if !attrs.classes.is_empty() {
        out.push_str(" class=\"");
        for (index, class) in attrs.classes.iter().enumerate() {
            if index > 0 {
                out.push(' ');
            }
            escape_html_attr(class, out);
        }
        out.push('"');
    }
    for (name, value) in &attrs.attrs {
        out.push(' ');
        out.push_str(name);
        out.push_str("=\"");
        escape_html_attr(value, out);
        out.push('"');
    }
}

fn append_edit_this_page(html: &str, options: &EditThisPageOptions) -> String {
    let href = edit_this_page_href(options);
    let mut out = String::with_capacity(html.len() + href.len() + options.label.len() + 96);
    out.push_str(html);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("<p class=\"ox-edit-this-page\"><a href=\"");
    escape_html_attr(&href, &mut out);
    out.push_str("\" target=\"_blank\" rel=\"noopener noreferrer\">");
    escape_html_text(&options.label, &mut out);
    out.push_str("</a></p>\n");
    out
}

fn edit_this_page_href(options: &EditThisPageOptions) -> String {
    let source = PathBuf::from(&options.source_path);
    let absolute = if source.is_absolute() { source } else { options.root_dir.join(source) };
    let relative = absolute
        .strip_prefix(&options.root_dir)
        .ok()
        .unwrap_or(absolute.as_path())
        .to_string_lossy()
        .replace('\\', "/");
    format!("{}/edit/{}/{}", options.repo_url, options.branch, percent_encode_path(&relative))
}

fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.') {
            out.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

struct FenceOpen {
    fence_char: u8,
    fence_len: usize,
    language: CompactString,
    meta: CompactString,
}

fn parse_opening_fence(line: &str) -> Option<FenceOpen> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let fence_char = *bytes.first()?;
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }
    let fence_len = count_repeated_byte(bytes, 0, fence_char);
    if fence_len < 3 {
        return None;
    }
    let rest = trimmed[fence_len..].trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let language = CompactString::from(parts.next().unwrap_or_default());
    let meta = CompactString::from(parts.next().unwrap_or_default().trim());
    Some(FenceOpen { fence_char, fence_len, language, meta })
}

fn is_closing_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    let trimmed = line.trim();
    let bytes = trimmed.as_bytes();
    bytes.len() >= fence_len
        && bytes[..fence_len].iter().all(|value| *value == fence_char)
        && bytes[fence_len..].iter().all(|value| *value == fence_char)
}

fn count_repeated_byte(bytes: &[u8], start: usize, byte: u8) -> usize {
    let mut count = 0usize;
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] == byte {
        count += 1;
        cursor += 1;
    }
    count
}

fn find_closing_backticks(bytes: &[u8], from: usize, count: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        let relative = memchr::memchr(b'`', &bytes[cursor..])?;
        let start = cursor + relative;
        if count_repeated_byte(bytes, start, b'`') >= count {
            return Some(start);
        }
        cursor = start + 1;
    }
    None
}

fn wiki_target_to_url(target: &str, base_url: &str) -> String {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('#')
    {
        return percent_encode_spaces(target);
    }

    let (path, anchor) =
        target.split_once('#').map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let mut normalized = path.trim().trim_end_matches(".md").trim_end_matches("/index").to_string();
    if normalized.is_empty() {
        normalized.push('/');
    }
    let mut url = join_base_url(base_url, &normalized);
    if let Some(anchor) = anchor {
        if !anchor.is_empty() {
            url.push('#');
            slugify_anchor(anchor, &mut url);
        }
    }
    percent_encode_spaces(&url)
}

fn join_base_url(base_url: &str, path: &str) -> String {
    if path.starts_with('/') {
        let base = base_url.trim_end_matches('/');
        if base.is_empty() || base == "/" {
            path.to_string()
        } else {
            format!("{base}{path}")
        }
    } else {
        let base = if base_url.is_empty() { "/" } else { base_url };
        format!("{}/{}", base.trim_end_matches('/'), path)
    }
}

fn default_wiki_label(target: &str) -> &str {
    let path = target.split('#').next().unwrap_or(target).trim();
    path.rsplit('/').next().filter(|value| !value.is_empty()).unwrap_or(target)
}

fn escape_markdown_link_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        if matches!(ch, '[' | ']') {
            out.push('\\');
        }
        out.push(ch);
    }
}

fn slugify_anchor(value: &str, out: &mut String) {
    let mut last_dash = false;
    for ch in value.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
}

fn percent_encode_spaces(value: &str) -> String {
    if !value.contains(' ') {
        return value.to_string();
    }
    value.replace(' ', "%20")
}

fn is_safe_attr_name(name: &str) -> bool {
    !name.is_empty()
        && !name.to_ascii_lowercase().starts_with("on")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':' | b'_'))
}

fn escape_html_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escape_html_attr(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_links_become_markdown_links() {
        let options = WikiLinkOptions { base_url: "/docs/".to_string() };
        let mut out = String::new();
        replace_wiki_links("See [[Guide Page#Install|the guide]].", &options, &mut out);
        assert_eq!(out, "See [the guide](/docs/Guide%20Page#install).");
    }

    #[test]
    fn emoji_shortcodes_use_defaults_and_custom_values() {
        let options = EmojiShortcodeOptions {
            custom: std::iter::once(("shipit".to_string(), "ship".to_string())).collect(),
        };
        let mut out = String::new();
        replace_emoji_shortcodes(":smile: :shipit: :octocat: :unknown:", &options, &mut out);
        assert_eq!(out, "\u{1F604} ship \u{1F431} :unknown:");
    }

    #[test]
    fn extracts_docs_test_blocks_by_meta() {
        let blocks = extract_docs_tests(
            "```ts test\nexpect(1).toBe(1)\n```\n```js\nnoop()\n```",
            Some(&crate::JsDocsTestOptions {
                enabled: Some(true),
                languages: None,
                require_meta: Some(true),
            }),
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "ts");
    }

    #[test]
    fn lints_code_block_trailing_spaces() {
        let diagnostics = lint_code_blocks(
            "```ts\nconst x = 1;  \n```",
            Some(&crate::JsCodeBlockLintOptions {
                enabled: Some(true),
                languages: None,
                require_language: Some(false),
                trailing_spaces: Some(true),
            }),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
    }
}
