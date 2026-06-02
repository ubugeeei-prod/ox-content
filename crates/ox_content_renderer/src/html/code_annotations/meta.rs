//! Common metadata utilities for annotated code blocks.
//!
//! This module normalizes a fence's language and metadata before either annotation
//! syntax reads it. The token splitter preserves bracketed and braced VitePress
//! sections as independent tokens so later stages do not need to rescan raw strings.

use std::collections::BTreeMap;

use compact_str::CompactString;
use smallvec::SmallVec;

use super::state::{
    CodeAnnotationKind, CodeLineRenderState, MetaToken, MetaTokenKind, NormalizedCodeBlockInfo,
    PendingCodeAnnotation,
};

pub(in crate::html) fn split_code_block_meta(meta: &str) -> SmallVec<[MetaToken<'_>; 4]> {
    // Split the metadata once into borrowed tokens. VitePress braces/brackets
    // and raw `:line-numbers` tokens are then consumed by later stages without
    // repeatedly rescanning the original string or allocating token strings.
    let bytes = meta.as_bytes();
    let mut index = 0;
    let mut tokens = SmallVec::new();

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
    // Some authors put VitePress metadata directly in the language token
    // (`js{1}` or `ts:line-numbers`). Return borrowed slices so normalization
    // can merge this inline metadata with the separate fence meta field.
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

pub(in crate::html) fn normalize_code_block_info(
    lang: Option<&str>,
    meta: Option<&str>,
) -> NormalizedCodeBlockInfo {
    let mut meta_parts: SmallVec<[&str; 2]> = SmallVec::new();
    let mut language = None;

    if let Some(raw_lang) = lang.map(str::trim).filter(|value| !value.is_empty()) {
        let (normalized_lang, inline_meta) = split_code_block_language_token(raw_lang);
        if !normalized_lang.is_empty() {
            language = Some(CompactString::from(normalized_lang));
        }
        if !inline_meta.trim().is_empty() {
            meta_parts.push(inline_meta.trim());
        }
    }

    if let Some(raw_meta) = meta.map(str::trim).filter(|value| !value.is_empty()) {
        meta_parts.push(raw_meta);
    }

    let meta = if meta_parts.is_empty() {
        CompactString::default()
    } else {
        CompactString::from(meta_parts.join(" "))
    };

    NormalizedCodeBlockInfo { language, meta }
}

pub(in crate::html) fn normalize_code_block_language(lang: Option<&str>) -> Option<&str> {
    let raw_lang = lang.map(str::trim).filter(|value| !value.is_empty())?;
    let (language, _) = split_code_block_language_token(raw_lang);
    let language = language.trim();

    if language.is_empty() {
        None
    } else {
        Some(language)
    }
}

pub(in crate::html) fn apply_annotation_numbers(
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

pub(in crate::html) fn apply_btree_annotations(
    lines: &mut [CodeLineRenderState],
    annotations: &BTreeMap<usize, SmallVec<[CodeAnnotationKind; 2]>>,
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

pub(in crate::html) fn apply_pending_annotations(
    line: &mut CodeLineRenderState,
    pending_annotations: &mut SmallVec<[PendingCodeAnnotation; 2]>,
) {
    // Standalone VitePress directives annotate following lines. Drain the
    // current pending list into the line and rebuild only the entries whose
    // remaining count extends past this line, keeping the list tiny and
    // stack-backed in normal use.
    let mut remaining = SmallVec::new();

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

pub(in crate::html) fn parse_annotation_count(value: &str) -> usize {
    value.trim().parse::<usize>().ok().filter(|count| *count > 0).unwrap_or(1)
}
