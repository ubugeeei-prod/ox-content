//! The three numbering passes: sections, figures, and tables.
//!
//! Each walks the HTML once, assigns the next number to every element carrying
//! a trackable `id`, and writes the `data-ox-xref-*` attributes the reference
//! pass and the stylesheet read.

use super::html::{
    append_attr, append_data_attrs, escape_url_fragment, expected_kind, is_word_byte, read_attr,
    should_track_target, text_content,
};
use super::types::{
    CrossReferenceDiagnostic, CrossReferenceEntry, CrossReferenceKind, CrossReferencesOptions,
    FailureMode,
};
use crate::html_scan::find_ci;
use rustc_hash::FxHashMap;
use std::fmt::Write as _;

/// A target plus where it was found, so the caller can report in document order.
pub(super) struct TrackedTarget {
    pub entry: CrossReferenceEntry,
    pub position: usize,
}

/// Everything the passes accumulate.
///
/// The vector keeps discovery order for the final sort; the index answers the
/// lookups. Every registration and every `@ref` in the document does a lookup,
/// so scanning the vector instead made the whole pass quadratic — measurably
/// so: a 100-section document took 88ms that way against 1.9ms for the
/// TypeScript this replaced.
pub(super) struct Registry {
    pub targets: Vec<TrackedTarget>,
    index: FxHashMap<String, usize>,
    pub diagnostics: Vec<CrossReferenceDiagnostic>,
}

impl Registry {
    pub(super) fn new() -> Self {
        Self { targets: Vec::new(), index: FxHashMap::default(), diagnostics: Vec::new() }
    }

    pub(super) fn get(&self, id: &str) -> Option<&CrossReferenceEntry> {
        self.index.get(id).map(|slot| &self.targets[*slot].entry)
    }

    fn register(&mut self, policy: FailureMode, target: TrackedTarget) {
        if self.index.contains_key(&target.entry.id) {
            self.diagnostics.push(CrossReferenceDiagnostic {
                policy,
                message: format!("duplicate cross-reference target \"{}\"", target.entry.id),
            });
            return;
        }
        self.index.insert(target.entry.id.clone(), self.targets.len());
        self.targets.push(target);
    }
}

/// Numbers `<h1>`–`<h6>` hierarchically: `1`, `1.2`, `1.2.1`.
pub(super) fn annotate_sections(
    html: &str,
    options: &CrossReferencesOptions,
    registry: &mut Registry,
) -> String {
    let mut counters = [0usize; 6];
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(open) = find_heading(html, cursor) {
        let depth = open.depth;
        counters[depth - 1] += 1;
        // A deeper level restarts once its parent moves on.
        counters[depth..].fill(0);

        out.push_str(&html[cursor..open.start]);
        let attrs = &html[open.attrs_start..open.attrs_end];
        let body = &html[open.body_start..open.body_end];

        match read_attr(attrs, "id").filter(|id| should_track_target(id)) {
            None => out.push_str(&html[open.start..open.end]),
            Some(id) => {
                let number = section_number(&counters, depth);
                let text = format!("{} {}", options.labels.section, number);
                registry.register(
                    options.duplicates,
                    TrackedTarget {
                        entry: CrossReferenceEntry {
                            href: format!("#{}", escape_url_fragment(&id)),
                            id,
                            kind: CrossReferenceKind::Section,
                            number: number.clone(),
                            label: options.labels.section.clone(),
                            text: text.clone(),
                            title: Some(text_content(body)),
                        },
                        position: open.start,
                    },
                );
                let _ = write!(
                    out,
                    "<h{depth}{}>{body}</h{depth}>",
                    append_data_attrs(attrs, CrossReferenceKind::Section, &number, &text)
                );
            }
        }
        cursor = open.end;
    }
    out.push_str(&html[cursor..]);
    out
}

/// `1.2.1`, skipping levels that were never opened.
///
/// The empty-string case is a heading whose own counter is zero, which cannot
/// happen here — it is incremented immediately above — but the original fell
/// back to the raw counter, so this keeps that shape.
fn section_number(counters: &[usize; 6], depth: usize) -> String {
    let parts: Vec<String> =
        counters[..depth].iter().filter(|value| **value != 0).map(usize::to_string).collect();
    if parts.is_empty() { counters[depth - 1].to_string() } else { parts.join(".") }
}

struct Heading {
    depth: usize,
    start: usize,
    end: usize,
    attrs_start: usize,
    attrs_end: usize,
    body_start: usize,
    body_end: usize,
}

/// `<hN …>…</hN>` with matching N, non-greedy body.
fn find_heading(html: &str, from: usize) -> Option<Heading> {
    let bytes = html.as_bytes();
    let mut cursor = from;

    while let Some(open) = memchr::memchr(b'<', &bytes[cursor..]).map(|rel| cursor + rel) {
        cursor = open + 1;
        let Some(marker) = bytes.get(open + 1) else { break };
        if !marker.eq_ignore_ascii_case(&b'h') {
            continue;
        }
        let Some(digit) = bytes.get(open + 2).copied() else { continue };
        if !(b'1'..=b'6').contains(&digit) {
            continue;
        }
        // `\b` after the digit.
        if bytes.get(open + 3).is_some_and(|byte| is_word_byte(*byte)) {
            continue;
        }
        let Some(attrs_end) = html[open + 3..].find('>').map(|rel| open + 3 + rel) else {
            continue;
        };
        let close = format!("</h{}>", digit as char);
        let Some(close_at) = find_ci(html, attrs_end + 1, &close) else {
            continue;
        };
        return Some(Heading {
            depth: usize::from(digit - b'0'),
            start: open,
            end: close_at + close.len(),
            attrs_start: open + 3,
            attrs_end,
            body_start: attrs_end + 1,
            body_end: close_at,
        });
    }
    None
}

/// Numbers `<figure>` blocks and standalone `<img>` in one shared sequence.
pub(super) fn annotate_figures_and_images(
    html: &str,
    options: &CrossReferencesOptions,
    registry: &mut Registry,
) -> String {
    let mut count = 0usize;
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(block) = find_figure_or_image(html, cursor) {
        out.push_str(&html[cursor..block.start()]);
        match block {
            FigureBlock::Figure { start, end, attrs_start, attrs_end, body_start, body_end } => {
                let attrs = &html[attrs_start..attrs_end];
                let body = &html[body_start..body_end];
                let figure_id = read_attr(attrs, "id");
                // A figure without its own id borrows the first image's.
                let image = if figure_id.is_some() { None } else { find_first_image(body) };
                let id = figure_id.clone().or_else(|| image.as_ref().map(|img| img.id.clone()));

                match id.filter(|value| should_track_target(value)) {
                    None => out.push_str(&html[start..end]),
                    Some(id) => {
                        count += 1;
                        let number = count.to_string();
                        let text = format!("{} {}", options.labels.figure, number);
                        let title = figure_caption(body)
                            .or_else(|| image.as_ref().and_then(|img| img.alt.clone()));
                        registry.register(
                            options.duplicates,
                            TrackedTarget {
                                entry: CrossReferenceEntry {
                                    href: format!("#{}", escape_url_fragment(&id)),
                                    id,
                                    kind: CrossReferenceKind::Figure,
                                    number: number.clone(),
                                    label: options.labels.figure.clone(),
                                    text: text.clone(),
                                    title,
                                },
                                position: start,
                            },
                        );
                        match (figure_id, image) {
                            // The figure owns the id, so it takes the attributes.
                            (Some(_), _) => {
                                let _ = write!(
                                    out,
                                    "<figure{}>{body}</figure>",
                                    append_data_attrs(
                                        attrs,
                                        CrossReferenceKind::Figure,
                                        &number,
                                        &text
                                    )
                                );
                            }
                            // Otherwise they go on the image that supplied it.
                            (None, Some(image)) => {
                                let _ = write!(
                                    out,
                                    "<figure{attrs}>{}<img{}>{}</figure>",
                                    &body[..image.start],
                                    append_data_attrs(
                                        &image.attrs,
                                        CrossReferenceKind::Figure,
                                        &number,
                                        &text
                                    ),
                                    &body[image.end..]
                                );
                            }
                            (None, None) => out.push_str(&html[start..end]),
                        }
                    }
                }
                cursor = end;
            }
            FigureBlock::Image { start, end, attrs_start, attrs_end } => {
                let attrs = &html[attrs_start..attrs_end];
                // Already numbered as part of a figure on an earlier pass.
                let claimed = read_attr(attrs, "data-ox-xref-kind").is_some();
                let id = read_attr(attrs, "id").filter(|value| should_track_target(value));
                match (claimed, id) {
                    (false, Some(id)) => {
                        count += 1;
                        let number = count.to_string();
                        let text = format!("{} {}", options.labels.figure, number);
                        let title = read_attr(attrs, "alt");
                        registry.register(
                            options.duplicates,
                            TrackedTarget {
                                entry: CrossReferenceEntry {
                                    href: format!("#{}", escape_url_fragment(&id)),
                                    id,
                                    kind: CrossReferenceKind::Figure,
                                    number: number.clone(),
                                    label: options.labels.figure.clone(),
                                    text: text.clone(),
                                    title,
                                },
                                position: start,
                            },
                        );
                        let _ = write!(
                            out,
                            "<img{}>",
                            append_data_attrs(attrs, CrossReferenceKind::Figure, &number, &text)
                        );
                    }
                    _ => out.push_str(&html[start..end]),
                }
                cursor = end;
            }
        }
    }
    out.push_str(&html[cursor..]);
    out
}

enum FigureBlock {
    Figure {
        start: usize,
        end: usize,
        attrs_start: usize,
        attrs_end: usize,
        body_start: usize,
        body_end: usize,
    },
    Image {
        start: usize,
        end: usize,
        attrs_start: usize,
        attrs_end: usize,
    },
}

impl FigureBlock {
    fn start(&self) -> usize {
        match self {
            Self::Figure { start, .. } | Self::Image { start, .. } => *start,
        }
    }
}

/// Whichever of `<figure>` or `<img>` comes first.
///
/// One forward scan rather than a separate search for each name: a document
/// with images but no more figures would otherwise scan to the end looking for
/// `<figure` on every single image.
fn find_figure_or_image(html: &str, from: usize) -> Option<FigureBlock> {
    let bytes = html.as_bytes();
    let mut cursor = from;

    while let Some(rel) = memchr::memchr(b'<', &bytes[cursor..]) {
        let start = cursor + rel;
        cursor = start + 1;
        let rest = &html[start + 1..];

        if opens_tag(rest, "figure") {
            let attrs_start = start + "<figure".len();
            let Some(attrs_end) = html[attrs_start..].find('>').map(|at| attrs_start + at) else {
                continue;
            };
            let Some(close) = find_ci(html, attrs_end + 1, "</figure>") else {
                continue;
            };
            return Some(FigureBlock::Figure {
                start,
                end: close + "</figure>".len(),
                attrs_start,
                attrs_end,
                body_start: attrs_end + 1,
                body_end: close,
            });
        }
        if opens_tag(rest, "img") {
            let attrs_start = start + "<img".len();
            let Some(attrs_end) = html[attrs_start..].find('>').map(|at| attrs_start + at) else {
                continue;
            };
            return Some(FigureBlock::Image { start, end: attrs_end + 1, attrs_start, attrs_end });
        }
    }
    None
}

/// `name\b` at the start of `rest`, which is the text just past a `<`.
fn opens_tag(rest: &str, name: &str) -> bool {
    rest.len() >= name.len()
        && rest[..name.len()].eq_ignore_ascii_case(name)
        && !rest.as_bytes().get(name.len()).is_some_and(|byte| is_word_byte(*byte))
}

pub(super) struct OpenTag {
    pub start: usize,
    pub attrs_start: usize,
    /// Offset of the `>`.
    pub attrs_end: usize,
}

/// `<name\b[^>]*>` at or after `from`.
pub(super) fn find_tag(html: &str, from: usize, name: &str) -> Option<OpenTag> {
    let open = format!("<{name}");
    let mut cursor = from;
    while let Some(start) = find_ci(html, cursor, &open) {
        cursor = start + 1;
        let attrs_start = start + open.len();
        if html.as_bytes().get(attrs_start).is_some_and(|byte| is_word_byte(*byte)) {
            continue;
        }
        let Some(attrs_end) = html[attrs_start..].find('>').map(|rel| attrs_start + rel) else {
            continue;
        };
        return Some(OpenTag { start, attrs_start, attrs_end });
    }
    None
}

struct FoundImage {
    id: String,
    attrs: String,
    /// Offsets within the figure body.
    start: usize,
    end: usize,
    alt: Option<String>,
}

/// The first `<img>` in the body, but only if it carries an `id`.
fn find_first_image(body: &str) -> Option<FoundImage> {
    let open = find_tag(body, 0, "img")?;
    let attrs = &body[open.attrs_start..open.attrs_end];
    let id = read_attr(attrs, "id")?;
    Some(FoundImage {
        id,
        attrs: attrs.to_string(),
        start: open.start,
        end: open.attrs_end + 1,
        alt: read_attr(attrs, "alt"),
    })
}

fn figure_caption(body: &str) -> Option<String> {
    let open = find_tag(body, 0, "figcaption")?;
    let close = find_ci(body, open.attrs_end + 1, "</figcaption>")?;
    Some(text_content(&body[open.attrs_end + 1..close]))
}

/// Numbers `<table>` elements that carry a trackable `id`.
pub(super) fn annotate_tables(
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
pub(super) fn apply_trailing_table_labels(html: &str) -> String {
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
