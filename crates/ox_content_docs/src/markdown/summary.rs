use std::borrow::Cow;
use std::sync::OnceLock;

use super::{cached_regex, process_doc_text, MarkdownLinkContext, RegexCache};

pub(super) fn collapse_inline_whitespace(text: &str) -> Cow<'_, str> {
    let text = text.trim();
    if text.is_empty() {
        return Cow::Borrowed("");
    }
    // Fast path: nothing to collapse iff every whitespace char is a lone ASCII
    // space. Any other whitespace char, or two adjacent whitespace chars, would
    // be rewritten below, so it must be owned.
    let needs_collapse = {
        let mut prev_ws = false;
        text.chars().any(|ch| {
            let collapse = ch.is_whitespace() && (ch != ' ' || prev_ws);
            prev_ws = ch.is_whitespace();
            collapse
        })
    };
    if !needs_collapse {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(ch);
        }
    }
    Cow::Owned(out)
}

/// Collapses type annotations for inline rendering while avoiding spaces created
/// by multiline generic formatting, e.g. `Foo<\n  Bar\n>` -> `Foo<Bar>`.
pub(super) fn collapse_type_annotation_whitespace(text: &str) -> Cow<'_, str> {
    let text = text.trim();
    if text.is_empty() {
        return Cow::Borrowed("");
    }

    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }

        if pending_space {
            if !matches!(out.chars().next_back(), Some('<')) && ch != '>' {
                out.push(' ');
            }
            pending_space = false;
        }
        out.push(ch);
    }

    if out == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(out)
    }
}

/// One-line summary for a module index table cell.
///
/// Resolves `{@link}`/`{@linkcode}` exactly like the per-symbol pages (keeping
/// the produced Markdown links and inline code), takes the first paragraph,
/// collapses it to a single line, and escapes table-cell pipes. Unlike
/// [`clean_summary_text`] it does not strip links/code, so the index matches
/// TypeDoc (e.g. `An object that contains [argument schema](…).`).
pub(super) fn typedoc_index_summary(
    description: &str,
    context: &MarkdownLinkContext<'_>,
) -> String {
    markdown_index_summary(description, Some(context))
}

pub(super) fn markdown_index_summary(
    description: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let resolved = process_doc_text(description, context);
    let first_paragraph = resolved.split("\n\n").next().unwrap_or_default();
    // Collapse the first paragraph onto one line (words joined by single
    // spaces) without an intermediate `Vec`, then escape table-cell pipes.
    let mut one_line = String::with_capacity(first_paragraph.len());
    for word in first_paragraph.split_whitespace() {
        if !one_line.is_empty() {
            one_line.push(' ');
        }
        one_line.push_str(word);
    }
    one_line.replace('|', "\\|")
}

pub(super) fn clean_summary_text(text: &str, max_length: usize) -> String {
    static MARKDOWN_LINK_RE: RegexCache = OnceLock::new();
    static BRACKET_LINK_RE: RegexCache = OnceLock::new();
    static INLINE_CODE_RE: RegexCache = OnceLock::new();
    static WHITESPACE_RE: RegexCache = OnceLock::new();

    if text.is_empty() {
        return String::new();
    }

    let fallback = || text.split_whitespace().collect::<Vec<_>>().join(" ");
    let Some(markdown_link_re) = cached_regex(&MARKDOWN_LINK_RE, r"\[([^\]]+)\]\([^)]+\)") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(bracket_link_re) = cached_regex(&BRACKET_LINK_RE, r"\[([^\]]+)\]") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(inline_code_re) = cached_regex(&INLINE_CODE_RE, r"`([^`]+)`") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(whitespace_re) = cached_regex(&WHITESPACE_RE, r"\s+") else {
        return truncate_summary_text(&fallback(), max_length);
    };

    // Summary cleanup is called for every entry in index views. `replace_all`
    // returns `Cow::Borrowed` when a pattern does not match, which is common
    // for short summaries, so thread the borrowed/owned value through each
    // regex stage and materialize only in `truncate_summary_text`.
    let s1 = markdown_link_re.replace_all(text, "$1");
    let s2 = bracket_link_re.replace_all(&s1, "$1");
    let s3 = inline_code_re.replace_all(&s2, "$1");
    let s4 = whitespace_re.replace_all(&s3, " ");

    truncate_summary_text(s4.trim(), max_length)
}

fn truncate_summary_text(text: &str, max_length: usize) -> String {
    if text.chars().count() <= max_length {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_length.saturating_sub(1)).collect();
    let trimmed = truncated.trim_end();
    let mut value = String::with_capacity(trimmed.len() + "…".len());
    value.push_str(trimmed);
    value.push('…');
    value
}
