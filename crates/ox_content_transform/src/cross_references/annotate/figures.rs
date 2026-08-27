//! Numbering `<figure>` blocks and standalone `<img>` in one sequence.

use super::super::html::{
    append_data_attrs, escape_url_fragment, is_word_byte, read_attr, should_track_target,
    text_content,
};
use super::super::types::{CrossReferenceEntry, CrossReferenceKind, CrossReferencesOptions};
use super::{Registry, TrackedTarget};
use crate::html_scan::find_ci;
use std::fmt::Write as _;

pub fn annotate_figures_and_images(
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

pub struct OpenTag {
    pub start: usize,
    pub attrs_start: usize,
    /// Offset of the `>`.
    pub attrs_end: usize,
}

/// `<name\b[^>]*>` at or after `from`.
pub fn find_tag(html: &str, from: usize, name: &str) -> Option<OpenTag> {
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
