use super::{emoji, ResolvedEmojiShortcodeOptions};

pub(super) fn replace_emoji_shortcodes(
    segment: &str,
    options: &ResolvedEmojiShortcodeOptions,
    out: &mut String,
) {
    let bytes = segment.as_bytes();
    let mut cursor = 0usize;
    while let Some(relative) = memchr::memchr(b':', &bytes[cursor..]) {
        let start = cursor + relative;
        out.push_str(&segment[cursor..start]);
        let name_start = start + 1;
        let mut name_end = name_start;
        while name_end < bytes.len() && emoji::is_shortcode_byte(bytes[name_end]) {
            name_end += 1;
        }
        if name_end == name_start || bytes.get(name_end) != Some(&b':') {
            out.push(':');
            cursor = name_start;
            continue;
        }
        let name = &segment[name_start..name_end];
        if let Some(value) = options.custom.get(name) {
            out.push_str(value);
        } else if let Some(value) = emoji::lookup(name) {
            out.push_str(value);
        } else {
            out.push_str(&segment[start..name_end + 1]);
        }
        cursor = name_end + 1;
    }
    out.push_str(&segment[cursor..]);
}
