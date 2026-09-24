//! HTML escaping for step Markdown that leaves code spans untouched.
//!
//! Code span content is already escaped by the renderer, so escaping it here
//! would produce `&amp;gt;` for `>`.

use super::super::escape_html_text;

pub(super) fn escape_markdown_text(text: &str, out: &mut String) {
    let bytes = text.as_bytes();
    let mut plain_start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' if bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation) => index += 2,
            b'`' => {
                let run = backtick_run(bytes, index);
                let Some(close) = find_closing_run(bytes, index + run, run) else {
                    index += run;
                    continue;
                };
                escape_html_text(&text[plain_start..index], out);
                out.push_str(&text[index..close + run]);
                index = close + run;
                plain_start = index;
            }
            _ => index += 1,
        }
    }
    escape_html_text(&text[plain_start..], out);
}

fn backtick_run(bytes: &[u8], start: usize) -> usize {
    bytes[start..].iter().take_while(|byte| **byte == b'`').count()
}

/// Finds a backtick run of exactly `len` bytes, stopping at a blank line
/// because code spans cannot cross paragraph boundaries.
fn find_closing_run(bytes: &[u8], start: usize, len: usize) -> Option<usize> {
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'`' => {
                let run = backtick_run(bytes, index);
                if run == len {
                    return Some(index);
                }
                index += run;
            }
            b'\n' if starts_blank_line(&bytes[index + 1..]) => return None,
            _ => index += 1,
        }
    }
    None
}

fn starts_blank_line(rest: &[u8]) -> bool {
    rest.iter()
        .find(|byte| **byte != b' ' && **byte != b'\t')
        .is_none_or(|byte| *byte == b'\n' || *byte == b'\r')
}
