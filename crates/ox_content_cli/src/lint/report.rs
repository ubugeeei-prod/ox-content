use ox_content_markdown_lint::MarkdownLintDiagnostic;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct FileDiagnostic<'a> {
    pub file: &'a str,
    #[serde(flatten)]
    pub diagnostic: &'a MarkdownLintDiagnostic,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Report<'a> {
    pub checked_file_count: usize,
    pub error_count: u32,
    pub warning_count: u32,
    pub fixed_count: u32,
    pub duration_ms: f64,
    pub diagnostics: Vec<FileDiagnostic<'a>>,
}
