use super::{ReportMode, ResolvedTimelineOptions};

#[derive(Default)]
pub(super) struct ItemMeta {
    pub(super) label: Option<String>,
    pub(super) status: Option<String>,
    pub(super) href: Option<String>,
}

pub(super) fn parse_item_meta(
    source: &str,
    line: usize,
    options: &ResolvedTimelineOptions,
    diagnostics: &mut Vec<String>,
    meta: &mut ItemMeta,
) {
    for token in split_meta_tokens(source) {
        let Some((key, value)) = token.split_once('=') else {
            push_unknown_meta(line, &token, options, diagnostics);
            continue;
        };
        let value = unquote(value).to_string();
        match key {
            "label" => meta.label = Some(value),
            "status" if is_safe_token(&value) => meta.status = Some(value),
            "status" => push_unknown_meta(line, &format!("status={value}"), options, diagnostics),
            "href" if is_safe_href(&value) => meta.href = Some(value),
            "href" => push_unknown_meta(line, "href", options, diagnostics),
            _ => push_unknown_meta(line, key, options, diagnostics),
        }
    }
}

pub(super) fn push_diagnostic(mode: ReportMode, message: String, diagnostics: &mut Vec<String>) {
    match mode {
        ReportMode::Ignore => {}
        ReportMode::Warn => diagnostics.push(message),
        ReportMode::Error => diagnostics.push(format!("error:{message}")),
    }
}

pub(super) fn strip_trailing_meta(value: &str) -> (&str, Option<&str>) {
    let trimmed = value.trim_end();
    if !trimmed.ends_with('}') {
        return (trimmed, None);
    }
    let Some(start) = trimmed.rfind('{') else {
        return (trimmed, None);
    };
    (trimmed[..start].trim_end(), Some(trimmed[start + 1..trimmed.len() - 1].trim()))
}

pub(super) fn split_meta_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    for ch in value.chars() {
        if quote == Some(ch) {
            quote = None;
            token.push(ch);
            continue;
        }
        if quote.is_none() && (ch == '"' || ch == '\'') {
            quote = Some(ch);
            token.push(ch);
            continue;
        }
        if quote.is_none() && ch.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
            continue;
        }
        token.push(ch);
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

pub(super) fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|value| value.strip_suffix('\'')))
        .unwrap_or(value)
}

fn push_unknown_meta(
    line: usize,
    key: &str,
    options: &ResolvedTimelineOptions,
    diagnostics: &mut Vec<String>,
) {
    push_diagnostic(
        options.unknown_meta,
        format!("Timeline item on line {line} has unsupported metadata `{key}`."),
        diagnostics,
    );
}

fn is_safe_token(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

fn is_safe_href(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    !lower.is_empty()
        && !lower.starts_with("javascript:")
        && !lower.starts_with("data:")
        && !lower.starts_with("vbscript:")
        && !lower.starts_with("//")
}
