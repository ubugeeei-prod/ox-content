use ox_content_markdown_lint::MarkdownLintDiagnostic;
use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};

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
    pub diagnostics: Diagnostics<'a>,
}

pub(super) struct Diagnostics<'a> {
    pub entries: &'a [(usize, MarkdownLintDiagnostic)],
    pub labels: &'a [String],
}

impl Serialize for Diagnostics<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.entries.len()))?;
        for (index, diagnostic) in self.entries {
            sequence
                .serialize_element(&FileDiagnostic { file: &self.labels[*index], diagnostic })?;
        }
        sequence.end()
    }
}
