use napi_derive::napi;

/// Options for appending to an incremental parser.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsIncrementalParseOptions {
    /// Treat this append as the final stream chunk.
    pub is_final: Option<bool>,
    /// Include a provisional AST for the current unstable tail.
    pub include_pending_ast: Option<bool>,
    /// Temporarily close unmatched inline delimiters in the provisional AST.
    pub complete_inline: Option<bool>,
}

/// Options for appending to an incremental renderer.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsIncrementalRenderOptions {
    /// Treat this append as the final stream chunk.
    pub is_final: Option<bool>,
    /// Render the current unstable tail as replaceable provisional HTML.
    pub render_pending: Option<bool>,
    /// Temporarily close unmatched inline delimiters in provisional HTML.
    pub complete_inline: Option<bool>,
}

/// Incremental parser append result.
#[napi(object)]
pub struct IncrementalMarkdownParseResult {
    pub ast: String,
    pub pending_ast: String,
    pub markdown: String,
    pub pending_markdown: String,
    pub committed_byte_start: u32,
    pub committed_byte_end: u32,
    pub committed_bytes: u32,
    pub pending_bytes: u32,
    pub total_bytes: u32,
    pub did_commit: bool,
    pub is_final: bool,
    pub errors: Vec<String>,
}

/// Incremental renderer append result.
#[napi(object)]
pub struct IncrementalMarkdownRenderResult {
    pub delta_html: String,
    pub committed_html: String,
    pub pending_html: String,
    pub html: String,
    pub markdown: String,
    pub pending_markdown: String,
    pub committed_byte_start: u32,
    pub committed_byte_end: u32,
    pub committed_bytes: u32,
    pub pending_bytes: u32,
    pub total_bytes: u32,
    pub did_commit: bool,
    pub is_final: bool,
    pub errors: Vec<String>,
}
