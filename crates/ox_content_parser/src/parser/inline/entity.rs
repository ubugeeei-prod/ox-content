//! Entity and numeric character references (CommonMark).
//!
//! Named references use the HTML5 entity table (`entities.txt`, generated
//! from the WHATWG list; semicolon-terminated forms only, as CommonMark
//! requires). Numeric references decode `&#N;` and `&#xH;`, mapping
//! U+0000 and invalid code points to U+FFFD. A decoded reference behaves
//! as literal text — it can never open or close markup.

use std::sync::OnceLock;

use rustc_hash::FxHashMap;

/// A successfully scanned reference at the start of the input.
pub(in crate::parser) enum EntityValue {
    /// Expansion straight from the named entity table.
    Named(&'static str),
    /// Code point from a numeric reference.
    Char(char),
}

/// Scans an entity or numeric character reference at the start of
/// `rest` (which must begin with `&`). Returns the decoded value and the
/// number of bytes consumed including the `&` and `;`.
pub(in crate::parser) fn scan_entity(rest: &str) -> Option<(EntityValue, usize)> {
    let bytes = rest.as_bytes();
    debug_assert_eq!(bytes.first(), Some(&b'&'));

    if bytes.get(1) == Some(&b'#') {
        return scan_numeric(rest);
    }

    // Longest HTML5 entity name is 31 characters.
    let mut end = 1;
    while end < bytes.len().min(33) && bytes[end].is_ascii_alphanumeric() {
        end += 1;
    }
    if end == 1 || bytes.get(end) != Some(&b';') {
        return None;
    }
    let expansion = entities().get(&rest[1..end])?;
    Some((EntityValue::Named(expansion), end + 1))
}

fn scan_numeric(rest: &str) -> Option<(EntityValue, usize)> {
    let bytes = rest.as_bytes();
    let (digits_start, radix, max_digits) = match bytes.get(2) {
        Some(b'x' | b'X') => (3, 16, 6),
        _ => (2, 10, 7),
    };

    let mut end = digits_start;
    while end < bytes.len() && (bytes[end] as char).is_digit(radix) {
        end += 1;
    }
    let digit_count = end - digits_start;
    if digit_count == 0 || digit_count > max_digits || bytes.get(end) != Some(&b';') {
        return None;
    }

    let code = u32::from_str_radix(&rest[digits_start..end], radix).unwrap_or(u32::MAX);
    // The spec maps U+0000, surrogates, and out-of-range references to
    // the replacement character.
    let ch = char::from_u32(code).filter(|&c| c != '\0').unwrap_or('\u{fffd}');
    Some((EntityValue::Char(ch), end + 1))
}

fn entities() -> &'static FxHashMap<&'static str, &'static str> {
    static ENTITIES: OnceLock<FxHashMap<&'static str, &'static str>> = OnceLock::new();
    ENTITIES.get_or_init(|| {
        let mut map = FxHashMap::default();
        map.reserve(2200);
        for line in include_str!("entities.txt").lines() {
            if let Some((name, expansion)) = line.split_once('\t') {
                map.insert(name, expansion);
            }
        }
        map
    })
}

#[cfg(test)]
mod tests {
    use super::{scan_entity, EntityValue};

    #[test]
    fn scans_named_numeric_and_invalid_references() {
        match scan_entity("&amp; rest") {
            Some((EntityValue::Named("&"), 5)) => {}
            _ => panic!("named entity should decode"),
        }
        match scan_entity("&ngE;") {
            Some((EntityValue::Named("\u{2267}\u{338}"), 5)) => {}
            _ => panic!("multi-char entity should decode"),
        }
        match scan_entity("&#35;") {
            Some((EntityValue::Char('#'), 5)) => {}
            _ => panic!("decimal reference should decode"),
        }
        match scan_entity("&#X22;") {
            Some((EntityValue::Char('"'), 6)) => {}
            _ => panic!("hex reference should decode"),
        }
        match scan_entity("&#0;") {
            Some((EntityValue::Char('\u{fffd}'), 4)) => {}
            _ => panic!("nul reference maps to replacement char"),
        }
        assert!(scan_entity("&MadeUpEntity;").is_none());
        assert!(scan_entity("&;").is_none());
        assert!(scan_entity("&#;").is_none());
        assert!(scan_entity("&#12345678;").is_none(), "8 digits exceed the limit");
        assert!(scan_entity("&amp rest").is_none(), "missing semicolon");
    }
}
