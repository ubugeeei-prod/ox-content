use napi_derive::napi;

/// Mermaid transform result.
#[napi(object)]
pub struct MermaidTransformResult {
    /// The transformed HTML with mermaid code blocks replaced by rendered SVGs.
    pub html: String,
    /// Non-fatal errors encountered during rendering (per-diagram).
    pub errors: Vec<String>,
}

impl From<ox_content_mermaid::MermaidTransformResult> for MermaidTransformResult {
    fn from(result: ox_content_mermaid::MermaidTransformResult) -> Self {
        Self { html: result.html, errors: result.errors }
    }
}

/// Transforms mermaid code blocks in HTML to rendered SVG diagrams.
#[napi]
pub fn transform_mermaid(html: String, mmdc_path: String) -> MermaidTransformResult {
    ox_content_mermaid::transform_mermaid(html, &mmdc_path).into()
}
