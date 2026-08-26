//! Opt-in static `::: if` / `::: else` conditional blocks.
//!
//! Conditions are evaluated from transform config and page frontmatter only.
use crate::ConditionalBlockOptions;
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::hash::BuildHasher;

mod expr;
#[cfg(test)]
mod tests;
use expr::{EvalContext, evaluate};

#[derive(Clone, Debug)]
pub(super) struct ResolvedConditionalBlockOptions {
    values: FxHashMap<String, Value>,
}

pub(super) fn resolve(
    options: Option<&ConditionalBlockOptions>,
) -> Option<ResolvedConditionalBlockOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedConditionalBlockOptions { values: options.values.clone().unwrap_or_default() })
}

pub(super) fn transform<S: BuildHasher>(
    source: &str,
    options: &ResolvedConditionalBlockOptions,
    frontmatter: &HashMap<String, Value, S>,
    errors: &mut Vec<String>,
) -> String {
    if !source.contains(":::") || !source.contains("if") {
        return source.to_string();
    }
    let context = EvalContext { config: &options.values, frontmatter };
    rewrite(source, &context, errors)
}

fn rewrite<S: BuildHasher>(
    source: &str,
    context: &EvalContext<'_, S>,
    errors: &mut Vec<String>,
) -> String {
    let lines: Vec<SourceLine<'_>> = source.split_inclusive('\n').map(SourceLine::new).collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;

    while index < lines.len() {
        let SourceLine { line, ending } = lines[index];

        if in_fence {
            out.push_str(line);
            out.push_str(ending);
            if super::segments::is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            index += 1;
            continue;
        }

        if let Some(open) = super::segments::parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            out.push_str(line);
            out.push_str(ending);
            index += 1;
            continue;
        }

        let trimmed = trim_container_indent(line);
        if let Some(opener) = parse_if_opener(trimmed) {
            match collect_block(&lines, index, opener) {
                CollectedBlock::Closed { next, branches } => {
                    if let Some(body) = selected_body(&branches, context, errors) {
                        out.push_str(&rewrite(body, context, errors));
                    }
                    index = next;
                }
                CollectedBlock::Unclosed { original } => {
                    errors.push("Conditional block is missing a closing ::: fence.".to_string());
                    out.push_str(&original);
                    break;
                }
            }
            continue;
        }

        out.push_str(line);
        out.push_str(ending);
        index += 1;
    }

    out
}

fn collect_block(
    lines: &[SourceLine<'_>],
    start: usize,
    opener: BranchMarker<'_>,
) -> CollectedBlock {
    let mut branches = Vec::new();
    let mut current_kind = BranchKind::If;
    let mut current_expression = opener.expression.to_string();
    let mut body_start = start + 1;
    let mut cursor = body_start;

    loop {
        let Some(boundary) = find_boundary(lines, cursor, opener.colon_count) else {
            return CollectedBlock::Unclosed { original: collect_lines(lines, start, lines.len()) };
        };
        branches.push(Branch {
            kind: current_kind,
            expression: std::mem::take(&mut current_expression),
            body: collect_lines(lines, body_start, boundary.index),
        });
        match boundary.kind {
            BoundaryKind::Close => {
                return CollectedBlock::Closed { next: boundary.index + 1, branches };
            }
            BoundaryKind::Branch(marker) => {
                current_kind = marker.kind;
                current_expression = marker.expression.to_string();
                body_start = boundary.index + 1;
                cursor = body_start;
            }
        }
    }
}

fn find_boundary<'a>(
    lines: &'a [SourceLine<'a>],
    start: usize,
    outer_colons: usize,
) -> Option<Boundary<'a>> {
    let mut in_fence = false;
    let mut fence_char = b'\0';
    let mut fence_len = 0usize;
    let mut stack = vec![outer_colons];

    for (index, SourceLine { line, .. }) in lines.iter().enumerate().skip(start) {
        if in_fence {
            if super::segments::is_closing_fence(line, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        if let Some(open) = super::segments::parse_opening_fence(line) {
            in_fence = true;
            fence_char = open.fence_char;
            fence_len = open.fence_len;
            continue;
        }

        let trimmed = trim_container_indent(line);
        if stack.len() == 1 {
            if let Some(marker) = parse_branch_marker(trimmed)
                && marker.kind != BranchKind::If
                && marker.colon_count >= outer_colons
            {
                return Some(Boundary { index, kind: BoundaryKind::Branch(marker) });
            }
            if let Some(close) = parse_closer(trimmed)
                && close >= outer_colons
            {
                return Some(Boundary { index, kind: BoundaryKind::Close });
            }
        }

        if let Some(open) = parse_any_opener_for_nesting(trimmed) {
            stack.push(open);
            continue;
        }
        if let Some(close) = parse_closer(trimmed)
            && let Some(index) = stack.iter().rposition(|open| *open <= close)
        {
            stack.truncate(index);
        }
    }
    None
}

fn selected_body<'a>(
    branches: &'a [Branch],
    context: &EvalContext<'_, impl BuildHasher>,
    errors: &mut Vec<String>,
) -> Option<&'a str> {
    for branch in branches {
        match branch.kind {
            BranchKind::If | BranchKind::ElseIf => match evaluate(&branch.expression, context) {
                Ok(true) => return Some(&branch.body),
                Ok(false) => {}
                Err(error) => errors.push(format!(
                    "Conditional block expression `{}` could not be evaluated: {error}.",
                    branch.expression.trim()
                )),
            },
            BranchKind::Else => {
                return Some(&branch.body);
            }
        }
    }
    None
}

fn collect_lines(lines: &[SourceLine<'_>], start: usize, end: usize) -> String {
    let mut out = String::new();
    for SourceLine { line, ending } in &lines[start..end] {
        out.push_str(line);
        out.push_str(ending);
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Branch {
    kind: BranchKind,
    expression: String,
    body: String,
}

enum CollectedBlock {
    Closed { next: usize, branches: Vec<Branch> },
    Unclosed { original: String },
}

struct Boundary<'a> {
    index: usize,
    kind: BoundaryKind<'a>,
}

enum BoundaryKind<'a> {
    Branch(BranchMarker<'a>),
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BranchMarker<'a> {
    kind: BranchKind,
    expression: &'a str,
    colon_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BranchKind {
    If,
    ElseIf,
    Else,
}

#[derive(Clone, Copy)]
struct SourceLine<'a> {
    line: &'a str,
    ending: &'a str,
}

impl<'a> SourceLine<'a> {
    fn new(line_with_end: &'a str) -> Self {
        let (line, ending) = split_ending(line_with_end);
        Self { line, ending }
    }
}

fn parse_if_opener(line: &str) -> Option<BranchMarker<'_>> {
    let marker = parse_branch_marker(line)?;
    (marker.kind == BranchKind::If).then_some(marker)
}

fn parse_branch_marker(line: &str) -> Option<BranchMarker<'_>> {
    let (colon_count, rest) = split_directive(line)?;
    if let Some(expression) = strip_keyword(rest, "if") {
        return Some(BranchMarker { kind: BranchKind::If, expression, colon_count });
    }
    if let Some(expression) = strip_keyword(rest, "elif") {
        return Some(BranchMarker { kind: BranchKind::ElseIf, expression, colon_count });
    }
    let after_else = strip_keyword(rest, "else")?;
    if after_else.is_empty() {
        return Some(BranchMarker { kind: BranchKind::Else, expression: "", colon_count });
    }
    strip_keyword(after_else, "if").map(|expression| BranchMarker {
        kind: BranchKind::ElseIf,
        expression,
        colon_count,
    })
}

fn parse_any_opener_for_nesting(line: &str) -> Option<usize> {
    let (colon_count, rest) = split_directive(line)?;
    if rest.is_empty() || parse_closer(line).is_some() {
        return None;
    }
    if let Some(marker) = parse_branch_marker(line)
        && marker.kind != BranchKind::If
    {
        return None;
    }
    Some(colon_count)
}

fn parse_closer(line: &str) -> Option<usize> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    line[colon_count..].bytes().all(|byte| byte.is_ascii_whitespace()).then_some(colon_count)
}

fn split_directive(line: &str) -> Option<(usize, &str)> {
    let colon_count = line.bytes().take_while(|byte| *byte == b':').count();
    if colon_count < 3 {
        return None;
    }
    Some((colon_count, line[colon_count..].trim_start()))
}

fn strip_keyword<'a>(value: &'a str, keyword: &str) -> Option<&'a str> {
    if value == keyword {
        return Some("");
    }
    let rest = value.strip_prefix(keyword)?;
    rest.as_bytes().first().is_some_and(u8::is_ascii_whitespace).then(|| rest.trim_start())
}

fn split_ending(line_with_end: &str) -> (&str, &str) {
    if let Some(line) = line_with_end.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = line_with_end.strip_suffix('\n') {
        (line, "\n")
    } else {
        (line_with_end, "")
    }
}

fn trim_container_indent(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut indent = 0usize;
    while indent < bytes.len() && indent < 3 && bytes[indent] == b' ' {
        indent += 1;
    }
    &line[indent..]
}
