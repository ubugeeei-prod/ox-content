use memchr::memchr;

/// Returns the byte length of the stable prefix in an append-only Markdown buffer.
///
/// A blank-line boundary is committed only after a following unindented line is
/// observed. This keeps loose list continuations and fenced-code bodies in the
/// unstable tail while still letting the UI render that tail provisionally.
#[must_use]
pub fn stable_prefix_len(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut pos = 0usize;
    let mut blank_boundary = 0usize;
    let mut stable_boundary = 0usize;
    let mut fence: Option<(u8, usize)> = None;

    while pos < bytes.len() {
        let line_start = pos;
        let line_end =
            memchr(b'\n', &bytes[line_start..]).map_or(bytes.len(), |off| line_start + off);
        let next_line = if line_end < bytes.len() { line_end + 1 } else { line_end };
        let line = strip_cr(&source[line_start..line_end]);

        if let Some((fence_byte, fence_len)) = fence {
            if is_closing_fence(line, fence_byte, fence_len) {
                fence = None;
            }
            pos = next_line;
            continue;
        }

        if let Some(opening) = opening_fence(line) {
            fence = Some(opening);
            pos = next_line;
            continue;
        }

        if line.trim().is_empty() {
            blank_boundary = next_line;
            pos = next_line;
            continue;
        }

        if blank_boundary != 0 && indentation_columns(line) == 0 {
            stable_boundary = blank_boundary;
        }
        blank_boundary = 0;
        pos = next_line;
    }

    stable_boundary
}

pub fn opening_fence(line: &str) -> Option<(u8, usize)> {
    if indentation_columns(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let fence_byte = *bytes.first()?;
    if !matches!(fence_byte, b'`' | b'~') {
        return None;
    }
    let len = bytes.iter().take_while(|&&byte| byte == fence_byte).count();
    (len >= 3).then_some((fence_byte, len))
}

pub fn is_closing_fence(line: &str, fence_byte: u8, fence_len: usize) -> bool {
    if indentation_columns(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let len = bytes.iter().take_while(|&&byte| byte == fence_byte).count();
    len >= fence_len && bytes[len..].iter().all(|byte| matches!(byte, b' ' | b'\t'))
}

fn indentation_columns(line: &str) -> usize {
    let mut indent = 0usize;
    for byte in line.bytes() {
        match byte {
            b' ' => indent += 1,
            b'\t' => indent += 4,
            _ => break,
        }
    }
    indent
}

pub fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}
