use napi_derive::napi;

use crate::{JsHeadDiagnostic, JsSsgHtmlResult};

pub(super) fn convert_head_validation(value: Option<String>) -> ox_content_ssg::HeadValidation {
    match value.as_deref().map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case("warn") => ox_content_ssg::HeadValidation::Warn,
        Some(value) if value.eq_ignore_ascii_case("strict") => {
            ox_content_ssg::HeadValidation::Strict
        }
        _ => ox_content_ssg::HeadValidation::Off,
    }
}

/// Resolve Unhead-compatible page-head descriptors to HTML.
#[napi(js_name = "renderHead")]
pub fn render_head(input_json: String) -> napi::Result<JsSsgHtmlResult> {
    let input: ox_content_ssg::HeadInput = serde_json::from_str(&input_json)
        .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    let rendered = ox_content_ssg::render_head(&input);
    Ok(JsSsgHtmlResult {
        html: rendered.html,
        diagnostics: rendered
            .diagnostics
            .into_iter()
            .map(|d| JsHeadDiagnostic { strict: d.strict, message: d.message })
            .collect(),
    })
}
