//! Opt-in `::: type` custom containers.
//!
//! Disabled by default. GitHub-style `> [!NOTE]` callouts stay on the existing
//! renderer path and are not rewritten here.

use rustc_hash::FxHashMap;

use crate::ContainerOptions;

mod parse;
#[cfg(test)]
mod tests;

use parse::{ParsedOpener, normalize_type_name, parse_closer, parse_opener};

const BUILTIN_TYPES: &[(&str, ContainerKind)] = &[
    ("tip", ContainerKind::Div),
    ("note", ContainerKind::Div),
    ("info", ContainerKind::Div),
    ("important", ContainerKind::Div),
    ("warning", ContainerKind::Div),
    ("danger", ContainerKind::Div),
    ("caution", ContainerKind::Div),
    ("details", ContainerKind::Details),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ContainerKind {
    Div,
    Details,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedContainerType {
    pub(super) name: String,
    pub(super) title: String,
    pub(super) kind: ContainerKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedContainerOptions {
    pub(super) types: FxHashMap<String, ResolvedContainerType>,
}

pub(super) fn resolve(options: Option<&ContainerOptions>) -> Option<ResolvedContainerOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }

    let mut types = FxHashMap::default();
    for (name, kind) in BUILTIN_TYPES {
        types.insert(
            (*name).to_string(),
            ResolvedContainerType {
                name: (*name).to_string(),
                title: default_title(name),
                kind: *kind,
            },
        );
    }

    if let Some(custom) = &options.types {
        for (name, spec) in custom {
            let Some(key) = normalize_type_name(name) else {
                continue;
            };
            let kind = match spec.tag.as_deref() {
                Some("details") => ContainerKind::Details,
                _ => ContainerKind::Div,
            };
            types.insert(
                key.clone(),
                ResolvedContainerType {
                    title: spec
                        .title
                        .clone()
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| default_title(&key)),
                    name: key,
                    kind,
                },
            );
        }
    }

    Some(ResolvedContainerOptions { types })
}

pub(super) fn remove_reserved_type_names(
    options: &mut Option<ResolvedContainerOptions>,
    cards: Option<&[&str]>,
    steps: bool,
    code_groups: Option<&[&str]>,
    galleries: bool,
    timelines: bool,
) {
    let Some(options) = options.as_mut() else {
        return;
    };
    for names in [cards, code_groups].into_iter().flatten() {
        for name in names {
            options.types.remove(*name);
        }
    }
    if steps {
        options.types.remove("steps");
    }
    if galleries {
        options.types.remove("gallery");
    }
    if timelines {
        options.types.remove("timeline");
    }
}

pub(super) fn transform(source: &str, options: &ResolvedContainerOptions) -> String {
    let mut out = String::with_capacity(source.len() + 64);
    let mut lines = source.split_inclusive('\n').peekable();
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut stack: Vec<OpenContainer<'_>> = Vec::new();

    while let Some(line_with_end) = lines.next() {
        let (line, ending) = split_ending(line_with_end);

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.0;
            fence_len = open.1;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        let trimmed = trim_container_indent(line);
        if let Some(opener) = parse_opener(trimmed)
            && options.types.contains_key(&opener.name)
        {
            let spec = &options.types[&opener.name];
            emit_open(&mut out, spec, &opener);
            stack.push(OpenContainer { spec, colon_count: opener.colon_count });
            continue;
        }

        if let Some(close) = parse_closer(trimmed)
            && let Some(index) = stack.iter().rposition(|open| open.colon_count <= close)
        {
            while stack.len() > index {
                if let Some(inner) = stack.pop() {
                    emit_close(&mut out, inner.spec);
                }
            }
            continue;
        }

        if stack.is_empty() {
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        out.push_str(line);
        out.push_str(ending);
        let _ = lines.peek();
    }

    while let Some(open) = stack.pop() {
        emit_close(&mut out, open.spec);
    }

    out
}

struct OpenContainer<'a> {
    spec: &'a ResolvedContainerType,
    colon_count: usize,
}

fn default_title(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => {
            let mut title = first.to_uppercase().collect::<String>();
            title.extend(chars);
            title
        }
        None => name.to_string(),
    }
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

fn trim_container_indent(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}

fn parse_opening_fence(line: &str) -> Option<(u8, usize)> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let fence_char = *bytes.first()?;
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }
    let fence_len = bytes.iter().take_while(|byte| **byte == fence_char).count();
    (fence_len >= 3).then_some((fence_char, fence_len))
}

fn is_closing_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let close_len = bytes.iter().take_while(|byte| **byte == fence_char).count();
    close_len >= fence_len
        && bytes.get(close_len..).is_none_or(|rest| rest.iter().all(u8::is_ascii_whitespace))
}

fn emit_open(out: &mut String, spec: &ResolvedContainerType, opener: &ParsedOpener) {
    let title = opener.title.as_deref().unwrap_or(spec.title.as_str());
    let tag = match spec.kind {
        ContainerKind::Details => "details",
        ContainerKind::Div => "div",
    };
    out.push('<');
    out.push_str(tag);
    out.push_str(" class=\"ox-container ox-container--");
    out.push_str(&spec.name);
    write_class_suffix(out, &opener.attrs);
    out.push('"');
    write_id_and_flags(out, &opener.attrs, spec.kind == ContainerKind::Details);
    match spec.kind {
        ContainerKind::Details => {
            out.push_str(">\n<summary>");
            escape_html(title, out);
            out.push_str("</summary>\n\n");
        }
        ContainerKind::Div => {
            out.push_str(">\n<p class=\"ox-container-title\">");
            escape_html(title, out);
            out.push_str("</p>\n\n");
        }
    }
}

fn emit_close(out: &mut String, spec: &ResolvedContainerType) {
    match spec.kind {
        ContainerKind::Details => out.push_str("\n</details>\n"),
        ContainerKind::Div => out.push_str("\n</div>\n"),
    }
}

fn write_class_suffix(out: &mut String, attrs: &[(String, Option<String>)]) {
    for (key, value) in attrs {
        if key == "class"
            && let Some(class) = value
        {
            out.push(' ');
            out.push_str(class);
        }
    }
}

fn write_id_and_flags(out: &mut String, attrs: &[(String, Option<String>)], allow_open: bool) {
    for (key, value) in attrs {
        match (key.as_str(), value.as_deref()) {
            ("id", Some(id)) => {
                out.push_str(" id=\"");
                escape_html(id, out);
                out.push('"');
            }
            ("open", None) if allow_open => out.push_str(" open"),
            _ => {}
        }
    }
}

fn escape_html(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}
