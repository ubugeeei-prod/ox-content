//! HTML renderer implementation.

use std::collections::BTreeMap;
use std::fmt::{Display, Write as _};

use ox_content_ast::{
    BlockQuote, Break, CodeBlock, Definition, Delete, Document, Emphasis, FootnoteDefinition,
    FootnoteReference, Heading, Html, Image, InlineCode, Link, List, ListItem, Node, Paragraph,
    Strong, Table, TableCell, TableRow, Text, ThematicBreak, Visit,
};
use rustc_hash::FxHashMap;

#[allow(unused_imports)]
// The macro is no-op without the `profile` feature, which suppresses the use.
use crate::profile_span;
use crate::render::{RenderResult, Renderer};

/// HTML renderer options.
#[derive(Debug, Clone)]
pub struct HtmlRendererOptions {
    /// Use XHTML-style self-closing tags (e.g., `<br />`).
    pub xhtml: bool,
    /// Add soft breaks between inline elements.
    pub soft_break: String,
    /// Add hard breaks.
    pub hard_break: String,
    /// Enable syntax highlighting for code blocks.
    pub highlight: bool,
    /// Sanitize HTML output.
    pub sanitize: bool,
    /// Convert `.md` links to `.html` links for SSG output.
    pub convert_md_links: bool,
    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    pub base_url: String,
    /// Source file path for relative link resolution.
    /// Used to determine if the current file is an index file.
    pub source_path: String,
    /// Enable line annotations for code blocks using fence meta.
    pub code_annotations: bool,
    /// Fence meta key used to read code annotations.
    pub code_annotation_meta_key: String,
    /// Code annotation syntax mode.
    pub code_annotation_syntax: CodeAnnotationSyntax,
    /// Enable line numbers for all code blocks by default.
    pub code_annotation_default_line_numbers: bool,
    /// Maximum heading depth included in inline TOCs.
    pub toc_max_depth: u8,
    /// Auto-link bare URLs in text. When enabled, any occurrence in a text
    /// node that starts with one of [`Self::autolink_patterns`] is wrapped
    /// in an `<a>` tag. Auto-linking is suppressed inside an existing link.
    pub autolink_urls: bool,
    /// URL prefix patterns recognised by [`Self::autolink_urls`]. Defaults
    /// to `["http://", "https://"]`. Register additional schemes (e.g.
    /// `"ftp://"`, `"mailto:"`) by pushing onto this vec.
    pub autolink_patterns: Vec<String>,
    /// When auto-linking, emit `target="_blank" rel="noopener noreferrer"`.
    /// Independent from the existing markdown-link behaviour, which always
    /// adds the attributes for http/https hrefs.
    pub autolink_target_blank: bool,
}

impl HtmlRendererOptions {
    /// Creates new options with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            xhtml: false,
            soft_break: "\n".to_string(),
            hard_break: "<br>\n".to_string(),
            highlight: false,
            sanitize: false,
            convert_md_links: false,
            base_url: "/".to_string(),
            source_path: String::new(),
            code_annotations: false,
            code_annotation_meta_key: "annotate".to_string(),
            code_annotation_syntax: CodeAnnotationSyntax::Attribute,
            code_annotation_default_line_numbers: false,
            toc_max_depth: 3,
            autolink_urls: false,
            autolink_patterns: vec!["http://".to_string(), "https://".to_string()],
            autolink_target_blank: true,
        }
    }
}

impl Default for HtmlRendererOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeAnnotationSyntax {
    Attribute,
    VitePress,
    Both,
}

impl CodeAnnotationSyntax {
    fn includes_attribute(self) -> bool {
        matches!(self, Self::Attribute | Self::Both)
    }

    fn includes_vitepress(self) -> bool {
        matches!(self, Self::VitePress | Self::Both)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeAnnotationKind {
    Highlight,
    Warning,
    Error,
    Add,
    Remove,
    Focus,
}

impl CodeAnnotationKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "highlight" => Some(Self::Highlight),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            "add" => Some(Self::Add),
            "remove" => Some(Self::Remove),
            "focus" => Some(Self::Focus),
            _ => None,
        }
    }

    fn class_name(self) -> &'static str {
        match self {
            Self::Highlight => "ox-code-line--highlight",
            Self::Warning => "ox-code-line--warning",
            Self::Error => "ox-code-line--error",
            Self::Add => "ox-code-line--add",
            Self::Remove => "ox-code-line--remove",
            Self::Focus => "ox-code-line--focus",
        }
    }

    fn extra_class_names(self) -> &'static [&'static str] {
        match self {
            Self::Highlight => &["highlighted"],
            Self::Warning => &["highlighted", "warning"],
            Self::Error => &["highlighted", "error"],
            Self::Add => &["diff", "add"],
            Self::Remove => &["diff", "remove"],
            Self::Focus => &["focused"],
        }
    }

    fn block_class_name(self) -> Option<&'static str> {
        match self {
            Self::Highlight | Self::Warning | Self::Error => Some("has-highlighted"),
            Self::Add | Self::Remove => Some("has-diff"),
            Self::Focus => Some("has-focused"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalloutKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl CalloutKind {
    fn parse_marker(value: &str) -> Option<(Self, &str)> {
        let marker = value.strip_prefix("[!")?;
        let end = marker.find(']')?;
        // Allocation-free: the previous `to_ascii_uppercase().as_str()`
        // path allocated a fresh `String` for every `[!FOO]`-prefixed
        // text run that reached this branch. `eq_ignore_ascii_case`
        // compares the trimmed slice in place against each known label.
        let name = marker[..end].trim();
        let kind = if name.eq_ignore_ascii_case("NOTE") {
            Self::Note
        } else if name.eq_ignore_ascii_case("TIP") {
            Self::Tip
        } else if name.eq_ignore_ascii_case("IMPORTANT") {
            Self::Important
        } else if name.eq_ignore_ascii_case("WARNING") {
            Self::Warning
        } else if name.eq_ignore_ascii_case("CAUTION") {
            Self::Caution
        } else {
            return None;
        };

        Some((kind, marker[end + 1..].trim_start_matches(char::is_whitespace)))
    }

    fn class_name(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Tip => "Tip",
            Self::Important => "Important",
            Self::Warning => "Warning",
            Self::Caution => "Caution",
        }
    }
}

#[derive(Debug, Clone)]
struct CodeLineRenderState {
    value: String,
    annotations: Vec<CodeAnnotationKind>,
}

#[derive(Debug, Clone)]
struct CodeBlockRenderState {
    language: Option<String>,
    title: Option<String>,
    line_numbers_start: Option<usize>,
    lines: Vec<CodeLineRenderState>,
}

impl CodeBlockRenderState {
    fn has_annotations(&self) -> bool {
        self.lines.iter().any(|line| !line.annotations.is_empty())
    }

    fn has_focus(&self) -> bool {
        self.lines.iter().any(|line| line.annotations.contains(&CodeAnnotationKind::Focus))
    }

    fn block_classes(&self) -> Vec<&'static str> {
        let mut classes = Vec::new();
        if self.has_annotations() || self.line_numbers_start.is_some() || self.title.is_some() {
            classes.push("ox-code-block");
        }
        if self.has_annotations() {
            classes.push("ox-code-block--annotated");
        }
        if self.line_numbers_start.is_some() {
            classes.push("ox-code-block--line-numbers");
            classes.push("line-numbers-mode");
        }
        if self.title.is_some() {
            classes.push("ox-code-block--with-title");
        }

        for line in &self.lines {
            for annotation in &line.annotations {
                if let Some(class_name) = annotation.block_class_name() {
                    if !classes.contains(&class_name) {
                        classes.push(class_name);
                    }
                }
            }
        }

        classes
    }

    fn needs_line_wrappers(&self) -> bool {
        self.has_annotations() || self.line_numbers_start.is_some()
    }
}

#[derive(Debug, Clone)]
struct NormalizedCodeBlockInfo {
    language: Option<String>,
    meta: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetaTokenKind {
    Raw,
    Braces,
    Brackets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MetaToken<'a> {
    kind: MetaTokenKind,
    value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingCodeAnnotation {
    kind: CodeAnnotationKind,
    remaining: usize,
}

#[derive(Debug, Clone)]
struct ParsedInlineDirective {
    kind: CodeAnnotationKind,
    count: usize,
    stripped_line: String,
    standalone: bool,
}

fn parse_code_annotations(meta: &str, key: &str) -> BTreeMap<usize, Vec<CodeAnnotationKind>> {
    let Some(value) = extract_meta_attribute(meta, key) else {
        return BTreeMap::new();
    };

    let mut annotations = BTreeMap::new();

    for entry in value.split(';') {
        let Some((raw_kind, raw_lines)) = entry.split_once(':') else {
            continue;
        };

        let Some(kind) = CodeAnnotationKind::from_str(raw_kind.trim()) else {
            continue;
        };

        for line_number in parse_line_numbers(raw_lines.trim()) {
            push_code_annotation(&mut annotations, line_number, kind);
        }
    }

    annotations
}

fn extract_meta_attribute<'a>(meta: &'a str, target: &str) -> Option<&'a str> {
    let bytes = meta.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let key_start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'=' {
            index += 1;
        }

        if key_start == index {
            index += 1;
            continue;
        }

        let key = &meta[key_start..index];

        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }

        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let value = if bytes[index] == b'"' || bytes[index] == b'\'' {
            let quote = bytes[index];
            index += 1;
            let value_start = index;

            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }

            let value_end = index;
            if index < bytes.len() {
                index += 1;
            }
            &meta[value_start..value_end]
        } else {
            let value_start = index;
            while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            &meta[value_start..index]
        };

        if key == target {
            return Some(value);
        }
    }

    None
}

fn parse_line_numbers(value: &str) -> Vec<usize> {
    let mut line_numbers = Vec::new();

    for part in value.split(',').map(str::trim).filter(|part| !part.is_empty()) {
        if let Some((raw_start, raw_end)) = part.split_once('-') {
            let Ok(start) = raw_start.trim().parse::<usize>() else {
                continue;
            };
            let Ok(end) = raw_end.trim().parse::<usize>() else {
                continue;
            };

            if start == 0 || end < start {
                continue;
            }

            for line_number in start..=end {
                if !line_numbers.contains(&line_number) {
                    line_numbers.push(line_number);
                }
            }
            continue;
        }

        let Ok(line_number) = part.parse::<usize>() else {
            continue;
        };

        if line_number > 0 && !line_numbers.contains(&line_number) {
            line_numbers.push(line_number);
        }
    }

    line_numbers.sort_unstable();
    line_numbers
}

fn push_code_annotation(
    annotations: &mut BTreeMap<usize, Vec<CodeAnnotationKind>>,
    line_number: usize,
    kind: CodeAnnotationKind,
) {
    let kinds = annotations.entry(line_number).or_default();
    if !kinds.contains(&kind) {
        kinds.push(kind);
    }
}

fn split_code_block_meta(meta: &str) -> Vec<MetaToken<'_>> {
    let bytes = meta.as_bytes();
    let mut index = 0;
    let mut tokens = Vec::new();

    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        match bytes[index] {
            b'{' => {
                let start = index + 1;
                index += 1;
                while index < bytes.len() && bytes[index] != b'}' {
                    index += 1;
                }
                tokens.push(MetaToken {
                    kind: MetaTokenKind::Braces,
                    value: &meta[start..index.min(bytes.len())],
                });
                if index < bytes.len() {
                    index += 1;
                }
            }
            b'[' => {
                let start = index + 1;
                index += 1;
                while index < bytes.len() && bytes[index] != b']' {
                    index += 1;
                }
                tokens.push(MetaToken {
                    kind: MetaTokenKind::Brackets,
                    value: &meta[start..index.min(bytes.len())],
                });
                if index < bytes.len() {
                    index += 1;
                }
            }
            _ => {
                let start = index;
                let mut quote: Option<u8> = None;

                while index < bytes.len() {
                    let byte = bytes[index];
                    if let Some(current_quote) = quote {
                        if byte == current_quote {
                            quote = None;
                        }
                        index += 1;
                        continue;
                    }

                    if byte == b'"' || byte == b'\'' {
                        quote = Some(byte);
                        index += 1;
                        continue;
                    }

                    if byte.is_ascii_whitespace() || byte == b'{' || byte == b'[' {
                        break;
                    }

                    index += 1;
                }

                tokens.push(MetaToken { kind: MetaTokenKind::Raw, value: &meta[start..index] });
            }
        }
    }

    tokens
}

fn split_code_block_language_token(raw: &str) -> (&str, &str) {
    for (index, ch) in raw.char_indices() {
        match ch {
            '{' | '[' => return (&raw[..index], &raw[index..]),
            ':' if raw[index..].starts_with(":line-numbers")
                || raw[index..].starts_with(":no-line-numbers") =>
            {
                return (&raw[..index], &raw[index..]);
            }
            _ => {}
        }
    }

    (raw, "")
}

fn normalize_code_block_info(lang: Option<&str>, meta: Option<&str>) -> NormalizedCodeBlockInfo {
    let mut meta_parts: Vec<&str> = Vec::new();
    let mut language = None;

    if let Some(raw_lang) = lang.map(str::trim).filter(|value| !value.is_empty()) {
        let (normalized_lang, inline_meta) = split_code_block_language_token(raw_lang);
        if !normalized_lang.is_empty() {
            language = Some(normalized_lang.to_string());
        }
        if !inline_meta.trim().is_empty() {
            meta_parts.push(inline_meta.trim());
        }
    }

    if let Some(raw_meta) = meta.map(str::trim).filter(|value| !value.is_empty()) {
        meta_parts.push(raw_meta);
    }

    NormalizedCodeBlockInfo { language, meta: meta_parts.join(" ") }
}

fn normalize_code_block_language(lang: Option<&str>) -> Option<&str> {
    let raw_lang = lang.map(str::trim).filter(|value| !value.is_empty())?;
    let (language, _) = split_code_block_language_token(raw_lang);
    let language = language.trim();

    if language.is_empty() {
        None
    } else {
        Some(language)
    }
}

fn apply_annotation_numbers(
    lines: &mut [CodeLineRenderState],
    line_numbers: &[usize],
    kind: CodeAnnotationKind,
) {
    for line_number in line_numbers {
        let Some(line) = lines.get_mut(line_number.saturating_sub(1)) else {
            continue;
        };

        if !line.annotations.contains(&kind) {
            line.annotations.push(kind);
        }
    }
}

fn apply_btree_annotations(
    lines: &mut [CodeLineRenderState],
    annotations: &BTreeMap<usize, Vec<CodeAnnotationKind>>,
) {
    for (line_number, kinds) in annotations {
        let Some(line) = lines.get_mut(line_number.saturating_sub(1)) else {
            continue;
        };
        for kind in kinds {
            if !line.annotations.contains(kind) {
                line.annotations.push(*kind);
            }
        }
    }
}

fn apply_pending_annotations(
    line: &mut CodeLineRenderState,
    pending_annotations: &mut Vec<PendingCodeAnnotation>,
) {
    let mut remaining = Vec::new();

    for mut pending in pending_annotations.drain(..) {
        if !line.annotations.contains(&pending.kind) {
            line.annotations.push(pending.kind);
        }

        if pending.remaining > 1 {
            pending.remaining -= 1;
            remaining.push(pending);
        }
    }

    *pending_annotations = remaining;
}

fn parse_annotation_count(value: &str) -> usize {
    value.trim().parse::<usize>().ok().filter(|count| *count > 0).unwrap_or(1)
}

fn parse_vitepress_directive_kind(value: &str) -> Option<(CodeAnnotationKind, usize)> {
    let trimmed = value.trim();

    if trimmed == "++" {
        return Some((CodeAnnotationKind::Add, 1));
    }

    if trimmed == "--" {
        return Some((CodeAnnotationKind::Remove, 1));
    }

    if let Some((kind, count)) = trimmed.split_once(':') {
        let parsed_kind = match kind.trim() {
            "highlight" => CodeAnnotationKind::Highlight,
            "focus" => CodeAnnotationKind::Focus,
            "warning" => CodeAnnotationKind::Warning,
            "error" => CodeAnnotationKind::Error,
            _ => return None,
        };
        return Some((parsed_kind, parse_annotation_count(count)));
    }

    match trimmed {
        "highlight" => Some((CodeAnnotationKind::Highlight, 1)),
        "warning" => Some((CodeAnnotationKind::Warning, 1)),
        "error" => Some((CodeAnnotationKind::Error, 1)),
        "focus" => Some((CodeAnnotationKind::Focus, 1)),
        _ => None,
    }
}

fn parse_vitepress_inline_directive(line: &str) -> Option<ParsedInlineDirective> {
    let marker_start = line.find("[!code ")?;
    let directive_start = marker_start + "[!code ".len();
    let marker_end = line[directive_start..].find(']')? + directive_start;
    let directive = &line[directive_start..marker_end];

    let before_marker = &line[..marker_start];
    let after_marker = &line[marker_end + 1..];
    let trimmed_before = before_marker.trim_end();

    let (comment_start, requires_closer) = if trimmed_before.ends_with("//") {
        (trimmed_before.len() - 2, false)
    } else if trimmed_before.ends_with('#') {
        (trimmed_before.len() - 1, false)
    } else if trimmed_before.ends_with("<!--") {
        (trimmed_before.len() - 4, true)
    } else if trimmed_before.ends_with("/*") {
        (trimmed_before.len() - 2, true)
    } else {
        return None;
    };

    let trailing = after_marker.trim();
    if requires_closer && trailing != "-->" && trailing != "*/" {
        return None;
    }
    if !requires_closer && !trailing.is_empty() {
        return None;
    }

    let stripped_line = before_marker[..comment_start].trim_end().to_string();
    let standalone = stripped_line.trim().is_empty();
    let (kind, count) = parse_vitepress_directive_kind(directive)?;

    Some(ParsedInlineDirective { kind, count, stripped_line, standalone })
}

fn parse_vitepress_inline_annotations(value: &str) -> Vec<CodeLineRenderState> {
    let mut lines = Vec::new();
    let mut pending_annotations: Vec<PendingCodeAnnotation> = Vec::new();

    for raw_line in value.split('\n') {
        if let Some(directive) = parse_vitepress_inline_directive(raw_line) {
            if directive.standalone {
                pending_annotations.push(PendingCodeAnnotation {
                    kind: directive.kind,
                    remaining: directive.count,
                });
                continue;
            }

            let mut line =
                CodeLineRenderState { value: directive.stripped_line, annotations: Vec::new() };
            apply_pending_annotations(&mut line, &mut pending_annotations);
            if !line.annotations.contains(&directive.kind) {
                line.annotations.push(directive.kind);
            }
            if directive.count > 1 {
                pending_annotations.push(PendingCodeAnnotation {
                    kind: directive.kind,
                    remaining: directive.count - 1,
                });
            }
            lines.push(line);
            continue;
        }

        let mut line = CodeLineRenderState { value: raw_line.to_string(), annotations: Vec::new() };
        apply_pending_annotations(&mut line, &mut pending_annotations);
        lines.push(line);
    }

    lines
}

// Per-byte HTML-escape mapping. `ESCAPE_FLAG[b] == 1` and
// `ESCAPE_TABLE[b]` is the replacement string when `b` must be escaped;
// otherwise the flag is 0 and the entry is `""`. Splitting flag/string
// lets the inner scan use a plain integer OR over 8-byte chunks (which
// LLVM vectorizes) instead of branching on a string comparison.
static ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "&lt;";
    table[b'>' as usize] = "&gt;";
    table[b'"' as usize] = "&quot;";
    table[b'\'' as usize] = "&#39;";
    table
};

static ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b'\'' as usize] = 1;
    t
};

static URL_ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "%3C";
    table[b'>' as usize] = "%3E";
    table[b'"' as usize] = "%22";
    table[b' ' as usize] = "%20";
    table
};

static URL_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b' ' as usize] = 1;
    t
};

#[inline]
fn write_escaped_into(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    out.reserve(s.len());

    // 8-byte chunk fast-skip — the OR over 8 lookups vectorizes well and
    // dominates the inner loop on long no-escape runs (most plain text).
    while i + 8 <= bytes.len() {
        let chunk = &bytes[i..i + 8];
        let mask = ESCAPE_FLAG[chunk[0] as usize]
            | ESCAPE_FLAG[chunk[1] as usize]
            | ESCAPE_FLAG[chunk[2] as usize]
            | ESCAPE_FLAG[chunk[3] as usize]
            | ESCAPE_FLAG[chunk[4] as usize]
            | ESCAPE_FLAG[chunk[5] as usize]
            | ESCAPE_FLAG[chunk[6] as usize]
            | ESCAPE_FLAG[chunk[7] as usize];
        if mask == 0 {
            i += 8;
            continue;
        }
        break;
    }

    while i < bytes.len() {
        let b = bytes[i];
        if ESCAPE_FLAG[b as usize] != 0 {
            if start < i {
                out.push_str(&s[start..i]);
            }
            out.push_str(ESCAPE_TABLE[b as usize]);
            i += 1;
            start = i;
            while i + 8 <= bytes.len() {
                let chunk = &bytes[i..i + 8];
                let mask = ESCAPE_FLAG[chunk[0] as usize]
                    | ESCAPE_FLAG[chunk[1] as usize]
                    | ESCAPE_FLAG[chunk[2] as usize]
                    | ESCAPE_FLAG[chunk[3] as usize]
                    | ESCAPE_FLAG[chunk[4] as usize]
                    | ESCAPE_FLAG[chunk[5] as usize]
                    | ESCAPE_FLAG[chunk[6] as usize]
                    | ESCAPE_FLAG[chunk[7] as usize];
                if mask == 0 {
                    i += 8;
                    continue;
                }
                break;
            }
            continue;
        }
        i += 1;
    }

    if start < bytes.len() {
        out.push_str(&s[start..]);
    }
}

fn write_url_escaped_into(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;

    while i + 8 <= bytes.len() {
        let chunk = &bytes[i..i + 8];
        let mask = URL_ESCAPE_FLAG[chunk[0] as usize]
            | URL_ESCAPE_FLAG[chunk[1] as usize]
            | URL_ESCAPE_FLAG[chunk[2] as usize]
            | URL_ESCAPE_FLAG[chunk[3] as usize]
            | URL_ESCAPE_FLAG[chunk[4] as usize]
            | URL_ESCAPE_FLAG[chunk[5] as usize]
            | URL_ESCAPE_FLAG[chunk[6] as usize]
            | URL_ESCAPE_FLAG[chunk[7] as usize];
        if mask == 0 {
            i += 8;
            continue;
        }
        break;
    }

    while i < bytes.len() {
        let b = bytes[i];
        if URL_ESCAPE_FLAG[b as usize] != 0 {
            if start < i {
                out.push_str(&s[start..i]);
            }
            out.push_str(URL_ESCAPE_TABLE[b as usize]);
            i += 1;
            start = i;
            while i + 8 <= bytes.len() {
                let chunk = &bytes[i..i + 8];
                let mask = URL_ESCAPE_FLAG[chunk[0] as usize]
                    | URL_ESCAPE_FLAG[chunk[1] as usize]
                    | URL_ESCAPE_FLAG[chunk[2] as usize]
                    | URL_ESCAPE_FLAG[chunk[3] as usize]
                    | URL_ESCAPE_FLAG[chunk[4] as usize]
                    | URL_ESCAPE_FLAG[chunk[5] as usize]
                    | URL_ESCAPE_FLAG[chunk[6] as usize]
                    | URL_ESCAPE_FLAG[chunk[7] as usize];
                if mask == 0 {
                    i += 8;
                    continue;
                }
                break;
            }
            continue;
        }
        i += 1;
    }

    if start < bytes.len() {
        out.push_str(&s[start..]);
    }
}

/// Case-insensitive index over the first byte of every registered autolink
/// pattern, used to skip the long runs of text that can't begin a URL.
///
/// The default patterns (`http://`, `https://`) share the single leading
/// letter `h`, so [`Self::next`] collapses to a `memchr2` over `{b'h', b'H'}`
/// — letting the scanner jump straight to candidate offsets instead of
/// testing the word-boundary + prefix at every byte. Up to three distinct
/// leading bytes keep the SIMD `memchr` fast path; beyond that (rare, only
/// with many custom schemes) it falls back to a 256-entry lookup table.
struct FirstByteIndex {
    table: [bool; 256],
    needles: [u8; 3],
    needle_len: usize,
    overflow: bool,
}

impl FirstByteIndex {
    fn from_patterns(patterns: &[String]) -> Self {
        let mut table = [false; 256];
        let mut needles = [0u8; 3];
        let mut needle_len = 0usize;
        let mut overflow = false;
        for pat in patterns {
            let Some(&first) = pat.as_bytes().first() else {
                continue;
            };
            for cand in [first.to_ascii_lowercase(), first.to_ascii_uppercase()] {
                if table[cand as usize] {
                    continue;
                }
                table[cand as usize] = true;
                if needle_len < needles.len() {
                    needles[needle_len] = cand;
                }
                // Count past the array so >3 distinct bytes trips `overflow`.
                needle_len += 1;
            }
        }
        if needle_len > needles.len() {
            overflow = true;
        }
        Self { table, needles, needle_len, overflow }
    }

    /// Byte offset of the next possible pattern start within `hay`, or `None`.
    #[inline]
    fn next(&self, hay: &[u8]) -> Option<usize> {
        if self.overflow {
            return hay.iter().position(|&b| self.table[b as usize]);
        }
        match self.needle_len {
            1 => memchr::memchr(self.needles[0], hay),
            2 => memchr::memchr2(self.needles[0], self.needles[1], hay),
            3 => memchr::memchr3(self.needles[0], self.needles[1], self.needles[2], hay),
            _ => None,
        }
    }
}

/// Scans `s` from `from` for the next position that begins one of the
/// registered URL prefixes at a word boundary, and returns the
/// `(match_start, url_end)` byte range with trailing punctuation trimmed.
///
/// The boundary rule mirrors common autolinkers: a match is only accepted
/// when the preceding byte (if any) is not an ASCII alphanumeric — so
/// `"see http://x"` matches but `"shttp://x"` doesn't. The URL extends to
/// the next whitespace, `<`, `>`, `"`, `'`, or backtick, and we then strip
/// trailing `.,;:!?` plus an unbalanced `)`, `]`, or `}`.
///
/// `index` skips ahead to the next byte that could start a pattern, so the
/// per-byte boundary and prefix checks below only run at real candidates
/// rather than across every byte of non-URL prose.
fn find_autolink_match(
    s: &str,
    from: usize,
    patterns: &[String],
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut base = from;
    while base < bytes.len() {
        let rel = index.next(&bytes[base..])?;
        let i = base + rel;
        // Word boundary: the previous byte must not be ASCII alphanumeric.
        let is_boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if is_boundary {
            for pat in patterns {
                let pat_bytes = pat.as_bytes();
                if pat_bytes.is_empty() {
                    continue;
                }
                if i + pat_bytes.len() <= bytes.len()
                    && bytes[i..i + pat_bytes.len()].eq_ignore_ascii_case(pat_bytes)
                {
                    let url_start = i;
                    let mut url_end = i + pat_bytes.len();
                    while url_end < bytes.len() && is_url_byte(bytes[url_end]) {
                        url_end += 1;
                    }
                    // Require at least one byte beyond the scheme/prefix
                    // so `"http://"` on its own isn't auto-linked.
                    if url_end == i + pat_bytes.len() {
                        continue;
                    }
                    url_end = trim_trailing_punct(bytes, url_start, url_end);
                    return Some((url_start, url_end));
                }
            }
        }
        base = i + 1;
    }
    None
}

#[inline]
fn is_url_byte(byte: u8) -> bool {
    !matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'"' | b'\'' | b'`')
}

fn trim_trailing_punct(bytes: &[u8], start: usize, mut end: usize) -> usize {
    while end > start {
        let b = bytes[end - 1];
        match b {
            b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
            b')' | b']' | b'}' => {
                let (open, close) = match b {
                    b')' => (b'(', b')'),
                    b']' => (b'[', b']'),
                    _ => (b'{', b'}'),
                };
                // Strip the closing bracket only when it has no unmatched
                // partner inside the URL — a single pass over the slice is
                // simpler than two `filter().count()` walks and avoids the
                // `naive_bytecount` clippy lint.
                let mut opens = 0usize;
                let mut closes = 0usize;
                for &x in &bytes[start..end - 1] {
                    if x == open {
                        opens += 1;
                    } else if x == close {
                        closes += 1;
                    }
                }
                if closes >= opens {
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}

fn is_html_attr_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

fn is_html_attr_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn html_attr_value_range(html: &str, bytes: &[u8], name_end: usize) -> Option<(usize, usize)> {
    let mut cursor = name_end;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'=') {
        return None;
    }
    cursor += 1;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if let quote @ (b'"' | b'\'') = bytes.get(cursor).copied()? {
        let value_start = cursor + 1;
        let value_end = html[value_start..]
            .bytes()
            .position(|byte| byte == quote)
            .map(|offset| value_start + offset)?;
        Some((value_start, value_end))
    } else {
        let value_start = cursor;
        let mut value_end = value_start;
        while value_end < bytes.len()
            && !bytes[value_end].is_ascii_whitespace()
            && bytes[value_end] != b'>'
        {
            value_end += 1;
        }
        Some((value_start, value_end))
    }
}

fn collect_heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    collect_heading_text_into(nodes, &mut text);
    text
}

fn collect_heading_text_into(nodes: &[Node<'_>], text: &mut String) {
    for node in nodes {
        collect_node_text(node, text);
    }
}

fn collect_node_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Strong(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Delete(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Link(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        _ => {}
    }
}

fn slugify_heading(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    slugify_heading_into(text, &mut out);
    out
}

/// Slugify `text` into `out`. `out` is **not** cleared by this function —
/// callers should clear it themselves so they can reuse a long-lived
/// scratch buffer across many headings without giving up the allocation.
fn slugify_heading_into(text: &str, out: &mut String) {
    // Single-pass slugify. Hot path is the all-ASCII byte loop (no
    // UTF-8 decode, no `char::to_lowercase` iterator allocation per
    // character); we fall back to the char iterator only when a
    // non-ASCII byte appears.
    let bytes = text.as_bytes();
    out.reserve(text.len());
    let start_len = out.len();
    let mut last_was_separator = true;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            if b.is_ascii_alphanumeric() {
                // Lowercase ASCII letters with a branchless add.
                let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
                out.push(lower as char);
                last_was_separator = false;
            } else if !last_was_separator {
                out.push('-');
                last_was_separator = true;
            }
            i += 1;
        } else {
            // Find the next ASCII boundary and process the multi-byte run
            // through the char iterator (handles Unicode case folding /
            // alphanumeric classification correctly).
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] >= 0x80 {
                j += 1;
            }
            for ch in text[i..j].chars() {
                for lower in ch.to_lowercase() {
                    if lower.is_alphanumeric() {
                        out.push(lower);
                        last_was_separator = false;
                    } else if !last_was_separator {
                        out.push('-');
                        last_was_separator = true;
                    }
                }
            }
            i = j;
        }
    }

    while out.len() > start_len && out.ends_with('-') {
        out.pop();
    }

    if out.len() == start_len {
        out.push_str("section");
    }
}

#[derive(Debug, Clone)]
struct InlineTocEntry {
    depth: u8,
    text: String,
    id: String,
}

fn collect_inline_toc_entries(
    document: &Document<'_>,
    max_depth: u8,
    entries: &mut Vec<InlineTocEntry>,
) {
    let mut counts = FxHashMap::default();

    for node in &document.children {
        collect_inline_toc_node(node, max_depth, &mut counts, entries);
    }
}

/// Cheap, allocation-free scan for a standalone `[[toc]]` paragraph. The
/// directive is recognized only when the paragraph contains a single text
/// node whose trimmed value is exactly `[[toc]]` — which is also the only
/// form `visit_paragraph` will render as a TOC.
fn document_has_toc_marker(document: &Document<'_>) -> bool {
    document.children.iter().any(node_has_toc_marker)
}

fn node_has_toc_marker(node: &Node<'_>) -> bool {
    match node {
        Node::Paragraph(p) => is_toc_marker_paragraph(p),
        Node::BlockQuote(bq) => bq.children.iter().any(node_has_toc_marker),
        Node::List(list) => list.children.iter().any(list_item_has_toc_marker),
        Node::ListItem(item) => list_item_has_toc_marker(item),
        Node::FootnoteDefinition(def) => def.children.iter().any(node_has_toc_marker),
        _ => false,
    }
}

fn list_item_has_toc_marker(item: &ListItem<'_>) -> bool {
    item.children.iter().any(node_has_toc_marker)
}

fn is_toc_marker_paragraph(paragraph: &Paragraph<'_>) -> bool {
    // Equivalent to the prior
    // `collect_text_nodes_only(...).is_some_and(|t| t.trim() == "[[toc]]")`
    // check, but allocation-free: bails on the first non-Text child and
    // matches the marker byte-by-byte against the concatenated text. Note
    // that the inline parser emits the literal "[[toc]]" as three Text
    // nodes (`[`, `[`, `toc]]`) because the bracket-as-link path fails
    // open — so a "single Text child only" shortcut would miss it.
    const MARKER: &[u8] = b"[[toc]]";
    let mut matched = 0usize;
    let mut after_marker_ws = false;

    for child in &paragraph.children {
        let Node::Text(text) = child else {
            return false;
        };
        for &byte in text.value.as_bytes() {
            let is_ws = matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
            if is_ws {
                if matched > 0 {
                    after_marker_ws = true;
                }
                continue;
            }
            if after_marker_ws || matched == MARKER.len() || byte != MARKER[matched] {
                return false;
            }
            matched += 1;
        }
    }

    matched == MARKER.len()
}

fn collect_inline_toc_node(
    node: &Node<'_>,
    max_depth: u8,
    counts: &mut FxHashMap<String, usize>,
    entries: &mut Vec<InlineTocEntry>,
) {
    use std::fmt::Write as _;

    match node {
        Node::Heading(heading) => {
            let include_heading = heading.depth <= max_depth;
            let text = collect_heading_text(&heading.children);
            let mut slug = slugify_heading(&text);
            let id = if let Some(count) = counts.get_mut(slug.as_str()) {
                let suffix = *count;
                *count += 1;
                if include_heading {
                    let _ = write!(slug, "-{suffix}");
                    Some(slug)
                } else {
                    None
                }
            } else if include_heading {
                counts.insert(slug.clone(), 1);
                Some(slug)
            } else {
                counts.insert(slug, 1);
                None
            };

            if let Some(id) = id {
                entries.push(InlineTocEntry { depth: heading.depth, text, id });
            }
        }
        Node::BlockQuote(block_quote) => {
            for child in &block_quote.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        Node::List(list) => {
            for item in &list.children {
                for child in &item.children {
                    collect_inline_toc_node(child, max_depth, counts, entries);
                }
            }
        }
        Node::ListItem(item) => {
            for child in &item.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        Node::FootnoteDefinition(definition) => {
            for child in &definition.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        _ => {}
    }
}

/// HTML renderer.
pub struct HtmlRenderer {
    options: HtmlRendererOptions,
    output: String,
    heading_id_counts: FxHashMap<String, usize>,
    toc_entries: Vec<InlineTocEntry>,
    /// Whether the document being rendered contains at least one
    /// `[[toc]]` directive paragraph. Cached at `render()` entry so each
    /// `visit_paragraph` can skip the marker check entirely when no
    /// directive exists (the common case). Kept separate from
    /// `toc_entries.is_empty()` because a document may have a marker
    /// AND zero entries (no headings, or all filtered by `toc_max_depth`)
    /// — in that case we still need to suppress the literal `[[toc]]`
    /// text from the output.
    document_has_toc_marker: bool,
    /// Reusable scratch buffer for the raw concatenated heading text in
    /// `heading_id`. A long-lived buffer avoids paying for a fresh
    /// `String` allocation per heading — `slugify_heading` previously
    /// allocated one `text` String per call.
    heading_text_scratch: String,
    /// Reusable scratch buffer for the slugified id. The final id String
    /// that ends up in `heading_id_counts` is cloned out of here on
    /// vacant inserts; the buffer itself stays around across renders.
    heading_slug_scratch: String,
    /// Suppresses URL auto-linking while we're already inside an `<a>` so
    /// the builtin can't nest anchors. Tracked manually rather than via
    /// the AST because `visit_text` can be reached through many parents
    /// (paragraphs, headings, emphasis, …) and only the link case needs
    /// to mask it out.
    in_link: bool,
    /// First-byte skip index for the autolink scanner. It depends only on
    /// `options.autolink_patterns`, which is immutable for the duration of a
    /// render, so it is built once at `render()` entry and reused for every
    /// text node instead of being rebuilt per node (the prior behaviour zeroed
    /// and filled a 256-byte table on the hottest inline path). `None` when
    /// autolinking is disabled or there are no patterns.
    autolink_index: Option<FirstByteIndex>,
}

impl HtmlRenderer {
    /// Creates a new HTML renderer with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(HtmlRendererOptions::new())
    }

    /// Creates a new HTML renderer with the specified options.
    #[must_use]
    pub fn with_options(options: HtmlRendererOptions) -> Self {
        Self {
            options,
            output: String::new(),
            heading_id_counts: FxHashMap::default(),
            toc_entries: Vec::new(),
            document_has_toc_marker: false,
            // Pre-size the heading scratch buffers: a typical heading text
            // is well under 64 chars. Pre-allocating spares the first
            // heading from a `String::with_capacity(0)` → `reserve(N)`
            // round-trip without meaningful memory cost (these buffers
            // live for the renderer's lifetime regardless).
            heading_text_scratch: String::with_capacity(64),
            heading_slug_scratch: String::with_capacity(64),
            in_link: false,
            autolink_index: None,
        }
    }

    /// Renders a document to HTML string.
    #[must_use]
    pub fn render(&mut self, document: &Document<'_>) -> String {
        profile_span!("renderer::render");
        self.output.clear();
        // TOC collection walks every heading and allocates a slug per entry,
        // which used to fire on every render regardless of whether a
        // `[[toc]]` directive was actually present. Pre-scan the document
        // cheaply (no allocations) and skip the work when no marker exists —
        // this is the common case for normal docs.
        self.toc_entries.clear();
        self.document_has_toc_marker = document_has_toc_marker(document);
        if self.document_has_toc_marker {
            collect_inline_toc_entries(document, self.options.toc_max_depth, &mut self.toc_entries);
        }
        self.heading_id_counts.clear();
        // Build the autolink first-byte index once per render (it depends only
        // on the immutable pattern list) so `write_text_with_autolinks` can
        // reuse it instead of rebuilding a 256-byte table per text node.
        self.autolink_index =
            if self.options.autolink_urls && !self.options.autolink_patterns.is_empty() {
                Some(FirstByteIndex::from_patterns(&self.options.autolink_patterns))
            } else {
                None
            };
        // HTML output is typically 2×–3× the markdown source (every
        // `**bold**` becomes `<strong>...</strong>` etc.) so the prior
        // 1.5× estimate kept undersizing the buffer and forcing 1–2
        // power-of-two reallocs per render on docs >32 KB. 2× hits the
        // realistic mean for the bundled corpora (rust-book / vite /
        // vue / typescript-handbook all land between 1.8× and 2.6×).
        let estimated_len = (document.span.len() as usize).saturating_mul(2);
        if self.output.capacity() < estimated_len {
            self.output.reserve(estimated_len - self.output.capacity());
        }
        self.visit_document(document);
        std::mem::take(&mut self.output)
    }

    fn render_inline_toc(&mut self) {
        use std::fmt::Write as _;

        if self.toc_entries.is_empty() {
            return;
        }

        self.write("<nav class=\"ox-toc\" aria-label=\"Table of contents\">\n<ul>\n");
        for entry in &self.toc_entries {
            self.output.push_str("<li class=\"ox-toc__item ox-toc__item--depth-");
            let _ = write!(self.output, "{}", entry.depth);
            self.output.push_str("\"><a href=\"#");
            write_url_escaped_into(&mut self.output, &entry.id);
            self.output.push_str("\">");
            write_escaped_into(&mut self.output, &entry.text);
            self.output.push_str("</a></li>\n");
        }
        self.write("</ul>\n</nav>\n");
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_display(&mut self, value: impl Display) {
        write!(self.output, "{value}").expect("writing to String should not fail");
    }

    fn write_escaped(&mut self, s: &str) {
        profile_span!("renderer::write_escaped");
        write_escaped_into(&mut self.output, s);
    }

    /// Walks `s` and emits an `<a>` tag for each registered URL pattern
    /// match, escaping the non-URL spans the normal way. Caller is expected
    /// to have already gated this on the autolink flag — the function does
    /// no flag checks of its own.
    fn write_text_with_autolinks(&mut self, s: &str) {
        profile_span!("renderer::write_text_with_autolinks");
        let bytes = s.as_bytes();
        // Reuse the per-render first-byte index (see `autolink_index`). If it's
        // absent the caller's gating slipped — fall back to emitting the text
        // verbatim rather than rebuilding the index here.
        let Some(index) = self.autolink_index.as_ref() else {
            write_escaped_into(&mut self.output, s);
            return;
        };
        // Borrow the relevant fields disjointly so the URL scan (which only
        // reads `options`/`autolink_index`) and the output writes can coexist.
        let patterns = &self.options.autolink_patterns;
        let target_blank = self.options.autolink_target_blank;
        let out = &mut self.output;
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let Some((match_start, url_end)) = find_autolink_match(s, cursor, patterns, index)
            else {
                break;
            };
            // Emit the literal text preceding the URL.
            if match_start > cursor {
                write_escaped_into(out, &s[cursor..match_start]);
            }
            let url = &s[match_start..url_end];
            out.push_str("<a href=\"");
            write_url_escaped_into(out, url);
            out.push('"');
            if target_blank {
                out.push_str(" target=\"_blank\" rel=\"noopener noreferrer\"");
            }
            out.push('>');
            // The visible text is the URL itself; escape it like any text.
            write_escaped_into(out, url);
            out.push_str("</a>");
            cursor = url_end;
        }
        if cursor < bytes.len() {
            write_escaped_into(out, &s[cursor..]);
        }
    }

    fn write_url_escaped(&mut self, s: &str) {
        write_url_escaped_into(&mut self.output, s);
    }

    fn sanitized_url<'a>(&self, url: &'a str, fallback: &'static str) -> &'a str {
        if !self.options.sanitize {
            return url;
        }

        let trimmed =
            url.trim_matches(|ch: char| ch.is_ascii_control() || ch.is_ascii_whitespace());

        if Self::is_safe_url(trimmed) {
            trimmed
        } else {
            fallback
        }
    }

    fn is_safe_url(url: &str) -> bool {
        if url.bytes().any(|byte| byte.is_ascii_control()) {
            return false;
        }

        let Some(colon_index) = url.find(':') else {
            return true;
        };

        let first_path_marker = url.find(&['/', '?', '#'][..]).unwrap_or(usize::MAX);
        if first_path_marker < colon_index {
            return true;
        }

        let scheme = url[..colon_index]
            .chars()
            .filter(|ch| !ch.is_ascii_whitespace())
            .map(|ch| ch.to_ascii_lowercase())
            .collect::<String>();

        matches!(scheme.as_str(), "http" | "https" | "mailto" | "tel")
    }

    fn render_paragraph_with_skipped_text_prefix<'a>(
        &self,
        paragraph: &Paragraph<'a>,
        mut skip_chars: usize,
    ) -> String {
        let mut renderer = HtmlRenderer::with_options(self.options.clone());

        for child in &paragraph.children {
            match child {
                Node::Text(text) if skip_chars > 0 => {
                    if skip_chars >= text.value.len() {
                        skip_chars -= text.value.len();
                        continue;
                    }

                    renderer.write_escaped(&text.value[skip_chars..]);
                    skip_chars = 0;
                }
                _ => renderer.visit_node(child),
            }
        }

        renderer.output
    }

    fn detect_callout<'a>(paragraph: &Paragraph<'a>) -> Option<(CalloutKind, usize)> {
        // Fast bail: a callout marker is `[!KIND]...` so the very first
        // text byte must be `[`. The previous version unconditionally
        // allocated a `String prefix` and pushed Text values into it
        // before checking — pure waste for the overwhelmingly common
        // case of a regular block quote.
        let mut iter = paragraph.children.iter();
        let Node::Text(first_text) = iter.next()? else {
            return None;
        };
        if first_text.value.as_bytes().first() != Some(&b'[') {
            return None;
        }

        // The first Text node almost always contains the entire marker
        // (parsers don't split `[!NOTE]` across multiple Text nodes
        // unless inline markup interleaves). Try in-place first, and
        // only fall back to the concatenating slow path if the marker
        // straddles nodes.
        if let Some((kind, remainder)) = CalloutKind::parse_marker(first_text.value) {
            let consumed = first_text.value.len().saturating_sub(remainder.len());
            return Some((kind, consumed));
        }

        let mut prefix = String::from(first_text.value);
        for child in iter {
            let Node::Text(text) = child else {
                return None;
            };
            prefix.push_str(text.value);
            if let Some((kind, remainder)) = CalloutKind::parse_marker(&prefix) {
                let consumed = prefix.len().saturating_sub(remainder.len());
                return Some((kind, consumed));
            }
        }

        None
    }

    fn render_callout_block_quote<'a>(&mut self, block_quote: &BlockQuote<'a>) -> bool {
        let Some(Node::Paragraph(first_paragraph)) = block_quote.children.first() else {
            return false;
        };
        let Some((kind, consumed_chars)) = Self::detect_callout(first_paragraph) else {
            return false;
        };

        self.write("<blockquote class=\"ox-callout ox-callout--");
        self.write(kind.class_name());
        self.write("\">\n");
        self.write("<p class=\"ox-callout-title\">");
        self.write(kind.label());
        self.write("</p>\n");

        let paragraph_body =
            self.render_paragraph_with_skipped_text_prefix(first_paragraph, consumed_chars);
        if !paragraph_body.trim().is_empty() {
            self.write("<p>");
            self.write(&paragraph_body);
            self.write("</p>\n");
        }

        for child in block_quote.children.iter().skip(1) {
            self.visit_node(child);
        }

        self.write("</blockquote>\n");
        true
    }

    fn build_code_block_state(&self, code_block: &CodeBlock<'_>) -> CodeBlockRenderState {
        let info = normalize_code_block_info(code_block.lang, code_block.meta);
        let syntax = self.options.code_annotation_syntax;
        let mut lines = if self.options.code_annotations && syntax.includes_vitepress() {
            parse_vitepress_inline_annotations(code_block.value)
        } else {
            code_block
                .value
                .split('\n')
                .map(|line| CodeLineRenderState {
                    value: line.to_string(),
                    annotations: Vec::new(),
                })
                .collect()
        };

        let mut title = None;
        let mut line_numbers_start = if self.options.code_annotations
            && syntax.includes_vitepress()
            && self.options.code_annotation_default_line_numbers
        {
            Some(1)
        } else {
            None
        };

        if self.options.code_annotations && !info.meta.is_empty() {
            if syntax.includes_attribute() {
                let annotations =
                    parse_code_annotations(&info.meta, &self.options.code_annotation_meta_key);
                apply_btree_annotations(&mut lines, &annotations);
            }

            if syntax.includes_vitepress() {
                for token in split_code_block_meta(&info.meta) {
                    match token.kind {
                        MetaTokenKind::Braces => {
                            let line_numbers = parse_line_numbers(token.value);
                            apply_annotation_numbers(
                                &mut lines,
                                &line_numbers,
                                CodeAnnotationKind::Highlight,
                            );
                        }
                        MetaTokenKind::Brackets => {
                            if title.is_none() && !token.value.trim().is_empty() {
                                title = Some(token.value.trim().to_string());
                            }
                        }
                        MetaTokenKind::Raw => {
                            if token.value == ":line-numbers" {
                                line_numbers_start = Some(1);
                            } else if let Some(start) =
                                token.value.strip_prefix(":line-numbers=").and_then(|value| {
                                    value
                                        .trim()
                                        .parse::<usize>()
                                        .ok()
                                        .filter(|line_number| *line_number > 0)
                                })
                            {
                                line_numbers_start = Some(start);
                            } else if token.value == ":no-line-numbers" {
                                line_numbers_start = None;
                            }
                        }
                    }
                }
            }
        }

        CodeBlockRenderState { language: info.language, title, line_numbers_start, lines }
    }

    fn write_code_lines(&mut self, state: &CodeBlockRenderState) {
        let has_focus = state.has_focus();

        for (index, line) in state.lines.iter().enumerate() {
            let line_number = index + 1;
            let mut class_names: Vec<&str> = vec!["line", "ox-code-line"];

            for annotation in &line.annotations {
                let class_name = annotation.class_name();
                if !class_names.contains(&class_name) {
                    class_names.push(class_name);
                }
                for extra_class_name in annotation.extra_class_names() {
                    if !class_names.contains(extra_class_name) {
                        class_names.push(extra_class_name);
                    }
                }
            }

            if has_focus && !line.annotations.contains(&CodeAnnotationKind::Focus) {
                class_names.push("ox-code-line--dimmed");
            }

            self.write("<span class=\"");
            self.write(&class_names.join(" "));
            self.write("\" data-line=\"");
            self.write_display(line_number);
            self.write("\"");

            if let Some(start) = state.line_numbers_start {
                self.write(" data-line-number=\"");
                self.write_display(start + index);
                self.write("\"");
            }

            self.write(">");
            self.write_escaped(&line.value);
            self.write("</span>");

            if index + 1 < state.lines.len() {
                self.write("\n");
            }
        }
    }

    fn write_html_value(&mut self, value: &str) {
        if self.options.sanitize {
            self.write_escaped(value);
        } else if self.options.convert_md_links {
            let rewritten = self.rewrite_html_root_urls(value);
            self.write(&rewritten);
        } else {
            self.write(value);
        }
    }

    fn visit_inline_node(&mut self, node: &Node<'_>) {
        // Text is the overwhelmingly common child of paragraphs / headings
        // / links / emphasis / strong, etc. — on the bundled corpora it
        // accounts for roughly 60-70% of inline visits. Inlining the
        // write here skips the trait's 20-arm `walk_node` match and the
        // `visit_text` wrapper, both of which are the only thing
        // `visit_text` would do anyway (escape into `self.output`).
        match node {
            Node::Text(text) => {
                // The autolink builtin lives on this hot path too: when
                // the flag is on (and we're not already inside an `<a>`)
                // we have to scan the text for URLs before escaping. The
                // common case — flag off — collapses back to the original
                // single `write_escaped_into` call thanks to the early
                // boolean check.
                // `autolink_index` is `Some` iff `autolink_urls` and a non-empty
                // pattern list (computed once at `render()` entry), so this one
                // Option check replaces the three field reads.
                if self.autolink_index.is_some() && !self.in_link {
                    self.write_text_with_autolinks(text.value);
                } else {
                    write_escaped_into(&mut self.output, text.value);
                }
            }
            Node::Html(html) => self.write_html_value(html.value),
            _ => self.visit_node(node),
        }
    }

    /// Writes the heading's slugified id directly into `self.output`.
    /// Avoids allocating a return `String`: the unique-heading hot path
    /// now pays for exactly one `String` allocation (the slug clone that
    /// becomes the map key) and the duplicate-heading path pays for
    /// zero, since the `-N` suffix is written directly via `write!`.
    fn write_heading_id(&mut self, heading: &Heading<'_>) {
        use std::fmt::Write as _;

        self.heading_text_scratch.clear();
        collect_heading_text_into(&heading.children, &mut self.heading_text_scratch);
        self.heading_slug_scratch.clear();
        slugify_heading_into(&self.heading_text_scratch, &mut self.heading_slug_scratch);

        // Cheap lookup first — avoids cloning the slug on the duplicate
        // path. The `entry()` API would force us to materialize an owned
        // key up front, defeating the point.
        if let Some(count) = self.heading_id_counts.get_mut(self.heading_slug_scratch.as_str()) {
            let n = *count;
            *count += 1;
            self.output.push_str(&self.heading_slug_scratch);
            // `write!` into `String` is infallible; the formatter pushes
            // bytes directly into the existing buffer with no `format!`
            // intermediate allocation.
            let _ = write!(self.output, "-{n}");
            return;
        }

        self.output.push_str(&self.heading_slug_scratch);
        let key = self.heading_slug_scratch.clone();
        self.heading_id_counts.insert(key, 1);
    }

    fn convert_markdown_url(&self, url: &str) -> Option<String> {
        if let Some(converted) = self.convert_md_url(url) {
            return Some(converted);
        }

        self.apply_base_to_root_absolute_url(url)
    }

    fn apply_base_to_root_absolute_url(&self, url: &str) -> Option<String> {
        if !self.options.convert_md_links || !url.starts_with('/') || url.starts_with("//") {
            return None;
        }

        let suffix_start = url.find(&['?', '#'][..]).unwrap_or(url.len());
        let (path, suffix) = url.split_at(suffix_start);
        let base = self.options.base_url.trim_end_matches('/');

        if base.is_empty() {
            None
        } else if path == "/" {
            Some(format!("{base}/{suffix}"))
        } else {
            Some(format!("{base}{path}{suffix}"))
        }
    }

    fn rewrite_html_root_urls(&self, html: &str) -> String {
        let mut output = String::with_capacity(html.len());
        let bytes = html.as_bytes();
        let mut i = 0;
        let mut in_tag = false;

        while i < bytes.len() {
            match bytes[i] {
                b'<' => {
                    in_tag = true;
                    output.push('<');
                    i += 1;
                }
                b'>' => {
                    in_tag = false;
                    output.push('>');
                    i += 1;
                }
                byte if in_tag && is_html_attr_start(byte) => {
                    let name_start = i;
                    let mut name_end = i + 1;
                    while name_end < bytes.len() && is_html_attr_char(bytes[name_end]) {
                        name_end += 1;
                    }

                    let name = &html[name_start..name_end];
                    if name.eq_ignore_ascii_case("href") || name.eq_ignore_ascii_case("src") {
                        let Some((value_start, value_end)) =
                            html_attr_value_range(html, bytes, name_end)
                        else {
                            output.push_str(name);
                            i = name_end;
                            continue;
                        };
                        let value = &html[value_start..value_end];
                        if let Some(rewritten) = self.apply_base_to_root_absolute_url(value) {
                            output.push_str(&html[i..value_start]);
                            output.push_str(&rewritten);
                            i = value_end;
                            continue;
                        }
                    }

                    output.push_str(name);
                    i = name_end;
                }
                _ => {
                    if let Some(ch) = html[i..].chars().next() {
                        output.push(ch);
                        i += ch.len_utf8();
                    } else {
                        break;
                    }
                }
            }
        }

        output
    }

    /// Converts a Markdown URL to an `.html` URL for SSG output.
    fn convert_md_url(&self, url: &str) -> Option<String> {
        // Split URL into path and fragment
        let (path, fragment) = match url.split_once('#') {
            Some((p, f)) => (p, Some(f)),
            None => (url, None),
        };

        let markdown_extension =
            std::path::Path::new(path).extension().and_then(|ext| ext.to_str()).filter(|ext| {
                ext.eq_ignore_ascii_case("md")
                    || ext.eq_ignore_ascii_case("mdx")
                    || ext.eq_ignore_ascii_case("markdown")
            });

        let markdown_extension = markdown_extension?;

        if !self.options.convert_md_links {
            return None;
        }

        // Remove the Markdown extension, including the leading dot.
        let path_without_ext = &path[..path.len() - markdown_extension.len() - 1];

        // Check if the source file is an index file
        // index.md stays at the directory level, so relative paths work differently
        let source_is_index = self.is_source_index();

        // Convert path
        let converted = if path.starts_with('/') {
            // Absolute path: /getting-started.md -> {base}getting-started/index.html
            let path_without_slash = &path_without_ext[1..];
            let base = &self.options.base_url;
            if path_without_slash.is_empty() || path_without_slash == "index" {
                format!("{base}index.html")
            } else {
                format!("{base}{path_without_slash}/index.html")
            }
        } else if path.starts_with("./") {
            // Same-directory relative path
            let name = &path_without_ext[2..]; // Remove "./"
            if name == "index" {
                // ./index.md -> ./index.html (stay in same directory)
                "./index.html".to_string()
            } else if source_is_index {
                // Source is index.md, so we're at directory level
                // ./types.md -> ./types/index.html
                format!("./{name}/index.html")
            } else {
                // Source is not index.md (e.g., types.md -> types/index.html)
                // So we need to go up one level
                // ./types.md -> ../types/index.html
                format!("../{name}/index.html")
            }
        } else if path.starts_with("../") {
            // Parent-relative path
            let rest = &path_without_ext[3..]; // Remove "../"
            if source_is_index {
                // Source is index.md at directory level
                // ../types.md -> ../types/index.html
                if rest == "index" || rest.ends_with("/index") {
                    let dir = rest.trim_end_matches("/index").trim_end_matches("index");
                    if dir.is_empty() {
                        "../index.html".to_string()
                    } else {
                        format!("../{dir}/index.html")
                    }
                } else {
                    format!("../{rest}/index.html")
                }
            } else {
                // Source is not index.md, need extra ../
                // ../types.md -> ../../types/index.html
                if rest == "index" || rest.ends_with("/index") {
                    let dir = rest.trim_end_matches("/index").trim_end_matches("index");
                    if dir.is_empty() {
                        "../../index.html".to_string()
                    } else {
                        format!("../../{dir}/index.html")
                    }
                } else {
                    format!("../../{rest}/index.html")
                }
            }
        } else {
            // Plain relative path: types.md
            if path_without_ext == "index" || path_without_ext.ends_with("/index") {
                let dir = path_without_ext.trim_end_matches("/index").trim_end_matches("index");
                if dir.is_empty() {
                    "./index.html".to_string()
                } else if source_is_index {
                    format!("./{dir}/index.html")
                } else {
                    format!("../{dir}/index.html")
                }
            } else if source_is_index {
                // Source is index.md
                // types.md -> ./types/index.html
                format!("./{path_without_ext}/index.html")
            } else {
                // Source is not index.md
                // types.md -> ../types/index.html
                format!("../{path_without_ext}/index.html")
            }
        };

        // Reattach fragment if present
        Some(match fragment {
            Some(f) => format!("{converted}#{f}"),
            None => converted,
        })
    }

    /// Checks if the source file is an index file (index.md).
    fn is_source_index(&self) -> bool {
        if self.options.source_path.is_empty() {
            return false;
        }
        let source = std::path::Path::new(&self.options.source_path);
        source.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("index"))
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HtmlRenderer {
    type Output = String;

    fn render(&mut self, document: &Document<'_>) -> RenderResult<Self::Output> {
        Ok(self.render(document))
    }
}

impl<'a> Visit<'a> for HtmlRenderer {
    fn visit_paragraph(&mut self, paragraph: &Paragraph<'a>) {
        profile_span!("renderer::visit_paragraph");
        // Skip the `[[toc]]` byte scan entirely when the document has no
        // marker — pure overhead in the common case. When a marker IS
        // present we must run the check on every paragraph and suppress
        // the matching one, even if `toc_entries` is empty (e.g. document
        // has no headings or all are filtered by `toc_max_depth`).
        // Otherwise the literal `[[toc]]` would leak into the output.
        if self.document_has_toc_marker && is_toc_marker_paragraph(paragraph) {
            self.render_inline_toc();
            return;
        }

        self.output.push_str("<p>");
        for child in &paragraph.children {
            self.visit_inline_node(child);
        }
        self.output.push_str("</p>\n");
    }

    fn visit_heading(&mut self, heading: &Heading<'a>) {
        profile_span!("renderer::visit_heading");
        // Avoid the heading.depth -> &str match per call: heading depth is
        // 1..=6 by construction, and "h%d" is a fixed shape we can splat
        // directly. Saves a branch and a `write` call.
        let depth = heading.depth.clamp(1, 6);
        self.output.push_str("<h");
        self.output.push((b'0' + depth) as char);
        self.output.push_str(" id=\"");
        // Heading ids are slugified: lowercase alnum + '-' separators. None
        // of those bytes need HTML escaping, so the unconditional
        // `write_escaped` pass over the id was pure overhead. We also
        // skip materializing the id as a return-value `String`; it's
        // written straight into `self.output`.
        self.write_heading_id(heading);
        self.output.push_str("\">");
        for child in &heading.children {
            self.visit_inline_node(child);
        }
        self.output.push_str("</h");
        self.output.push((b'0' + depth) as char);
        self.output.push_str(">\n");
    }

    fn visit_thematic_break(&mut self, _thematic_break: &ThematicBreak) {
        if self.options.xhtml {
            self.write("<hr />\n");
        } else {
            self.write("<hr>\n");
        }
    }

    fn visit_block_quote(&mut self, block_quote: &BlockQuote<'a>) {
        profile_span!("renderer::visit_block_quote");
        if self.render_callout_block_quote(block_quote) {
            return;
        }

        self.write("<blockquote>\n");
        for child in &block_quote.children {
            self.visit_node(child);
        }
        self.write("</blockquote>\n");
    }

    fn visit_list(&mut self, list: &List<'a>) {
        profile_span!("renderer::visit_list");
        if list.ordered {
            if let Some(start) = list.start {
                if start != 1 {
                    self.write("<ol start=\"");
                    self.write_display(start);
                    self.write("\">\n");
                } else {
                    self.write("<ol>\n");
                }
            } else {
                self.write("<ol>\n");
            }
        } else {
            self.write("<ul>\n");
        }

        for child in &list.children {
            self.visit_list_item(child);
        }

        if list.ordered {
            self.write("</ol>\n");
        } else {
            self.write("</ul>\n");
        }
    }

    fn visit_list_item(&mut self, list_item: &ListItem<'a>) {
        self.write("<li>");

        if let Some(checked) = list_item.checked {
            if checked {
                self.write("<input type=\"checkbox\" checked disabled> ");
            } else {
                self.write("<input type=\"checkbox\" disabled> ");
            }
        }

        for child in &list_item.children {
            self.visit_node(child);
        }

        self.write("</li>\n");
    }

    fn visit_code_block(&mut self, code_block: &CodeBlock<'a>) {
        profile_span!("renderer::visit_code_block");
        if !self.options.code_annotations {
            self.write("<pre><code");
            if let Some(lang) = normalize_code_block_language(code_block.lang) {
                self.write(" class=\"language-");
                self.write_escaped(lang);
                self.write("\"");
            }
            self.write(">");
            self.write_escaped(code_block.value);
            self.write("</code></pre>\n");
            return;
        }

        let state = self.build_code_block_state(code_block);
        let block_classes = state.block_classes();

        self.write("<pre");
        if !block_classes.is_empty() {
            self.write(" class=\"");
            self.write(&block_classes.join(" "));
            self.write("\"");
        }
        if let Some(title) = state.title.as_deref() {
            self.write(" data-code-title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if let Some(start) = state.line_numbers_start {
            self.write(" data-line-numbers=\"true\" data-line-number-start=\"");
            self.write_display(start);
            self.write("\"");
        }
        self.write("><code");
        if let Some(lang) = state.language.as_deref() {
            self.write(" class=\"language-");
            self.write_escaped(lang);
            self.write("\"");
        }
        self.write(">");
        if state.needs_line_wrappers() {
            self.write_code_lines(&state);
        } else {
            self.write_escaped(code_block.value);
        }
        self.write("</code></pre>\n");
    }

    fn visit_html(&mut self, html: &Html<'a>) {
        self.write_html_value(html.value);
        self.write("\n");
    }

    fn visit_table(&mut self, table: &Table<'a>) {
        profile_span!("renderer::visit_table");
        self.write("<table>\n");
        for (i, row) in table.children.iter().enumerate() {
            if i == 0 {
                self.write("<thead>\n");
            } else if i == 1 {
                self.write("<tbody>\n");
            }
            self.visit_table_row_with_header(row, i == 0, &table.align);
            if i == 0 {
                self.write("</thead>\n");
            }
        }
        if !table.children.is_empty() {
            self.write("</tbody>\n");
        }
        self.write("</table>\n");
    }

    fn visit_text(&mut self, text: &Text<'a>) {
        profile_span!("renderer::visit_text");
        // See the matching gate in `visit_inline_node`: the cached
        // `autolink_index` already encodes `autolink_urls && !patterns.is_empty()`.
        if self.autolink_index.is_some() && !self.in_link {
            self.write_text_with_autolinks(text.value);
        } else {
            self.write_escaped(text.value);
        }
    }

    fn visit_emphasis(&mut self, emphasis: &Emphasis<'a>) {
        self.write("<em>");
        for child in &emphasis.children {
            self.visit_inline_node(child);
        }
        self.write("</em>");
    }

    fn visit_strong(&mut self, strong: &Strong<'a>) {
        self.write("<strong>");
        for child in &strong.children {
            self.visit_inline_node(child);
        }
        self.write("</strong>");
    }

    fn visit_inline_code(&mut self, inline_code: &InlineCode<'a>) {
        self.write("<code>");
        self.write_escaped(inline_code.value);
        self.write("</code>");
    }

    fn visit_break(&mut self, _break_node: &Break) {
        self.output.push_str(self.options.hard_break.as_str());
    }

    fn visit_link(&mut self, link: &Link<'a>) {
        self.write("<a href=\"");
        let converted_url =
            if self.options.convert_md_links { self.convert_markdown_url(link.url) } else { None };
        let href = self.sanitized_url(converted_url.as_deref().unwrap_or(link.url), "#");
        self.write_url_escaped(href);
        self.write("\"");
        // Add target="_blank" for external links (http:// or https://)
        if href.starts_with("http://") || href.starts_with("https://") {
            self.write(" target=\"_blank\" rel=\"noopener noreferrer\"");
        }
        if let Some(title) = link.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        self.write(">");
        // Suppress URL auto-linking inside the anchor — children text nodes
        // may contain literal URLs that we must not wrap in a nested <a>.
        let prev_in_link = self.in_link;
        self.in_link = true;
        for child in &link.children {
            self.visit_inline_node(child);
        }
        self.in_link = prev_in_link;
        self.write("</a>");
    }

    fn visit_image(&mut self, image: &Image<'a>) {
        self.write("<img src=\"");
        let converted_url =
            if self.options.convert_md_links { self.convert_markdown_url(image.url) } else { None };
        let src = self.sanitized_url(converted_url.as_deref().unwrap_or(image.url), "");
        self.write_url_escaped(src);
        self.write("\" alt=\"");
        self.write_escaped(image.alt);
        self.write("\"");
        if let Some(title) = image.title {
            self.write(" title=\"");
            self.write_escaped(title);
            self.write("\"");
        }
        if self.options.xhtml {
            self.write(" />");
        } else {
            self.write(">");
        }
    }

    fn visit_delete(&mut self, delete: &Delete<'a>) {
        self.write("<del>");
        for child in &delete.children {
            self.visit_inline_node(child);
        }
        self.write("</del>");
    }

    fn visit_footnote_reference(&mut self, footnote_ref: &FootnoteReference<'a>) {
        self.write("<sup><a href=\"#fn-");
        self.write_escaped(footnote_ref.identifier);
        self.write("\" id=\"fnref-");
        self.write_escaped(footnote_ref.identifier);
        self.write("\">");
        self.write_escaped(footnote_ref.identifier);
        self.write("</a></sup>");
    }

    fn visit_definition(&mut self, _definition: &Definition<'a>) {
        // Definitions are not rendered directly
    }

    fn visit_footnote_definition(&mut self, footnote_def: &FootnoteDefinition<'a>) {
        self.write("<div id=\"fn-");
        self.write_escaped(footnote_def.identifier);
        self.write("\" class=\"footnote\">\n");
        for child in &footnote_def.children {
            self.visit_node(child);
        }
        self.write("<a href=\"#fnref-");
        self.write_escaped(footnote_def.identifier);
        self.write("\">↩</a>\n</div>\n");
    }
}

impl HtmlRenderer {
    fn visit_table_row_with_header(
        &mut self,
        row: &TableRow<'_>,
        is_header: bool,
        align: &ox_content_allocator::Vec<'_, ox_content_ast::AlignKind>,
    ) {
        self.write("<tr>\n");
        let tag = if is_header { "th" } else { "td" };
        for (idx, cell) in row.children.iter().enumerate() {
            self.write("<");
            self.write(tag);
            match align.get(idx).copied().unwrap_or(ox_content_ast::AlignKind::None) {
                ox_content_ast::AlignKind::Left => self.write(" align=\"left\""),
                ox_content_ast::AlignKind::Center => self.write(" align=\"center\""),
                ox_content_ast::AlignKind::Right => self.write(" align=\"right\""),
                ox_content_ast::AlignKind::None => {}
            }
            self.write(">");
            self.visit_table_cell(cell);
            self.write("</");
            self.write(tag);
            self.write(">\n");
        }
        self.write("</tr>\n");
    }

    fn visit_table_cell(&mut self, cell: &TableCell<'_>) {
        for child in &cell.children {
            self.visit_inline_node(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ox_content_allocator::Allocator;
    use ox_content_parser::Parser;

    #[test]
    fn test_render_paragraph() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "Hello world").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<p>Hello world</p>\n");
    }

    #[test]
    fn test_render_heading() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "# Hello").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<h1 id=\"hello\">Hello</h1>\n");
    }

    #[test]
    fn test_render_heading_ids_are_unique_and_unicode() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "## はじめに\n## はじめに").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<h2 id=\"はじめに\">はじめに</h2>"));
        assert!(html.contains("<h2 id=\"はじめに-1\">はじめに</h2>"));
    }

    #[test]
    fn test_render_heading_id_uses_inline_text() {
        let allocator = Allocator::new();
        let doc =
            Parser::new(&allocator, "## **API** `Index` [Guide](./guide.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.starts_with("<h2 id=\"api-index-guide\">"));
    }

    #[test]
    fn test_render_inline_toc_directive() {
        let allocator = Allocator::new();
        let doc =
            Parser::new(&allocator, "# Title\n\n[[toc]]\n\n## Intro\n### API").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(html.contains("<nav class=\"ox-toc\" aria-label=\"Table of contents\">"));
        assert!(html.contains("<a href=\"#title\">Title</a>"));
        assert!(html.contains("<a href=\"#intro\">Intro</a>"));
        assert!(html.contains("<a href=\"#api\">API</a>"));
        assert!(!html.contains("<p>[[toc]]</p>"));
    }

    #[test]
    fn test_render_inline_toc_uses_unique_and_unicode_ids() {
        let allocator = Allocator::new();
        let doc =
            Parser::new(&allocator, "[[toc]]\n\n## Setup\n## Setup\n## はじめに").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(html.contains("href=\"#setup\""));
        assert!(html.contains("href=\"#setup-1\""));
        assert!(html.contains("href=\"#はじめに\""));
    }

    #[test]
    fn test_render_inline_toc_requires_standalone_text() {
        let allocator = Allocator::new();
        let doc =
            Parser::new(&allocator, "See [[toc]] here\n\n`[[toc]]`\n\n## Intro").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(html.contains("<p>See [[toc]] here</p>"));
        assert!(html.contains("<p><code>[[toc]]</code></p>"));
        assert!(!html.contains("ox-toc"));
    }

    #[test]
    fn test_render_inline_toc_marker_is_suppressed_when_no_headings() {
        // When the document contains `[[toc]]` but no headings (so
        // `toc_entries` is empty), the marker paragraph must still be
        // suppressed from output — otherwise the literal `[[toc]]`
        // leaks through as `<p>[[toc]]</p>`. Regression coverage for
        // the lazy-TOC optimization.
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[[toc]]").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(!html.contains("[[toc]]"), "marker leaked into output: {html}");
        assert!(!html.contains("<p>"), "expected no paragraph wrapper: {html}");
    }

    #[test]
    fn test_render_inline_toc_marker_is_suppressed_when_filtered_by_depth() {
        // `toc_max_depth: 0` filters every heading out, but the marker
        // paragraph should still be consumed so it doesn't leak.
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[[toc]]\n\n## Intro").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            toc_max_depth: 0,
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(!html.contains("[[toc]]"), "marker leaked: {html}");
        // The heading should still render as a heading (not as a TOC entry).
        assert!(html.contains("<h2"), "heading missing: {html}");
    }

    #[test]
    fn test_render_inline_toc_honors_max_depth() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[[toc]]\n\n# Title\n## Intro\n### API").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            toc_max_depth: 2,
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(html.contains("href=\"#title\""));
        assert!(html.contains("href=\"#intro\""));
        assert!(!html.contains("href=\"#api\""));
    }

    #[test]
    fn test_render_block_quote() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> Hello world").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<blockquote>\n<p>Hello world</p>\n</blockquote>\n");
    }

    #[test]
    fn test_render_block_quote_with_inline() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> **Note:** This is important").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("<strong>Note:</strong>"));
        assert!(html.contains("</blockquote>"));
    }

    #[test]
    fn test_render_github_style_important_callout() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> [!IMPORTANT]\n> This is important.").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(html.contains("<blockquote class=\"ox-callout ox-callout--important\">"));
        assert!(html.contains("<p class=\"ox-callout-title\">Important</p>"));
        assert!(html.contains("<p>This is important.</p>"));
        assert!(!html.contains("[!IMPORTANT]"));
    }

    #[test]
    fn test_render_github_style_callout_with_inline_content_after_marker() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "> [!NOTE] Supports **inline** content").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        assert!(html.contains("<blockquote class=\"ox-callout ox-callout--note\">"));
        assert!(html.contains("<p class=\"ox-callout-title\">Note</p>"));
        assert!(html.contains("<p>Supports <strong>inline</strong> content</p>"));
        assert!(!html.contains("[!NOTE]"));
    }

    #[test]
    fn test_render_code_block() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "```rust\nfn main() {}\n```").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
    }

    #[test]
    fn test_render_code_block_with_annotations() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "```ts file=main.ts annotate=\"highlight:1;warning:2;error:3\"\nconst ok = true;\nconst maybe = false;\nthrow new Error('boom');\n```",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            code_annotations: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(html.contains("class=\"ox-code-block ox-code-block--annotated has-highlighted\""));
        assert!(html.contains(
            "class=\"line ox-code-line ox-code-line--highlight highlighted\" data-line=\"1\""
        ));
        assert!(html.contains(
            "class=\"line ox-code-line ox-code-line--warning highlighted warning\" data-line=\"2\""
        ));
        assert!(html.contains(
            "class=\"line ox-code-line ox-code-line--error highlighted error\" data-line=\"3\""
        ));
        assert!(!html.contains("file=main.ts"));
    }

    #[test]
    fn test_render_code_block_with_custom_annotation_meta_key() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "```ts markers=\"warning:2\"\nconst ok = true;\nconst maybe = false;\n```",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            code_annotations: true,
            code_annotation_meta_key: "markers".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(html.contains("ox-code-block--annotated"));
        assert!(html.contains("ox-code-line--warning"));
    }

    #[test]
    fn test_render_code_block_with_vitepress_meta() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "```ts:line-numbers=2 {1,3} [config.ts]\nconst first = true;\nconst second = false;\nconst third = true;\n```",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            code_annotations: true,
            code_annotation_syntax: CodeAnnotationSyntax::VitePress,
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(html.contains("ox-code-block--annotated"));
        assert!(html.contains("ox-code-block--line-numbers"));
        assert!(html.contains("ox-code-block--with-title"));
        assert!(html.contains("line-numbers-mode"));
        assert!(html.contains("has-highlighted"));
        assert!(html.contains("data-code-title=\"config.ts\""));
        assert!(html.contains("data-line-number-start=\"2\""));
        assert!(html.contains("class=\"language-ts\""));
        assert!(html.contains("data-line-number=\"2\""));
        assert!(html.contains("data-line-number=\"4\""));
        assert!(html.contains("ox-code-line--highlight"));
    }

    #[test]
    fn test_render_code_block_with_vitepress_inline_directives() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "```ts\n// [!code focus:2]\nconst first = true;\nconst second = false;\nconsole.log('old value') // [!code --]\nconsole.log('new value') // [!code ++]\nconsole.warn('careful') // [!code warning]\nthrow new Error('boom') // [!code error]\n```",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            code_annotations: true,
            code_annotation_syntax: CodeAnnotationSyntax::VitePress,
            ..Default::default()
        });
        let html = renderer.render(&doc);

        assert!(!html.contains("[!code"));
        assert!(html.contains("has-focused"));
        assert!(html.contains("has-diff"));
        assert!(html.contains("ox-code-line--focus"));
        assert!(html.contains("ox-code-line--dimmed"));
        assert!(html.contains("ox-code-line--remove"));
        assert!(html.contains("ox-code-line--add"));
        assert!(html.contains("ox-code-line--warning"));
        assert!(html.contains("ox-code-line--error"));
        assert!(html.contains("console.log(&#39;old value&#39;)"));
        assert!(html.contains("console.log(&#39;new value&#39;)"));
    }

    #[test]
    fn test_render_nested_list() {
        let allocator = Allocator::new();
        // Indent with 2 spaces for nesting
        let doc = Parser::new(&allocator, "- item 1\n  - sub 1\n- item 2").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);

        // Normalize newlines for comparison
        let normalized = html.replace('\n', "");
        // We expect:
        // <ul>
        //   <li>
        //     <p>item 1</p>
        //     <ul>
        //       <li><p>sub 1</p></li>
        //     </ul>
        //   </li>
        //   <li><p>item 2</p></li>
        // </ul>
        // Note: The exact placement of <p> tags depends on how we handle list content.
        // Assuming tight list items might not have <p> if we implement loose/tight lists,
        // but currently everything is wrapped in <p> in parse_list implementation (wrapped in Paragraph).

        // Let's just check for the structure <li>...<ul>...</ul>...</li>
        assert!(normalized.contains("<li><p>item 1</p><ul><li><p>sub 1</p></li></ul></li>"));
        assert!(normalized.contains("<li><p>item 2</p></li>"));
    }

    #[test]
    fn test_render_table() {
        let allocator = Allocator::new();
        let parser_options = ox_content_parser::ParserOptions::gfm();
        let doc = Parser::with_options(&allocator, "| head |\n| --- |\n| body |", parser_options)
            .parse()
            .unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<th>head</th>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<td>body</td>"));
    }

    #[test]
    fn test_render_table_no_gfm() {
        let allocator = Allocator::new();
        // Default options have tables: false
        let doc = Parser::new(&allocator, "| head |\n| --- |\n| body |").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(!html.contains("<table>"));
        assert!(html.contains("| head |"));
    }

    #[test]
    fn test_render_heading_with_link() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "### [index](./index-module.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<h3 id=\"index\"><a href=\"./index-module.md\">index</a></h3>\n");
    }

    #[test]
    fn test_render_list_with_bold() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "- **bold** text").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_render_task_list() {
        let allocator = Allocator::new();
        let parser_options = ox_content_parser::ParserOptions::gfm();
        let doc = Parser::with_options(&allocator, "- [x] task 1\n- [ ] task 2", parser_options)
            .parse()
            .unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<input type=\"checkbox\" checked disabled> <p>task 1</p>"));
        assert!(html.contains("<input type=\"checkbox\" disabled> <p>task 2</p>"));
    }

    #[test]
    fn test_render_strikethrough() {
        let allocator = Allocator::new();
        let doc =
            Parser::with_options(&allocator, "~~done~~", ox_content_parser::ParserOptions::gfm())
                .parse()
                .unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<p><del>done</del></p>\n");
    }

    #[test]
    fn test_render_hard_break() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "line 1\\\nline 2").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert_eq!(html, "<p>line 1<br>\nline 2</p>\n");
    }

    #[test]
    fn test_render_image() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "![Alt text](/path/to/image.png)").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        assert!(html.contains("<img src=\"/path/to/image.png\" alt=\"Alt text\">"));
    }

    #[test]
    fn test_render_image_xhtml() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "![Logo](/logo.svg)").parse().unwrap();
        let mut renderer =
            HtmlRenderer::with_options(HtmlRendererOptions { xhtml: true, ..Default::default() });
        let html = renderer.render(&doc);
        assert!(html.contains("<img src=\"/logo.svg\" alt=\"Logo\" />"));
    }

    #[test]
    fn test_convert_md_link_from_index_file() {
        // When the source is an index file (api/index.md), relative links like ./docs.md
        // should become ./docs/index.html (not ../docs/index.html)
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Docs](./docs.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/index.md".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"./docs/index.html\""),
            "Expected ./docs/index.html but got: {html}"
        );
    }

    #[test]
    fn test_convert_md_link_from_non_index_file() {
        // When the source is NOT an index file (api/types.md -> becomes types/index.html),
        // relative links like ./docs.md should become ../docs/index.html
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Docs](./docs.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/types.md".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"../docs/index.html\""),
            "Expected ../docs/index.html but got: {html}"
        );
    }

    #[test]
    fn test_convert_md_link_plain_relative_from_index() {
        // Plain relative links (no ./) from index file
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Types](types.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/index.md".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"./types/index.html\""),
            "Expected ./types/index.html but got: {html}"
        );
    }

    #[test]
    fn test_convert_mdx_and_markdown_links() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Component](./component.mdx) [Guide](guide.markdown)")
            .parse()
            .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/index.mdx".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(html.contains("href=\"./component/index.html\""), "Got: {html}");
        assert!(html.contains("href=\"./guide/index.html\""), "Got: {html}");
    }

    #[test]
    fn test_convert_md_link_parent_relative_from_index() {
        // Parent-relative links from index file
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Guide](../guide.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/index.md".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"../guide/index.html\""),
            "Expected ../guide/index.html but got: {html}"
        );
    }

    #[test]
    fn test_convert_md_link_parent_relative_from_non_index() {
        // Parent-relative links from non-index file need extra ../
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "[Guide](../guide.md)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            convert_md_links: true,
            base_url: "/".to_string(),
            source_path: "api/types.md".to_string(),
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"../../guide/index.html\""),
            "Expected ../../guide/index.html but got: {html}"
        );
    }

    #[test]
    fn test_autolink_disabled_by_default() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "see http://example.com here").parse().unwrap();
        let mut renderer = HtmlRenderer::new();
        let html = renderer.render(&doc);
        // No <a> tag is emitted unless the flag is on.
        assert!(!html.contains("<a "), "unexpected autolink in: {html}");
        assert!(html.contains("http://example.com"));
    }

    #[test]
    fn test_autolink_basic_http_and_https() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "see http://example.com and https://example.org")
            .parse()
            .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains(
                "<a href=\"http://example.com\" target=\"_blank\" rel=\"noopener noreferrer\">http://example.com</a>"
            ),
            "missing http autolink in: {html}"
        );
        assert!(
            html.contains(
                "<a href=\"https://example.org\" target=\"_blank\" rel=\"noopener noreferrer\">https://example.org</a>"
            ),
            "missing https autolink in: {html}"
        );
    }

    #[test]
    fn test_autolink_target_blank_can_be_disabled() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "go to https://example.com now").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            autolink_target_blank: false,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("<a href=\"https://example.com\">https://example.com</a>"),
            "expected bare anchor in: {html}"
        );
        assert!(!html.contains("target=\"_blank\""), "blank attr leaked: {html}");
    }

    #[test]
    fn test_autolink_strips_trailing_punctuation() {
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "find it at https://example.com. or (https://example.org) maybe",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(html.contains(">https://example.com</a>."), "period leaked: {html}");
        assert!(html.contains("(<a href=\"https://example.org\""), "open paren lost: {html}");
        assert!(html.contains(">https://example.org</a>)"), "close paren lost: {html}");
    }

    #[test]
    fn test_autolink_word_boundary_required() {
        let allocator = Allocator::new();
        // "shttp://x" must not match — the prefix is glued to a word char.
        let doc = Parser::new(&allocator, "shttp://x and http://y").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(!html.contains("href=\"http://x\""), "unexpected glued autolink: {html}");
        assert!(html.contains("href=\"http://y\""), "missing real autolink: {html}");
    }

    #[test]
    fn test_autolink_custom_pattern_registration() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "email mailto:foo@example.com please").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            autolink_patterns: vec!["mailto:".to_string()],
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("<a href=\"mailto:foo@example.com\""),
            "missing custom-pattern autolink: {html}"
        );
    }

    #[test]
    fn test_autolink_many_patterns_uses_table_fallback() {
        // Five patterns with five distinct leading letters exceed the
        // three-needle SIMD fast path, exercising the `FirstByteIndex`
        // lookup-table fallback. All schemes must still autolink.
        let allocator = Allocator::new();
        let doc = Parser::new(
            &allocator,
            "a http://h.test b ftp://f.test c mailto:m@x d tel:123 e ssh://s.test f",
        )
        .parse()
        .unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            autolink_patterns: vec![
                "http://".to_string(),
                "ftp://".to_string(),
                "mailto:".to_string(),
                "tel:".to_string(),
                "ssh://".to_string(),
            ],
            ..Default::default()
        });
        let html = renderer.render(&doc);
        for href in ["http://h.test", "ftp://f.test", "mailto:m@x", "tel:123", "ssh://s.test"] {
            assert!(html.contains(&format!("<a href=\"{href}\"")), "missing {href} in: {html}");
        }
    }

    #[test]
    fn test_autolink_does_not_nest_inside_existing_link() {
        let allocator = Allocator::new();
        // The text inside the explicit markdown link contains a URL — the
        // builtin must not wrap that URL in a second <a>.
        let doc =
            Parser::new(&allocator, "[visit https://example.com here](/page)").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert_eq!(html.matches("<a ").count(), 1, "nested anchor in: {html}");
        assert!(html.contains("href=\"/page\""), "outer link lost: {html}");
        assert!(html.contains("visit https://example.com here"), "inner text lost: {html}");
    }

    #[test]
    fn test_autolink_escapes_query_string_safely() {
        let allocator = Allocator::new();
        // `&` inside the URL must be escaped both as href and as visible
        // text — otherwise the output would be parser-ambiguous HTML.
        let doc = Parser::new(&allocator, "see http://a.test/?q=foo&r=bar now").parse().unwrap();
        let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
            autolink_urls: true,
            ..Default::default()
        });
        let html = renderer.render(&doc);
        assert!(
            html.contains("href=\"http://a.test/?q=foo&amp;r=bar\""),
            "href not escaped: {html}"
        );
        assert!(
            html.contains(">http://a.test/?q=foo&amp;r=bar</a>"),
            "visible text not escaped: {html}"
        );
    }
}
