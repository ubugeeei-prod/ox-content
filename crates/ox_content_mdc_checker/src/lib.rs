use serde::Serialize;

pub mod registry;

pub use registry::{Attribute, Component, Registry, RegistryError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
}

#[derive(Debug, Clone)]
struct OpenTag {
    name: String,
    name_start: usize,
    name_end: usize,
}

#[must_use]
pub fn check(source: &str) -> Vec<Diagnostic> {
    let line_index = LineIndex::new(source);
    let masked = masked_fence_ranges(source);
    let mut diagnostics = Vec::new();
    let mut stack: Vec<OpenTag> = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'<' || is_masked(index, &masked) {
            index += 1;
            continue;
        }

        let Some(tag) = parse_tag(source, index) else {
            index += 1;
            continue;
        };

        if tag.name.starts_with(|ch: char| ch.is_ascii_lowercase()) {
            index = tag.end;
            continue;
        }

        diagnostics.extend(check_attributes(source, &line_index, &tag));

        if tag.closing {
            match stack.pop() {
                Some(open) if open.name == tag.name => {}
                Some(open) => {
                    diagnostics.push(diagnostic(
                        &line_index,
                        tag.name_start,
                        tag.name_end,
                        format!("MDC closing tag </{}> does not match <{}>.", tag.name, open.name),
                        Some(tag.name),
                    ));
                    stack.push(open);
                }
                None => diagnostics.push(diagnostic(
                    &line_index,
                    tag.name_start,
                    tag.name_end,
                    format!("MDC closing tag </{}> has no matching opening tag.", tag.name),
                    Some(tag.name),
                )),
            }
        } else if !tag.self_closing {
            stack.push(OpenTag {
                name: tag.name,
                name_start: tag.name_start,
                name_end: tag.name_end,
            });
        }

        index = tag.end;
    }

    for open in stack.into_iter().rev() {
        diagnostics.push(diagnostic(
            &line_index,
            open.name_start,
            open.name_end,
            format!("MDC component <{}> is missing a closing tag.", open.name),
            Some(open.name),
        ));
    }

    diagnostics
}

#[derive(Debug)]
struct ParsedTag {
    name: String,
    end: usize,
    name_start: usize,
    name_end: usize,
    attributes_start: usize,
    attributes_end: usize,
    closing: bool,
    self_closing: bool,
}

fn parse_tag(source: &str, start: usize) -> Option<ParsedTag> {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    let closing = bytes.get(cursor) == Some(&b'/');
    if closing {
        cursor += 1;
    }

    let name_start = cursor;
    while let Some(byte) = bytes.get(cursor) {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-') {
            cursor += 1;
        } else {
            break;
        }
    }

    if cursor == name_start {
        return None;
    }

    let name_end = cursor;
    let name = source[name_start..name_end].to_string();
    let attributes_start = cursor;
    let mut quote: Option<u8> = None;
    let mut brace_depth = 0u32;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(quote_byte) = quote {
            if byte == quote_byte {
                quote = None;
            }
            cursor += 1;
            continue;
        }

        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'{' => brace_depth += 1,
            b'}' => brace_depth = brace_depth.saturating_sub(1),
            b'>' if brace_depth == 0 => {
                let attributes_end = cursor;
                let self_closing = source[start..cursor].trim_end().ends_with('/');
                return Some(ParsedTag {
                    name,
                    end: cursor + 1,
                    name_start,
                    name_end,
                    attributes_start,
                    attributes_end,
                    closing,
                    self_closing,
                });
            }
            b'\n' if quote.is_none() && brace_depth == 0 => return None,
            _ => {}
        }
        cursor += 1;
    }

    None
}

fn check_attributes(source: &str, line_index: &LineIndex, tag: &ParsedTag) -> Vec<Diagnostic> {
    if tag.closing {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let mut cursor = tag.attributes_start;
    let end = tag.attributes_end;
    let bytes = source.as_bytes();

    while cursor < end {
        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end || bytes[cursor] == b'/' {
            break;
        }

        let attr_start = cursor;
        while cursor < end
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(bytes[cursor], b':' | b'_' | b'-'))
        {
            cursor += 1;
        }
        if cursor == attr_start {
            cursor += 1;
            continue;
        }

        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end || bytes[cursor] != b'=' {
            continue;
        }

        cursor += 1;
        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end || !matches!(bytes[cursor], b'"' | b'\'' | b'{') {
            diagnostics.push(diagnostic(
                line_index,
                attr_start,
                cursor.max(attr_start + 1),
                "MDC prop values must be quoted strings or brace expressions.".to_string(),
                Some(tag.name.clone()),
            ));
            continue;
        }

        cursor = skip_attribute_value(bytes, cursor, end);
    }

    diagnostics
}

fn skip_attribute_value(bytes: &[u8], start: usize, end: usize) -> usize {
    let first = bytes[start];
    if first == b'"' || first == b'\'' {
        let mut cursor = start + 1;
        while cursor < end {
            if bytes[cursor] == first {
                return cursor + 1;
            }
            cursor += 1;
        }
        return cursor;
    }

    let mut cursor = start + 1;
    let mut depth = 1u32;
    while cursor < end {
        match bytes[cursor] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return cursor + 1;
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    cursor
}

fn masked_fence_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    let mut fence: Option<(u8, usize)> = None;

    for line in source.split_inclusive('\n') {
        let line_start = cursor;
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent <= 3 && (trimmed.starts_with("```") || trimmed.starts_with("~~~")) {
            let marker = trimmed.as_bytes()[0];
            if let Some((open_marker, start)) = fence {
                if marker == open_marker {
                    ranges.push((start, line_start + line.len()));
                    fence = None;
                }
            } else {
                fence = Some((marker, line_start));
            }
        }
        cursor += line.len();
    }

    if let Some((_, start)) = fence {
        ranges.push((start, source.len()));
    }

    ranges
}

fn is_masked(offset: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(start, end)| offset >= *start && offset < *end)
}

fn diagnostic(
    line_index: &LineIndex,
    start: usize,
    end: usize,
    message: String,
    component: Option<String>,
) -> Diagnostic {
    let (line, column) = line_index.position(start);
    let (end_line, end_column) = line_index.position(end);
    Diagnostic { severity: Severity::Error, message, line, column, end_line, end_column, component }
}

struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (index, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    fn position(&self, offset: usize) -> (u32, u32) {
        let line = self.starts.partition_point(|start| *start <= offset).saturating_sub(1);
        let column = offset.saturating_sub(self.starts[line]);
        (line as u32 + 1, column as u32 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn accepts_balanced_components_and_quoted_props() {
        let diagnostics = check("<Alert tone=\"info\" :count={count}>Hello</Alert>\n<Card />");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn reports_mismatched_and_unquoted_component_props() {
        let diagnostics = check("<Alert tone=info><Panel></Alert>");
        let messages: Vec<&str> =
            diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect();

        assert!(messages.iter().any(|message| message.contains("quoted strings")));
        assert!(messages.iter().any(|message| message.contains("does not match")));
        assert!(messages.iter().any(|message| message.contains("missing a closing tag")));
    }

    #[test]
    fn ignores_component_like_text_inside_fences() {
        let diagnostics = check("```mdc\n<Alert tone=bad>\n```\n<Alert />");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
}
