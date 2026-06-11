use std::borrow::Cow;

use compact_str::CompactString;
use memchr::memchr;
use ox_content_parser::ParserOptions;

use crate::boundary::{is_closing_fence, opening_fence, strip_cr};

/// Builds provisional Markdown by temporarily closing unmatched inline delimiters.
#[must_use]
pub fn complete_provisional_markdown<'a>(source: &'a str, options: &ParserOptions) -> Cow<'a, str> {
    let closers = provisional_inline_closers(source, options);
    if closers.is_empty() {
        Cow::Borrowed(source)
    } else {
        let mut completed = String::with_capacity(source.len() + closers.len());
        completed.push_str(source);
        completed.push_str(&closers);
        Cow::Owned(completed)
    }
}

fn provisional_inline_closers(source: &str, options: &ParserOptions) -> String {
    let mut stack = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut in_fence: Option<(u8, usize)> = None;
    let mut line_start = true;

    while i < bytes.len() {
        if line_start {
            let line_end = memchr(b'\n', &bytes[i..]).map_or(bytes.len(), |off| i + off);
            let line = strip_cr(&source[i..line_end]);
            if let Some((fence_byte, fence_len)) = in_fence {
                if is_closing_fence(line, fence_byte, fence_len) {
                    in_fence = None;
                }
            } else if let Some(opening) = opening_fence(line) {
                in_fence = Some(opening);
            }
        }

        let byte = bytes[i];
        if byte == b'\n' {
            line_start = true;
            i += 1;
            continue;
        }
        line_start = false;

        if in_fence.is_some() {
            i += 1;
            continue;
        }

        if byte == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }

        if byte == b'`' {
            let run = count_run(bytes, i, b'`');
            toggle_delimiter(&mut stack, InlineDelimiter::Code(run));
            i += run;
            continue;
        }

        if stack.last().is_some_and(|delimiter| matches!(delimiter, InlineDelimiter::Code(_))) {
            i += 1;
            continue;
        }

        if options.strikethrough && byte == b'~' && bytes.get(i + 1) == Some(&b'~') {
            toggle_delimiter(&mut stack, InlineDelimiter::Delete);
            i += 2;
            continue;
        }

        if matches!(byte, b'*' | b'_') {
            let run = count_run(bytes, i, byte);
            let mut remaining = run;
            while remaining >= 2 {
                toggle_delimiter(&mut stack, InlineDelimiter::Strong(byte));
                remaining -= 2;
            }
            if remaining == 1 {
                toggle_delimiter(&mut stack, InlineDelimiter::Emphasis(byte));
            }
            i += run;
            continue;
        }

        i += 1;
    }

    inline_closers(stack)
}

fn inline_closers(mut stack: Vec<InlineDelimiter>) -> String {
    let mut closers = CompactString::default();
    while let Some(delimiter) = stack.pop() {
        match delimiter {
            InlineDelimiter::Strong(b'*') => closers.push_str("**"),
            InlineDelimiter::Strong(b'_') => closers.push_str("__"),
            InlineDelimiter::Strong(_) => {}
            InlineDelimiter::Emphasis(b'*') => closers.push('*'),
            InlineDelimiter::Emphasis(b'_') => closers.push('_'),
            InlineDelimiter::Emphasis(_) => {}
            InlineDelimiter::Delete => closers.push_str("~~"),
            InlineDelimiter::Code(count) => {
                for _ in 0..count {
                    closers.push('`');
                }
            }
        }
    }
    closers.into_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineDelimiter {
    Strong(u8),
    Emphasis(u8),
    Delete,
    Code(usize),
}

fn toggle_delimiter(stack: &mut Vec<InlineDelimiter>, delimiter: InlineDelimiter) {
    if stack.last() == Some(&delimiter) {
        stack.pop();
    } else {
        stack.push(delimiter);
    }
}

fn count_run(bytes: &[u8], start: usize, byte: u8) -> usize {
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] == byte {
        cursor += 1;
    }
    cursor - start
}
