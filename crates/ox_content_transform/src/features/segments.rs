use compact_str::CompactString;

pub(super) fn transform_markdown_text_segments(
    source: &str,
    mut transform: impl FnMut(&str, &mut String),
) -> Option<String> {
    let mut out = String::with_capacity(source.len());
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

        let before_len = out.len();
        transform_inline_code_segments(line, &mut out, &mut transform);
        let appended = &out[before_len..];
        if appended != line {
            changed = true;
        }
        out.push_str(ending);
    }

    changed.then_some(out)
}

fn transform_inline_code_segments(
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
