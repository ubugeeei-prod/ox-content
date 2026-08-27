//! Numbering `<table>` elements, and lifting labels authored beneath one.

use super::super::html::{
    append_attr, append_data_attrs, escape_url_fragment, expected_kind, read_attr,
    should_track_target,
};
use super::super::types::{CrossReferenceEntry, CrossReferenceKind, CrossReferencesOptions};
use super::figures::find_tag;
use super::{Registry, TrackedTarget};
use crate::html_scan::find_ci;
use std::fmt::Write as _;

pub fn annotate_tables(
    html: &str,
    options: &CrossReferencesOptions,
    registry: &mut Registry,
) -> String {
    let mut count = 0usize;
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(open) = find_tag(html, cursor, "table") {
        out.push_str(&html[cursor..open.start]);
        let attrs = &html[open.attrs_start..open.attrs_end];
        match read_attr(attrs, "id").filter(|id| should_track_target(id)) {
            None => out.push_str(&html[open.start..=open.attrs_end]),
            Some(id) => {
                count += 1;
                let number = count.to_string();
                let text = format!("{} {}", options.labels.table, number);
                registry.register(
                    options.duplicates,
                    TrackedTarget {
                        entry: CrossReferenceEntry {
                            href: format!("#{}", escape_url_fragment(&id)),
                            id,
                            kind: CrossReferenceKind::Table,
                            number: number.clone(),
                            label: options.labels.table.clone(),
                            text: text.clone(),
                            title: None,
                        },
                        position: open.start,
                    },
                );
                let _ = write!(
                    out,
                    "<table{}>",
                    append_data_attrs(attrs, CrossReferenceKind::Table, &number, &text)
                );
            }
        }
        cursor = open.attrs_end + 1;
    }
    out.push_str(&html[cursor..]);
    out
}

/// Moves a `{#tbl-x}` label written after a table onto the table itself.
///
/// Markdown gives no way to put an attribute on a table, so the label is
/// authored below it — either as a paragraph or as a trailing empty row — and
/// lifted here before the numbering pass sees it.
pub fn apply_trailing_table_labels(html: &str) -> String {
    lift_trailing_cell_labels(&lift_paragraph_labels(html))
}

fn lift_paragraph_labels(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(open) = find_tag(html, cursor, "table") {
        let Some(close) = find_ci(html, open.attrs_end + 1, "</table>") else {
            break;
        };
        let table_end = close + "</table>".len();
        let Some((label, consumed_to)) = trailing_paragraph_label(html, table_end) else {
            out.push_str(&html[cursor..table_end]);
            cursor = table_end;
            continue;
        };
        let attrs = &html[open.attrs_start..open.attrs_end];
        if read_attr(attrs, "id").is_some() {
            out.push_str(&html[cursor..table_end]);
            cursor = table_end;
            continue;
        }
        out.push_str(&html[cursor..open.start]);
        let _ = write!(out, "<table{}>", append_attr(attrs, "id", &label));
        out.push_str(&html[open.attrs_end + 1..table_end]);
        cursor = consumed_to;
    }
    out.push_str(&html[cursor..]);
    out
}

/// `\s*<p>{#id}</p>` immediately after the table.
fn trailing_paragraph_label(html: &str, from: usize) -> Option<(String, usize)> {
    let after_space = skip_whitespace(html, from);
    let rest = &html[after_space..];
    let body = rest.strip_prefix("<p>{#")?;
    let end = body.find("}</p>")?;
    let id = &body[..end];
    if !is_label_identifier(id) {
        return None;
    }
    Some((id.to_string(), after_space + rest.len() - body.len() + end + "}</p>".len()))
}

fn lift_trailing_cell_labels(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(open) = find_tag(html, cursor, "table") {
        let Some(close) = find_ci(html, open.attrs_end + 1, "</table>") else {
            break;
        };
        let table_end = close + "</table>".len();
        let attrs = &html[open.attrs_start..open.attrs_end];
        let body = &html[open.attrs_end + 1..close];

        match read_attr(attrs, "id").is_none().then(|| trailing_cell_label(body)).flatten() {
            None => out.push_str(&html[cursor..table_end]),
            Some((id, next_body)) => {
                out.push_str(&html[cursor..open.start]);
                let _ = write!(out, "<table{}>{next_body}</table>", append_attr(attrs, "id", &id));
            }
        }
        cursor = table_end;
    }
    out.push_str(&html[cursor..]);
    out
}

/// A final row of empty cells whose first cell carries a table id.
///
/// Returns the id and the body with that row removed.
fn trailing_cell_label(body: &str) -> Option<(String, String)> {
    let trailing = body.trim_end();
    let tbody_end = trailing.strip_suffix("</tbody>").map(str::trim_end);
    let (scan, tail) = match tbody_end {
        Some(rest) => (rest, &body[rest.trim_end().len()..]),
        None => (trailing, &body[trailing.len()..]),
    };
    let row_start = scan.rfind("<tr>")?;
    let row = &scan[row_start..];
    if !row.trim_end().ends_with("</tr>") {
        return None;
    }
    let (id, all_empty) = empty_row_id(row)?;
    if !all_empty || expected_kind(&id) != Some(CrossReferenceKind::Table) {
        return None;
    }
    Some((id, format!("{}{}", &scan[..row_start], tail)))
}

/// Reads the id off a row whose cells are all empty.
fn empty_row_id(row: &str) -> Option<(String, bool)> {
    let mut id = None;
    let mut cursor = 0usize;
    let mut all_empty = true;
    let mut cells = 0usize;

    while let Some(open) = find_tag(row, cursor, "td") {
        let close = find_ci(row, open.attrs_end + 1, "</td>")?;
        if !row[open.attrs_end + 1..close].trim().is_empty() {
            all_empty = false;
        }
        if cells == 0 {
            id = read_attr(&row[open.attrs_start..open.attrs_end], "id");
        }
        cells += 1;
        cursor = close + "</td>".len();
    }
    id.filter(|_| cells > 0).map(|value| (value, all_empty))
}

fn skip_whitespace(html: &str, from: usize) -> usize {
    let bytes = html.as_bytes();
    let mut cursor = from;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}

/// `[A-Za-z][A-Za-z0-9_-]*`
fn is_label_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}
