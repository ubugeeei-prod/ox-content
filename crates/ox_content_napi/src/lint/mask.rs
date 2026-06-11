use super::utils::*;
use super::*;

pub(super) fn create_skipped_line_mask(line: &str) -> String {
    " ".repeat(count_code_points(line))
}

pub(super) fn get_trailing_whitespace_length(line: &str) -> usize {
    line.as_bytes().iter().rev().take_while(|value| **value == b' ' || **value == b'\t').count()
}

pub(super) fn is_fence_close(line: &str, fence_char: char, fence_length: usize) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let bytes = trimmed.as_bytes();
    let fence_byte = fence_char as u8;

    if bytes.len() < fence_length || !bytes[..fence_length].iter().all(|value| *value == fence_byte)
    {
        return false;
    }

    bytes[fence_length..].iter().all(|value| *value == fence_byte)
}

pub(super) fn is_indented_code_block_line(line: &str) -> bool {
    line.starts_with('\t') || line.starts_with("    ")
}

pub(super) fn get_visible_text(text: &str) -> String {
    collapse_whitespace(&mask_markdown_line(text))
}

pub(super) fn mask_markdown_line(line: &str) -> String {
    let line_chars = line.chars().collect::<Vec<_>>();
    let mut chars = line_chars.clone();

    if let Some(list_prefix_pattern) = LIST_PREFIX_PATTERN.as_ref() {
        if let Some(prefix_match) = list_prefix_pattern.find(line) {
            let (start, end) =
                byte_range_to_char_range(line, prefix_match.start(), prefix_match.end());
            blank_range(&mut chars, start, end);
        }
    }

    if let Some(footnote_pattern) = FOOTNOTE_PATTERN.as_ref() {
        for value in footnote_pattern.find_iter(line) {
            let (start, end) = byte_range_to_char_range(line, value.start(), value.end());
            blank_range(&mut chars, start, end);
        }
    }

    if let Some(url_pattern) = URL_PATTERN.as_ref() {
        for value in url_pattern.find_iter(line) {
            let (start, end) = byte_range_to_char_range(line, value.start(), value.end());
            blank_range(&mut chars, start, end);
        }
    }

    if let Some(html_tag_pattern) = HTML_TAG_PATTERN.as_ref() {
        for value in html_tag_pattern.find_iter(line) {
            let (start, end) = byte_range_to_char_range(line, value.start(), value.end());
            blank_range(&mut chars, start, end);
        }
    }

    mask_inline_code(&line_chars, &mut chars);
    mask_link_targets(&line_chars, &mut chars);

    for value in &mut chars {
        if matches!(*value, '\\' | '*' | '_' | '~' | '|' | '!' | '[' | ']' | '(' | ')') {
            *value = ' ';
        }
    }

    chars.into_iter().collect()
}

fn mask_inline_code(line_chars: &[char], chars: &mut [char]) {
    let mut index = 0;

    while index < line_chars.len() {
        if line_chars[index] != '`' {
            index += 1;
            continue;
        }

        let mut tick_count = 1;
        while index + tick_count < line_chars.len() && line_chars[index + tick_count] == '`' {
            tick_count += 1;
        }

        if let Some(close_index) = find_backtick_fence(line_chars, index + tick_count, tick_count) {
            blank_range(chars, index, close_index + tick_count);
            index = close_index + tick_count;
        } else {
            blank_range(chars, index, index + tick_count);
            index += tick_count;
        }
    }
}

fn mask_link_targets(line_chars: &[char], chars: &mut [char]) {
    let mut index = 0;

    while index + 1 < line_chars.len() {
        if line_chars[index] != ']' {
            index += 1;
            continue;
        }

        if line_chars[index + 1] == '(' {
            let mut depth = 1_i32;
            let mut cursor = index + 2;
            while cursor < line_chars.len() && depth > 0 {
                if line_chars[cursor] == '(' {
                    depth += 1;
                } else if line_chars[cursor] == ')' {
                    depth -= 1;
                }
                cursor += 1;
            }
            blank_range(chars, index, cursor);
            index = cursor;
            continue;
        }

        if line_chars[index + 1] == '[' {
            let mut cursor = index + 2;
            while cursor < line_chars.len() {
                if line_chars[cursor] == ']' {
                    blank_range(chars, index, cursor + 1);
                    index = cursor + 1;
                    break;
                }
                cursor += 1;
            }
        } else {
            index += 1;
        }
    }
}

fn find_backtick_fence(chars: &[char], start: usize, tick_count: usize) -> Option<usize> {
    let mut index = start;

    while index + tick_count <= chars.len() {
        if chars[index..index + tick_count].iter().all(|value| *value == '`') {
            return Some(index);
        }
        index += 1;
    }

    None
}

fn blank_range(chars: &mut [char], start: usize, end: usize) {
    let safe_start = start.min(chars.len());
    let safe_end = end.min(chars.len());
    for value in chars.iter_mut().take(safe_end).skip(safe_start) {
        *value = ' ';
    }
}

fn byte_range_to_char_range(text: &str, start: usize, end: usize) -> (usize, usize) {
    (byte_to_char_index(text, start), byte_to_char_index(text, end))
}
