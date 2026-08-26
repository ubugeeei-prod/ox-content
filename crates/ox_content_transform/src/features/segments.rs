use compact_str::CompactString;

pub(crate) fn transform_markdown_text_segments(
    source: &str,
    mut transform: impl FnMut(&str, &mut String),
) -> Option<String> {
    let mut out = String::with_capacity(source.len());
    let mut changed = false;
    let mut in_fence = false;
    let mut in_html_comment = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = match line_with_end.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line_with_end, ""),
        };

        if in_html_comment {
            let before_len = out.len();
            in_html_comment = continue_html_comment(line, &mut out, &mut transform);
            if &out[before_len..] != line {
                changed = true;
            }
            out.push_str(ending);
            continue;
        }

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
                fence_char = b'\0';
                fence_len = 0;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        if is_indented_code_line(line) {
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        let before_len = out.len();
        in_html_comment =
            transform_inline_code_and_comment_segments(line, &mut out, &mut transform);
        if &out[before_len..] != line {
            changed = true;
        }
        out.push_str(ending);
    }

    changed.then_some(out)
}

/// Like [`transform_markdown_text_segments`], but feeds consecutive prose
/// (including newlines) to `transform` as one chunk so delimiters can span
/// lines. Fenced and indented code still pass through unchanged.
pub(crate) fn transform_markdown_prose_segments(
    source: &str,
    mut transform: impl FnMut(&str, &mut String),
) -> Option<String> {
    let mut out = String::with_capacity(source.len());
    let mut prose = String::new();
    let mut changed = false;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    for line_with_end in source.split_inclusive('\n') {
        let (line, ending) = match line_with_end.strip_suffix('\n') {
            Some(line) => (line, "\n"),
            None => (line_with_end, ""),
        };

        if in_fence {
            changed |= flush_prose(&mut prose, &mut out, &mut transform);
            out.push_str(line);
            out.push_str(ending);
            if is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
                fence_char = b'\0';
                fence_len = 0;
            }
            continue;
        }

        if let Some(open) = parse_opening_fence(line) {
            changed |= flush_prose(&mut prose, &mut out, &mut transform);
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        if is_indented_code_line(line) {
            changed |= flush_prose(&mut prose, &mut out, &mut transform);
            out.push_str(line);
            out.push_str(ending);
            continue;
        }

        prose.push_str(line);
        prose.push_str(ending);
    }

    changed |= flush_prose(&mut prose, &mut out, &mut transform);
    changed.then_some(out)
}

fn flush_prose(
    prose: &mut String,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) -> bool {
    if prose.is_empty() {
        return false;
    }
    let before_len = out.len();
    transform_inline_code_segments(prose, out, transform);
    let changed = &out[before_len..] != prose.as_str();
    prose.clear();
    changed
}

fn continue_html_comment(
    line: &str,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) -> bool {
    if let Some(close) = line.find("-->") {
        out.push_str(&line[..close + 3]);
        return transform_inline_code_and_comment_segments(&line[close + 3..], out, transform);
    }
    out.push_str(line);
    true
}

/// Like [`transform_inline_code_segments`], but also leaves HTML comments
/// (including close/reopen on one line) untouched. Returns whether `line`
/// ended inside an unclosed `<!--` comment.
fn transform_inline_code_and_comment_segments(
    line: &str,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) -> bool {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let tick = memchr::memchr(b'`', &bytes[cursor..]).map(|rel| cursor + rel);
        let comment = line[cursor..].find("<!--").map(|rel| cursor + rel);
        match (tick, comment) {
            (None, None) => {
                transform(&line[cursor..], out);
                return false;
            }
            (Some(tick_start), Some(comment_start)) if tick_start < comment_start => {
                emit_inline_code(line, bytes, tick_start, &mut cursor, out, transform);
            }
            (Some(_), Some(comment_start)) | (None, Some(comment_start)) => {
                transform(&line[cursor..comment_start], out);
                let after_open = comment_start + 4;
                if let Some(close) = line[after_open..].find("-->") {
                    out.push_str(&line[comment_start..after_open + close + 3]);
                    cursor = after_open + close + 3;
                } else {
                    out.push_str(&line[comment_start..]);
                    return true;
                }
            }
            (Some(tick_start), None) => {
                emit_inline_code(line, bytes, tick_start, &mut cursor, out, transform);
            }
        }
    }
    false
}

fn emit_inline_code(
    line: &str,
    bytes: &[u8],
    tick_start: usize,
    cursor: &mut usize,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) {
    transform(&line[*cursor..tick_start], out);
    let tick_count = count_repeated_byte(bytes, tick_start, b'`');
    let code_start = tick_start + tick_count;
    if let Some(close) = find_closing_backticks(bytes, code_start, tick_count) {
        out.push_str(&line[tick_start..close + tick_count]);
        *cursor = close + tick_count;
    } else {
        out.push_str(&line[tick_start..]);
        *cursor = bytes.len();
    }
}

pub(super) fn transform_inline_code_segments(
    line: &str,
    out: &mut String,
    transform: &mut impl FnMut(&str, &mut String),
) {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let Some(relative) = memchr::memchr(b'`', &bytes[cursor..]) else {
            transform(&line[cursor..], out);
            return;
        };
        let tick_start = cursor + relative;
        transform(&line[cursor..tick_start], out);
        let tick_count = count_repeated_byte(bytes, tick_start, b'`');
        let code_start = tick_start + tick_count;
        if let Some(close) = find_closing_backticks(bytes, code_start, tick_count) {
            out.push_str(&line[tick_start..close + tick_count]);
            cursor = close + tick_count;
        } else {
            out.push_str(&line[tick_start..]);
            return;
        }
    }
}

pub(super) struct FenceOpen {
    pub(super) fence_char: u8,
    pub(super) fence_len: usize,
    pub(super) language: CompactString,
    pub(super) meta: CompactString,
}

pub(super) fn parse_opening_fence(line: &str) -> Option<FenceOpen> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let fence_char = *bytes.first()?;
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }
    let fence_len = count_repeated_byte(bytes, 0, fence_char);
    if fence_len < 3 {
        return None;
    }
    let rest = trimmed[fence_len..].trim();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let language = CompactString::from(parts.next().unwrap_or_default());
    let meta = CompactString::from(parts.next().unwrap_or_default().trim());
    Some(FenceOpen { fence_char, fence_len, language, meta })
}

pub(super) fn is_closing_fence(line: &str, fence_char: u8, fence_len: usize) -> bool {
    let trimmed = line.trim();
    let bytes = trimmed.as_bytes();
    bytes.len() >= fence_len
        && bytes[..fence_len].iter().all(|value| *value == fence_char)
        && bytes[fence_len..].iter().all(|value| *value == fence_char)
}

pub(super) fn is_indented_code_line(line: &str) -> bool {
    line.starts_with('\t') || line.starts_with("    ")
}

fn count_repeated_byte(bytes: &[u8], start: usize, byte: u8) -> usize {
    let mut count = 0usize;
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] == byte {
        count += 1;
        cursor += 1;
    }
    count
}

fn find_closing_backticks(bytes: &[u8], from: usize, count: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < bytes.len() {
        let relative = memchr::memchr(b'`', &bytes[cursor..])?;
        let start = cursor + relative;
        if count_repeated_byte(bytes, start, b'`') >= count {
            return Some(start);
        }
        cursor = start + 1;
    }
    None
}
