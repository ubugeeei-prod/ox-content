/// Most block content is ASCII at its edges. Trim those bytes directly and
/// use Unicode trimming only when a non-ASCII edge might also be whitespace.
pub(super) fn trim_block_content(content: &str) -> &str {
    let bytes = content.as_bytes();
    let mut start = 0;
    while start < bytes.len() && matches!(bytes[start], b' ' | b'\t'..=b'\r') {
        start += 1;
    }
    let mut end = bytes.len();
    while end > start && matches!(bytes[end - 1], b' ' | b'\t'..=b'\r') {
        end -= 1;
    }
    let content = &content[start..end];
    if content.as_bytes().first().is_some_and(|byte| !byte.is_ascii())
        || content.as_bytes().last().is_some_and(|byte| !byte.is_ascii())
    {
        content.trim()
    } else {
        content
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_macros)]

    use super::trim_block_content;

    #[test]
    fn block_trim_keeps_unicode_whitespace_behavior() {
        for edge in (0..=127)
            .filter_map(char::from_u32)
            .chain(['\u{a0}', '\u{1680}', '\u{2003}', '\u{3000}'])
        {
            for input in
                [format!("{edge}text"), format!("text{edge}"), format!(" \n{edge}text{edge}\r ")]
            {
                assert_eq!(trim_block_content(&input), input.trim(), "{input:?}");
            }
        }
    }
}
