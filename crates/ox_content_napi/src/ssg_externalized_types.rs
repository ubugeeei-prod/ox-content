use napi_derive::napi;

use crate::{JsHeadDiagnostic, JsSsgPageData, JsSsgSharedAsset};

/// One page request for streamed built-in theme generation and extraction.
#[napi(object)]
pub struct JsSsgExternalizedHtmlPageRequest {
    /// Source Markdown path retained in the generated page result.
    pub input_path: String,
    /// Destination HTML path retained in the generated page result.
    pub output_path: String,
    /// Page data consumed by the built-in theme renderer.
    pub page_data: JsSsgPageData,
}

/// One compact page returned by streamed generation and extraction.
#[napi(object)]
pub struct JsSsgExternalizedHtmlPage {
    pub input_path: String,
    pub output_path: String,
    pub html: String,
    pub diagnostics: Vec<JsHeadDiagnostic>,
}

/// Streamed built-in theme generation result.
#[napi(object)]
pub struct JsSsgGeneratedExternalizedAssets {
    /// HTML pages after inline theme assets have been replaced.
    pub pages: Vec<JsSsgExternalizedHtmlPage>,
    /// Unique CSS and JavaScript chunks shared by those pages.
    pub assets: Vec<JsSsgSharedAsset>,
}
