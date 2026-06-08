use crate::incremental_types::{IncrementalMarkdownParseResult, IncrementalMarkdownRenderResult};

pub fn map_incremental_render_result(
    result: ox_content_incremental::IncrementalRenderResult,
    errors: Vec<String>,
) -> IncrementalMarkdownRenderResult {
    IncrementalMarkdownRenderResult {
        delta_html: result.delta_html,
        committed_html: result.committed_html,
        pending_html: result.pending_html,
        html: result.html,
        markdown: result.committed_markdown,
        pending_markdown: result.pending_markdown,
        committed_byte_start: usize_to_u32(result.committed_byte_start),
        committed_byte_end: usize_to_u32(result.committed_byte_end),
        committed_bytes: usize_to_u32(result.committed_bytes),
        pending_bytes: usize_to_u32(result.pending_bytes),
        total_bytes: usize_to_u32(result.total_bytes),
        did_commit: result.did_commit,
        is_final: result.is_final,
        errors,
    }
}

pub fn incremental_parse_error_result(
    pending_markdown: String,
    committed_bytes: usize,
    total_bytes: usize,
    is_final: bool,
    error: String,
) -> IncrementalMarkdownParseResult {
    IncrementalMarkdownParseResult {
        ast: String::new(),
        pending_ast: String::new(),
        markdown: String::new(),
        pending_markdown,
        committed_byte_start: usize_to_u32(committed_bytes),
        committed_byte_end: usize_to_u32(committed_bytes),
        committed_bytes: 0,
        pending_bytes: usize_to_u32(total_bytes.saturating_sub(committed_bytes)),
        total_bytes: usize_to_u32(total_bytes),
        did_commit: false,
        is_final,
        errors: vec![error],
    }
}

pub fn incremental_render_error_result(
    committed_html: String,
    pending_markdown: String,
    error: String,
) -> IncrementalMarkdownRenderResult {
    IncrementalMarkdownRenderResult {
        delta_html: String::new(),
        committed_html: committed_html.clone(),
        pending_html: String::new(),
        html: committed_html,
        markdown: String::new(),
        pending_markdown: pending_markdown.clone(),
        committed_byte_start: 0,
        committed_byte_end: 0,
        committed_bytes: 0,
        pending_bytes: usize_to_u32(pending_markdown.len()),
        total_bytes: 0,
        did_commit: false,
        is_final: false,
        errors: vec![error],
    }
}

pub fn usize_to_u32(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}
