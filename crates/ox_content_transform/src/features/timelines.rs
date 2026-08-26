//! Opt-in static `::: timeline` milestone lists.
//!
//! Timeline blocks are disabled by default. When enabled, closed blocks are
//! rewritten to semantic static HTML while nested item bodies stay Markdown so
//! the normal renderer can keep headings, links, and emphasis searchable.

use crate::TimelineOptions;

mod date;
mod html;
mod meta;
mod parse;
#[cfg(test)]
mod tests;

use html::emit_timeline;
use parse::{TimelineOpen, parse_timeline_items, parse_timeline_open};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReportMode {
    Ignore,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ResolvedTimelineOptions {
    ordered: bool,
    invalid_date: ReportMode,
    unknown_meta: ReportMode,
    empty: ReportMode,
}

pub(super) fn resolve(options: Option<&TimelineOptions>) -> Option<ResolvedTimelineOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedTimelineOptions {
        ordered: options.ordered.unwrap_or(true),
        invalid_date: report_mode(options.invalid_date.as_deref(), ReportMode::Error),
        unknown_meta: report_mode(options.unknown_meta.as_deref(), ReportMode::Error),
        empty: report_mode(options.empty.as_deref(), ReportMode::Error),
    })
}

pub(super) fn transform(
    source: &str,
    options: &ResolvedTimelineOptions,
    errors: &mut Vec<String>,
) -> String {
    if !source.contains(":::") || !source.contains("timeline") {
        return source.to_string();
    }
    rewrite(source, options, errors)
}

fn rewrite(source: &str, options: &ResolvedTimelineOptions, errors: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(source.len());
    let mut lines = source.split_inclusive('\n');
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    while let Some(line_with_end) = lines.next() {
        let (line, ending) = split_ending(line_with_end);

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if super::segments::is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        if let Some(open) = super::segments::parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        let Some(open) = parse_timeline_open(line, options.ordered) else {
            out.push_str(line);
            out.push_str(ending);
            continue;
        };

        let mut body = String::new();
        let mut close = None;
        for inner in lines.by_ref() {
            let (inner_line, inner_end) = split_ending(inner);
            if is_timeline_close(inner_line, open.indent, open.colon_count) {
                close = Some((inner_line, inner_end));
                break;
            }
            body.push_str(inner_line);
            body.push_str(inner_end);
        }

        let original = original_block(line, ending, &body, close);
        let Some((_, _)) = close else {
            errors.push("Timeline block is missing a closing ::: fence.".to_string());
            out.push_str(&original);
            continue;
        };

        match render_timeline(&open, &body, options) {
            Ok((html, diagnostics)) => {
                errors.extend(diagnostics);
                out.push_str(&html);
            }
            Err(diagnostics) => {
                errors.extend(diagnostics);
                out.push_str(&original);
            }
        }
    }

    out
}

fn render_timeline(
    open: &TimelineOpen,
    body: &str,
    options: &ResolvedTimelineOptions,
) -> Result<(String, Vec<String>), Vec<String>> {
    let (items, mut diagnostics) = parse_timeline_items(body, options);
    if items.is_empty()
        && push_diagnostic(options.empty, "Timeline block is empty.".to_string(), &mut diagnostics)
    {
        return Err(diagnostics);
    }
    if diagnostics.iter().any(|message| message.starts_with("error:")) {
        for message in &mut diagnostics {
            if let Some(stripped) = message.strip_prefix("error:") {
                *message = stripped.to_string();
            }
        }
        return Err(diagnostics);
    }
    let mut out = String::new();
    emit_timeline(&mut out, open.caption.as_deref(), open.ordered, &items);
    Ok((out, diagnostics))
}

fn push_diagnostic(mode: ReportMode, message: String, diagnostics: &mut Vec<String>) -> bool {
    match mode {
        ReportMode::Ignore => false,
        ReportMode::Warn => {
            diagnostics.push(message);
            false
        }
        ReportMode::Error => {
            diagnostics.push(format!("error:{message}"));
            true
        }
    }
}

fn original_block(line: &str, ending: &str, body: &str, close: Option<(&str, &str)>) -> String {
    let mut original = String::with_capacity(line.len() + ending.len() + body.len() + 8);
    original.push_str(line);
    original.push_str(ending);
    original.push_str(body);
    if let Some((close_line, close_end)) = close {
        original.push_str(close_line);
        original.push_str(close_end);
    }
    original
}

fn is_timeline_close(line: &str, opener_indent: usize, colon_count: usize) -> bool {
    let indent = line.len() - line.trim_start().len();
    if indent > opener_indent {
        return false;
    }
    let trimmed = line.trim_start();
    let count = trimmed.bytes().take_while(|byte| *byte == b':').count();
    count >= colon_count
        && trimmed
            .as_bytes()
            .get(count..)
            .is_none_or(|rest| rest.iter().all(u8::is_ascii_whitespace))
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

fn report_mode(value: Option<&str>, default: ReportMode) -> ReportMode {
    match value.map(str::trim) {
        Some("ignore") => ReportMode::Ignore,
        Some("warn") => ReportMode::Warn,
        Some("error") | None => default,
        Some(_) => default,
    }
}
