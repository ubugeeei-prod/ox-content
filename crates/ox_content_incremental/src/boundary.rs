use memchr::memchr;

/// Returns the byte length of the stable prefix in an append-only Markdown buffer.
///
/// A blank-line boundary is committed only after a following unindented line is
/// observed. This keeps loose list continuations and fenced-code bodies in the
/// unstable tail while still letting the UI render that tail provisionally.
#[must_use]
pub fn stable_prefix_len(source: &str) -> usize {
    BoundaryScan::default().advance(source)
}

/// Resumable form of the [`stable_prefix_len`] scan.
///
/// A streaming caller asks for the stable prefix after every chunk, and the
/// answer depends on state that only accumulates: how far the scan got, the
/// fence it is inside, and where the last run of blank lines ended. Keeping
/// that state turns a whole-buffer rescan per chunk into one pass over the
/// stream. It matters most exactly where nothing commits — a long fenced
/// code block arriving in chunks — which is where the rescan was quadratic.
#[derive(Debug, Clone, Default)]
pub struct BoundaryScan {
    /// Bytes of the buffer already consumed as complete lines.
    pos: usize,
    fence: Option<(u8, usize)>,
    /// Offset just past the most recent run of blank lines, or `0`.
    blank_boundary: usize,
    /// Highest offset proven safe to commit.
    stable_boundary: usize,
}

impl BoundaryScan {
    /// Consumes whatever `source` has gained since the last call and returns
    /// the stable prefix length. `source` must be the same buffer, extended
    /// at the end (and rebased through [`Self::commit`] when it is drained).
    pub fn advance(&mut self, source: &str) -> usize {
        let bytes = source.as_bytes();
        while self.pos < bytes.len() {
            let Some(offset) = memchr(b'\n', &bytes[self.pos..]) else {
                break;
            };
            let line_end = self.pos + offset;
            let line = strip_cr(&source[self.pos..line_end]);
            self.step(line, line_end + 1);
            self.pos = line_end + 1;
        }

        // The buffer may end mid-line. Such a line can still prove a
        // boundary — an unindented character after a blank line does that
        // on its own — but it may grow into something else on the next
        // chunk, so score it without keeping the state it implies.
        if self.pos < bytes.len() {
            let mut tentative = self.clone();
            tentative.step(strip_cr(&source[self.pos..]), bytes.len());
            return tentative.stable_boundary;
        }
        self.stable_boundary
    }

    /// Rebases the scan after `len` bytes have been drained from the front.
    pub fn commit(&mut self, len: usize) {
        self.pos = self.pos.saturating_sub(len);
        self.blank_boundary = self.blank_boundary.saturating_sub(len);
        self.stable_boundary = self.stable_boundary.saturating_sub(len);
    }

    fn step(&mut self, line: &str, next_line: usize) {
        if let Some((fence_byte, fence_len)) = self.fence {
            if is_closing_fence(line, fence_byte, fence_len) {
                self.fence = None;
            }
            return;
        }
        if let Some(opening) = opening_fence(line) {
            self.fence = Some(opening);
            return;
        }
        if line.trim().is_empty() {
            self.blank_boundary = next_line;
            return;
        }
        if self.blank_boundary != 0 && indentation_columns(line) == 0 {
            self.stable_boundary = self.blank_boundary;
        }
        self.blank_boundary = 0;
    }
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
