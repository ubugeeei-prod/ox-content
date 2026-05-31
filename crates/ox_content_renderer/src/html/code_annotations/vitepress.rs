//! VitePress-compatible code annotation parser.
//!
//! VitePress supports both fence metadata such as `{1,3}` and inline directives such
//! as `// [!code ++]`. This module owns the inline directive grammar and converts it
//! into the same semantic line states used by ox-content's attribute syntax.

use super::meta::{apply_pending_annotations, parse_annotation_count};
use super::state::{
    CodeAnnotationKind, CodeLineRenderState, InlineDirectiveAction, ParsedInlineDirective,
    PendingCodeAnnotation,
};

fn parse_vitepress_directive_action(value: &str) -> Option<InlineDirectiveAction> {
    let trimmed = value.trim();

    if matches!(trimmed, "escape" | "ignore" | "no-annotate") {
        return Some(InlineDirectiveAction::EscapeNextLine);
    }

    if trimmed == "++" {
        return Some(InlineDirectiveAction::Annotate { kind: CodeAnnotationKind::Add, count: 1 });
    }

    if trimmed == "--" {
        return Some(InlineDirectiveAction::Annotate {
            kind: CodeAnnotationKind::Remove,
            count: 1,
        });
    }

    if let Some((kind, count)) = trimmed.split_once(':') {
        let parsed_kind = match kind.trim() {
            "highlight" => CodeAnnotationKind::Highlight,
            "focus" => CodeAnnotationKind::Focus,
            "warning" => CodeAnnotationKind::Warning,
            "error" => CodeAnnotationKind::Error,
            _ => return None,
        };
        return Some(InlineDirectiveAction::Annotate {
            kind: parsed_kind,
            count: parse_annotation_count(count),
        });
    }

    let kind = match trimmed {
        "highlight" => Some(CodeAnnotationKind::Highlight),
        "warning" => Some(CodeAnnotationKind::Warning),
        "error" => Some(CodeAnnotationKind::Error),
        "focus" => Some(CodeAnnotationKind::Focus),
        _ => None,
    }?;

    Some(InlineDirectiveAction::Annotate { kind, count: 1 })
}

fn parse_vitepress_inline_directive(line: &str) -> Option<ParsedInlineDirective> {
    let marker_start = line.find("[!code ")?;
    let directive_start = marker_start + "[!code ".len();
    let marker_end = line[directive_start..].find(']')? + directive_start;
    let directive = &line[directive_start..marker_end];

    let before_marker = &line[..marker_start];
    let after_marker = &line[marker_end + 1..];
    let trimmed_before = before_marker.trim_end();

    let (comment_start, requires_closer) = if trimmed_before.ends_with("//") {
        (trimmed_before.len() - 2, false)
    } else if trimmed_before.ends_with('#') {
        (trimmed_before.len() - 1, false)
    } else if trimmed_before.ends_with("<!--") {
        (trimmed_before.len() - 4, true)
    } else if trimmed_before.ends_with("/*") {
        (trimmed_before.len() - 2, true)
    } else {
        return None;
    };

    let trailing = after_marker.trim();
    if requires_closer && trailing != "-->" && trailing != "*/" {
        return None;
    }
    if !requires_closer && !trailing.is_empty() {
        return None;
    }

    let stripped_line = before_marker[..comment_start].trim_end().to_string();
    let standalone = stripped_line.trim().is_empty();
    let action = parse_vitepress_directive_action(directive)?;
    if matches!(action, InlineDirectiveAction::EscapeNextLine) && !standalone {
        return None;
    }

    Some(ParsedInlineDirective { action, stripped_line, standalone })
}

pub(in crate::html) fn parse_vitepress_inline_annotations(value: &str) -> Vec<CodeLineRenderState> {
    let mut lines = Vec::new();
    let mut pending_annotations: Vec<PendingCodeAnnotation> = Vec::new();
    let mut escape_next_line = false;

    for raw_line in value.split('\n') {
        if escape_next_line {
            lines
                .push(CodeLineRenderState { value: raw_line.to_string(), annotations: Vec::new() });
            escape_next_line = false;
            continue;
        }

        if let Some(directive) = parse_vitepress_inline_directive(raw_line) {
            match directive.action {
                InlineDirectiveAction::EscapeNextLine => {
                    escape_next_line = true;
                    continue;
                }
                InlineDirectiveAction::Annotate { kind, count } => {
                    if directive.standalone {
                        pending_annotations.push(PendingCodeAnnotation { kind, remaining: count });
                        continue;
                    }

                    let mut line = CodeLineRenderState {
                        value: directive.stripped_line,
                        annotations: Vec::new(),
                    };
                    apply_pending_annotations(&mut line, &mut pending_annotations);
                    if !line.annotations.contains(&kind) {
                        line.annotations.push(kind);
                    }
                    if count > 1 {
                        pending_annotations
                            .push(PendingCodeAnnotation { kind, remaining: count - 1 });
                    }
                    lines.push(line);
                    continue;
                }
            }
        }

        let mut line = CodeLineRenderState { value: raw_line.to_string(), annotations: Vec::new() };
        apply_pending_annotations(&mut line, &mut pending_annotations);
        lines.push(line);
    }

    lines
}
