//! Parser for ox-content `annotate="..."` code fence metadata.
//!
//! The attribute syntax accepts semicolon-separated `kind:line-list` pairs, where the
//! line list can contain single line numbers and inclusive ranges. It intentionally
//! stays independent from VitePress metadata so users can opt into a stable native
//! grammar without changing their code fence language token.

use std::collections::BTreeMap;

use smallvec::SmallVec;

use super::state::CodeAnnotationKind;

pub(in crate::html) fn parse_code_annotations(
    meta: &str,
    key: &str,
) -> BTreeMap<usize, SmallVec<[CodeAnnotationKind; 2]>> {
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

pub(in crate::html) fn parse_line_numbers(value: &str) -> SmallVec<[usize; 4]> {
    let mut line_numbers = SmallVec::new();

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
    annotations: &mut BTreeMap<usize, SmallVec<[CodeAnnotationKind; 2]>>,
    line_number: usize,
    kind: CodeAnnotationKind,
) {
    let kinds = annotations.entry(line_number).or_default();
    if !kinds.contains(&kind) {
        kinds.push(kind);
    }
}
