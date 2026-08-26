//! Opt-in abbreviation and glossary expansion.
//!
//! Disabled by default. When enabled, `*[LSP]: Language Server Protocol`
//! definitions and config `terms` become
//! `<abbr class="ox-abbr" title="Language Server Protocol">LSP</abbr>`.
//! Matching uses Unicode word boundaries and skips code, comments, links,
//! and raw `pre` / `script` / `style`. There is no client JavaScript.

use rustc_hash::FxHashSet;
use std::borrow::Cow;

use crate::AbbreviationsOptions;

mod protect;

#[cfg(test)]
mod tests;

use protect::next_protected;

#[derive(Clone)]
pub(super) struct ResolvedAbbreviations {
    terms: Vec<(String, String)>,
    first_use_only: bool,
}

struct TermMatch<'a> {
    start: usize,
    end: usize,
    term: &'a str,
    title: &'a str,
}

pub(super) fn resolve(options: Option<&AbbreviationsOptions>) -> Option<ResolvedAbbreviations> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    let mut terms = Vec::new();
    if let Some(config) = &options.terms {
        for (term, title) in config {
            let term = term.trim();
            let title = title.trim();
            if term.is_empty() || title.is_empty() {
                continue;
            }
            terms.push((term.to_string(), title.to_string()));
        }
    }
    Some(ResolvedAbbreviations { terms, first_use_only: options.first_use_only.unwrap_or(false) })
}

pub(super) fn apply(current: &mut Cow<'_, str>, options: Option<&ResolvedAbbreviations>) {
    let Some(options) = options else {
        return;
    };
    let mut terms = options.terms.clone();
    if current.contains("*[")
        && let Some(stripped) =
            super::segments::transform_markdown_text_segments(current, |segment, out| {
                strip_or_copy_definition(segment, &mut terms, out);
            })
    {
        *current = Cow::Owned(stripped);
    }
    if terms.is_empty() {
        return;
    }
    terms.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    let mut used = FxHashSet::default();
    if let Some(replaced) =
        super::segments::transform_markdown_text_segments(current, |segment, out| {
            replace(segment, &terms, options.first_use_only, &mut used, out);
        })
    {
        *current = Cow::Owned(replaced);
    }
}

fn strip_or_copy_definition(segment: &str, terms: &mut Vec<(String, String)>, out: &mut String) {
    if let Some((term, title)) = parse_standalone_definition(segment) {
        if let Some(existing) = terms.iter_mut().find(|entry| entry.0 == term) {
            existing.1 = title;
        } else {
            terms.push((term, title));
        }
        return;
    }
    out.push_str(segment);
}

fn parse_standalone_definition(segment: &str) -> Option<(String, String)> {
    let trimmed = segment.trim();
    let rest = trimmed.strip_prefix("*[")?;
    let close = rest.find("]:")?;
    let term = rest[..close].trim();
    let title = rest[close + 2..].trim();
    if term.is_empty() || title.is_empty() || term.contains(']') {
        return None;
    }
    Some((term.to_string(), title.to_string()))
}

fn replace(
    segment: &str,
    terms: &[(String, String)],
    first_use_only: bool,
    used: &mut FxHashSet<String>,
    out: &mut String,
) {
    let mut cursor = 0usize;
    while cursor < segment.len() {
        let protected = next_protected(segment, cursor);
        let found = next_term(segment, cursor, terms, protected.as_ref().map(|span| span.start));
        match (found, protected) {
            (None, None) => {
                out.push_str(&segment[cursor..]);
                return;
            }
            (None, Some(span)) => {
                out.push_str(&segment[cursor..span.end]);
                cursor = span.end;
            }
            (Some(found), Some(span)) if span.start <= found.start => {
                out.push_str(&segment[cursor..span.end]);
                cursor = span.end;
            }
            (Some(found), _) => {
                out.push_str(&segment[cursor..found.start]);
                cursor = found.end;
                emit_or_copy(found, first_use_only, used, out);
            }
        }
    }
}

fn emit_or_copy(
    found: TermMatch<'_>,
    first_use_only: bool,
    used: &mut FxHashSet<String>,
    out: &mut String,
) {
    if first_use_only && !used.insert(found.term.to_string()) {
        out.push_str(found.term);
        return;
    }
    out.push_str("<abbr class=\"ox-abbr\" title=\"");
    super::escape_html_attr(found.title, out);
    out.push_str("\">");
    super::escape_html_text(found.term, out);
    out.push_str("</abbr>");
}

fn next_term<'a>(
    segment: &'a str,
    from: usize,
    terms: &'a [(String, String)],
    limit: Option<usize>,
) -> Option<TermMatch<'a>> {
    let limit = limit.unwrap_or(segment.len());
    let mut cursor = from;
    while cursor < limit {
        if !segment.is_char_boundary(cursor) {
            cursor += 1;
            continue;
        }
        if left_boundary(segment, cursor) {
            for (term, title) in terms {
                let end = cursor + term.len();
                if end <= limit
                    && segment.is_char_boundary(end)
                    && segment[cursor..].starts_with(term.as_str())
                    && right_boundary(segment, end)
                {
                    return Some(TermMatch { start: cursor, end, term, title });
                }
            }
        }
        cursor = next_char_index(segment, cursor);
    }
    None
}

fn left_boundary(segment: &str, index: usize) -> bool {
    let Some(previous) = segment[..index].chars().next_back() else {
        return true;
    };
    let Some(current) = segment[index..].chars().next() else {
        return true;
    };
    !continues_word(previous, current)
}

fn right_boundary(segment: &str, index: usize) -> bool {
    let Some(next) = segment[index..].chars().next() else {
        return true;
    };
    let Some(previous) = segment[..index].chars().next_back() else {
        return true;
    };
    !continues_word(previous, next)
}

fn continues_word(left: char, right: char) -> bool {
    if is_ascii_word_char(left) && is_ascii_word_char(right) {
        return true;
    }
    if is_cjk(left) && is_cjk(right) {
        return true;
    }
    left.is_alphanumeric()
        && right.is_alphanumeric()
        && !is_ascii_word_char(left)
        && !is_ascii_word_char(right)
        && !is_cjk(left)
        && !is_cjk(right)
}

fn is_ascii_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch,
        '\u{3000}'..='\u{303F}'
            | '\u{3040}'..='\u{30FF}'
            | '\u{31F0}'..='\u{31FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF66}'..='\u{FF9D}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{20000}'..='\u{2A6DF}'
    )
}

fn next_char_index(segment: &str, index: usize) -> usize {
    segment[index..].chars().next().map_or(segment.len(), |ch| index + ch.len_utf8())
}
