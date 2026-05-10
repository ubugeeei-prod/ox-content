// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Low-level scanner helpers for turning a raw block comment into logical
//! content lines.
//!
//! Verbatim port of `crates/ox_jsdoc/src/parser/scanner.rs`. The scanner
//! does not depend on the AST representation, so the binary parser shares
//! identical logic with the typed-AST parser.

/// One source line after stripping the comment prefix syntax.
///
/// Kept small (24 bytes) so that `Copy` in hot loops is cheap; doubling
/// to 32 bytes regresses parse_batch by ~30 %, so any new field has to
/// either replace an existing slot or live in the parallel `MarginInfo`
/// array.
///
/// The `is_content_empty` flag is mirrored here (rather than only in
/// `MarginInfo`) because it is read by callers that iterate only the
/// lines array (`normalize_lines`, `partition_sections`); we save 4
/// bytes by deriving the line-end offset from `content_start +
/// content.len()` instead of storing it.
#[derive(Debug, Clone, Copy)]
pub struct LogicalLine<'a> {
    /// Content after removing the visual JSDoc margin.
    pub content: &'a str,
    /// Absolute byte offset where `content` starts.
    pub content_start: u32,
    /// `true` when `content` contains only ASCII space/tab (or is empty).
    /// Mirrored from [`MarginInfo::is_content_empty`] so consumers iterating
    /// only the lines array can shortcut whitespace-only checks.
    pub is_content_empty: bool,
}

impl<'a> LogicalLine<'a> {
    /// Absolute byte offset where the original physical line ends. Equal to
    /// `content_start + content.len()` because `content` is a sub-slice of
    /// `raw_line` that runs to the line's end (the scanner does not trim
    /// trailing whitespace from `content`).
    #[inline]
    #[must_use]
    pub fn content_end(&self) -> u32 {
        self.content_start + self.content.len() as u32
    }
}

/// Source-preserving margin metadata for one logical line.
#[derive(Debug, Clone, Copy)]
pub struct MarginInfo<'a> {
    /// Indentation before the `*` delimiter (spaces/tabs).
    pub initial: &'a str,
    /// The `*` delimiter itself, or `""` if absent.
    pub delimiter: &'a str,
    /// Whitespace after the `*` delimiter (at most one space/tab), or `""`.
    pub post_delimiter: &'a str,
    /// Line ending characters (`"\n"`, `"\r\n"`, or `""`).
    pub line_end: &'a str,
    /// Whether content (after margin stripping) is empty or whitespace-only.
    pub is_content_empty: bool,
}

/// Result of `logical_lines()`: parallel arrays of content and margin info.
pub struct ScanResult<'a> {
    /// Content portion of each logical line.
    pub lines: Vec<LogicalLine<'a>>,
    /// Margin metadata for each logical line.
    pub margins: Vec<MarginInfo<'a>>,
}

/// JSDoc blocks must start with `/**`; plain `/*` comments are rejected.
#[must_use]
pub fn is_jsdoc_block(source_text: &str) -> bool {
    source_text.trim_start().starts_with("/**")
}

/// The parser currently accepts only complete block comments.
#[must_use]
pub fn has_closing_block(source_text: &str) -> bool {
    source_text.trim_end().ends_with("*/")
}

/// Return the byte range between the opening `/**` and closing `*/`.
#[must_use]
pub fn body_range(source_text: &str) -> Option<(usize, usize)> {
    if !is_jsdoc_block(source_text) || !has_closing_block(source_text) || source_text.len() < 5 {
        return None;
    }
    let leading = source_text.len() - source_text.trim_start().len();
    let trailing = source_text.len() - source_text.trim_end().len();
    Some((leading + 3, source_text.len() - trailing - 2))
}

/// Split the comment body into content lines with parallel margin metadata.
///
/// Hot path on parse phase (~44% of `parse_block_into_data` on the
/// typescript-checker.ts fixture per `examples/profile_parse_block.rs`).
/// The implementation:
///
/// - uses `memchr` for both the line-count estimate and the per-iteration
///   newline search (SIMD-accelerated on every target memchr supports),
/// - converts `(raw_start + …) -> u32` via `as u32` instead of
///   `u32::try_from(…).unwrap()` to drop the panic check (the encoder's
///   precondition is that source offsets fit in u32 — see `format.md`).
#[must_use]
pub fn logical_lines(source_text: &str, base_offset: u32) -> ScanResult<'_> {
    let Some((body_start, body_end)) = body_range(source_text) else {
        return ScanResult { lines: Vec::new(), margins: Vec::new() };
    };

    let body = &source_text[body_start..body_end];
    let body_bytes = body.as_bytes();
    let body_len = body_bytes.len();

    // Estimate line count from newline density. memchr's SIMD-accelerated
    // iterator beats the generic `iter().filter().count()` lowering on the
    // x86_64 / ARM64 targets we ship.
    let estimated_lines = memchr::memchr_iter(b'\n', body_bytes).count() + 1;
    let mut lines = Vec::with_capacity(estimated_lines);
    let mut margins = Vec::with_capacity(estimated_lines);
    let mut cursor = 0usize;

    loop {
        // memchr from `cursor` — single SIMD scan for the next '\n'.
        let line_end = match memchr::memchr(b'\n', &body_bytes[cursor..]) {
            Some(off) => cursor + off,
            None => body_len,
        };
        let raw_line = &body[cursor..line_end];
        let raw_start = body_start + cursor;

        let mut pos = 0usize;
        let bytes = raw_line.as_bytes();

        let initial_start = pos;
        while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
            pos += 1;
        }
        let initial = &raw_line[initial_start..pos];

        let delimiter_start = pos;
        if bytes.get(pos) == Some(&b'*') {
            pos += 1;
        }
        let delimiter = &raw_line[delimiter_start..pos];

        let post_delim_start = pos;
        if !delimiter.is_empty() && matches!(bytes.get(pos), Some(b' ' | b'\t')) {
            pos += 1;
        }
        let post_delimiter = &raw_line[post_delim_start..pos];

        let content_start = pos;
        let content = &raw_line[content_start..];

        let is_content_empty = content.bytes().all(|b| b == b' ' || b == b'\t');

        let line_end_str = if line_end < body_len {
            if line_end > 0 && body_bytes[line_end - 1] == b'\r' { "\r\n" } else { "\n" }
        } else {
            ""
        };

        // Source offsets fit in u32 by encoder precondition; skip the
        // `try_from(...).unwrap()` panic check.
        let absolute_content_start = base_offset + (raw_start + content_start) as u32;

        lines.push(LogicalLine {
            content,
            content_start: absolute_content_start,
            is_content_empty,
        });
        margins.push(MarginInfo {
            initial,
            delimiter,
            post_delimiter,
            line_end: line_end_str,
            is_content_empty,
        });

        if line_end == body_len {
            break;
        }
        cursor = line_end + 1;
    }

    ScanResult { lines, margins }
}

#[cfg(test)]
mod tests {
    use super::{body_range, has_closing_block, is_jsdoc_block, logical_lines};

    #[test]
    fn recognizes_only_closed_jsdoc_blocks() {
        assert!(is_jsdoc_block("/** ok */"));
        assert!(!is_jsdoc_block("/* plain */"));
        assert!(has_closing_block("/** ok */"));
        assert!(!has_closing_block("/** unclosed"));
        assert_eq!(body_range("/** ok */"), Some((3, 7)));
        assert_eq!(body_range("/* plain */"), None);
    }

    #[test]
    fn strips_jsdoc_margin_and_keeps_absolute_offsets() {
        let source = "/**\n * Find a user.\n * @param {string} id\n */";
        let result = logical_lines(source, 100);
        assert_eq!(result.lines.len(), 4);
        assert_eq!(result.lines[1].content, "Find a user.");
        assert_eq!(result.lines[1].content_start, 107);
        assert_eq!(result.margins[1].delimiter, "*");
    }
}
