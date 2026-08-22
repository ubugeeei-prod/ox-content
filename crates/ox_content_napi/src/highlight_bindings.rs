//! Bindings for the tree-sitter syntax highlighter.

use napi_derive::napi;

/// Highlights one fenced code block, returning the full `<pre>` element.
///
/// Returns `null` when no grammar claims `lang`, which is the caller's signal
/// to fall back — either to another highlighter or to emitting the code
/// unhighlighted. Callers that want to decide before paying for the call can
/// ask [`supports_highlight_language`] instead.
#[napi(js_name = "highlightCodeBlock")]
pub fn highlight_code_block(code: String, lang: String) -> Option<String> {
    ox_content_highlight::highlight_to_html(&code, &lang)
}

/// Whether a fenced code block tagged `lang` will be highlighted natively.
#[napi(js_name = "supportsHighlightLanguage")]
pub fn supports_highlight_language(lang: String) -> bool {
    ox_content_highlight::supports(&lang)
}

/// Every language name and alias the native highlighter answers to.
#[napi(js_name = "nativeHighlightLanguages")]
pub fn native_highlight_languages() -> Vec<String> {
    ox_content_highlight::supported_languages().map(ToOwned::to_owned).collect()
}
