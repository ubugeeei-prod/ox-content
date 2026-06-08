/// Result returned by [`crate::IncrementalParser`] after appending input.
#[derive(Debug)]
pub struct IncrementalParseResult<T> {
    /// Value produced from the newly committed Markdown prefix.
    pub committed: Option<T>,
    /// Markdown source for the newly committed prefix.
    pub committed_markdown: String,
    /// Absolute byte offset where this commit starts in the full stream.
    pub committed_byte_start: usize,
    /// Absolute byte offset where this commit ends in the full stream.
    pub committed_byte_end: usize,
    /// Number of bytes committed by this append call.
    pub committed_bytes: usize,
    /// Markdown still held as an unstable tail.
    pub pending_markdown: String,
    /// Number of pending bytes.
    pub pending_bytes: usize,
    /// Total stream bytes observed so far.
    pub total_bytes: usize,
    /// Whether this append call committed a prefix.
    pub did_commit: bool,
    /// Whether the stream has been finalized.
    pub is_final: bool,
}

impl<T> IncrementalParseResult<T> {
    pub(crate) fn empty(
        pending_markdown: String,
        committed_byte_start: usize,
        pending_bytes: usize,
        total_bytes: usize,
        is_final: bool,
    ) -> Self {
        Self {
            committed: None,
            committed_markdown: String::new(),
            committed_byte_start,
            committed_byte_end: committed_byte_start,
            committed_bytes: 0,
            pending_markdown,
            pending_bytes,
            total_bytes,
            did_commit: false,
            is_final,
        }
    }
}

/// Options for incremental HTML rendering.
#[derive(Debug, Clone, Copy)]
pub struct IncrementalRenderOptions {
    /// Treat the current append as the final stream chunk.
    pub is_final: bool,
    /// Render the current unstable tail as replaceable provisional HTML.
    pub render_pending: bool,
    /// Temporarily close unmatched inline delimiters when rendering pending HTML.
    pub complete_inline: bool,
}

impl Default for IncrementalRenderOptions {
    fn default() -> Self {
        Self { is_final: false, render_pending: true, complete_inline: true }
    }
}

/// Result returned by [`crate::IncrementalHtmlRenderer`] after appending input.
#[derive(Debug, Clone)]
pub struct IncrementalRenderResult {
    /// Newly committed HTML. Append this to a stable DOM region.
    pub delta_html: String,
    /// All committed HTML so far.
    pub committed_html: String,
    /// Replaceable HTML for the current unstable Markdown tail.
    pub pending_html: String,
    /// Current full HTML snapshot: `committed_html + pending_html`.
    pub html: String,
    /// Markdown source for the newly committed prefix.
    pub committed_markdown: String,
    /// Markdown still held as an unstable tail.
    pub pending_markdown: String,
    /// Absolute byte offset where this commit starts in the full stream.
    pub committed_byte_start: usize,
    /// Absolute byte offset where this commit ends in the full stream.
    pub committed_byte_end: usize,
    /// Number of bytes committed by this append call.
    pub committed_bytes: usize,
    /// Number of pending bytes.
    pub pending_bytes: usize,
    /// Total stream bytes observed so far.
    pub total_bytes: usize,
    /// Whether this append call committed a prefix.
    pub did_commit: bool,
    /// Whether the stream has been finalized.
    pub is_final: bool,
}
