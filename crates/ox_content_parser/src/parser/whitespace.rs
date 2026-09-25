/// Check the usual ASCII blank line without Unicode decoding. A non-ASCII
/// byte falls back to the previous Unicode-aware behavior.
#[inline]
pub(super) fn is_blank(value: &str) -> bool {
    for &byte in value.as_bytes() {
        if matches!(byte, b' ' | b'\t'..=b'\r') {
            continue;
        }
        return !byte.is_ascii() && value.trim().is_empty();
    }
    true
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_macros)]

    use super::is_blank;

    #[test]
    fn blank_check_keeps_unicode_whitespace_behavior() {
        for edge in (0..=127)
            .filter_map(char::from_u32)
            .chain(['\u{a0}', '\u{1680}', '\u{2003}', '\u{3000}'])
        {
            for input in [format!("{edge}"), format!(" \n{edge}\r "), format!("{edge}text")] {
                assert_eq!(is_blank(&input), input.trim().is_empty(), "{input:?}");
            }
        }
    }
}
