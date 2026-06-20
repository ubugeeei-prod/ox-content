use napi_derive::napi;

use ox_content_transform::features;

/// Extracted fenced code block.
#[napi(object)]
#[derive(Clone)]
pub struct JsCodeBlock {
    pub language: String,
    pub meta: String,
    pub code: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Diagnostic emitted by code block linting.
#[napi(object)]
#[derive(Clone)]
pub struct JsCodeBlockDiagnostic {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub language: Option<String>,
}

impl From<features::ExtractedCodeBlock> for JsCodeBlock {
    fn from(block: features::ExtractedCodeBlock) -> Self {
        Self {
            language: block.language,
            meta: block.meta,
            code: block.code,
            start_line: block.start_line,
            end_line: block.end_line,
        }
    }
}

impl From<features::CodeBlockDiagnostic> for JsCodeBlockDiagnostic {
    fn from(diagnostic: features::CodeBlockDiagnostic) -> Self {
        Self {
            rule_id: diagnostic.rule_id,
            severity: diagnostic.severity,
            message: diagnostic.message,
            line: diagnostic.line,
            column: diagnostic.column,
            end_line: diagnostic.end_line,
            end_column: diagnostic.end_column,
            language: diagnostic.language,
        }
    }
}
