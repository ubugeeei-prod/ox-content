use memchr::memchr;

/// Returns true when `haystack` contains a case-insensitive `</tag` opener.
///
/// Type-1 HTML blocks close on the first line containing their closing tag.
/// Searching for `<` with `memchr` skips the common case of long text/code
/// lines that contain no tag-looking byte at all.
pub(in crate::parser) fn ascii_contains_closing_tag(haystack: &str, tag: &[u8]) -> bool {
    let bytes = haystack.as_bytes();
    if bytes.len() < tag.len() + 2 {
        return false;
    }
    let mut search_start = 0;
    while let Some(off) = memchr(b'<', &bytes[search_start..]) {
        let i = search_start + off;
        if i + tag.len() + 2 <= bytes.len() && bytes[i + 1] == b'/' {
            let candidate = &bytes[i + 2..i + 2 + tag.len()];
            if candidate.eq_ignore_ascii_case(tag) {
                return true;
            }
        }
        search_start = i + 1;
    }
    false
}
