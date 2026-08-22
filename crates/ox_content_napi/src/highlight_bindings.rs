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

/// Result of highlighting every code block in a rendered document.
#[napi(object)]
pub struct JsHighlightedDocument {
    /// The document with each handled block replaced.
    pub html: String,
    /// Languages of elements the native pass could not read, which means the
    /// caller's own highlighter has to produce the whole page. Non-empty
    /// leaves `html` untouched and `pending` empty.
    pub skipped: Vec<String>,
    /// Well-formed blocks whose language has no native grammar, in document
    /// order. Highlight each one and hand the results to
    /// `applyPendingHighlights`; they are still in `html`, unchanged.
    pub pending: Vec<JsPendingBlock>,
}

/// A block the native pass left for the caller's highlighter.
#[napi(object)]
pub struct JsPendingBlock {
    /// The `language-…` class the block carries.
    pub language: String,
    /// The block's source text, already unescaped.
    pub source: String,
}

/// Highlights every code block in a rendered document in one call.
///
/// The alternative is walking the page through an HTML parser and serializer
/// to find the blocks and splice results back, which on the documentation
/// corpus costs an order of magnitude more than the highlighting itself.
#[napi(js_name = "highlightHtmlCodeBlocks")]
pub fn highlight_html_code_blocks(html: String) -> JsHighlightedDocument {
    let result = ox_content_transform::highlight::highlight_code_blocks(
        &html,
        ox_content_highlight::supports,
        ox_content_highlight::highlight_to_html,
    );
    JsHighlightedDocument {
        html: result.html,
        skipped: result.skipped,
        pending: result
            .pending
            .into_iter()
            .map(|block| JsPendingBlock { language: block.language, source: block.source })
            .collect(),
    }
}

/// Splices the caller's highlighting back over the blocks
/// `highlightHtmlCodeBlocks` left pending.
///
/// `replacements` lines up with the `pending` list it returned: entry `i` is a
/// full `<pre>` element for pending block `i`, or an empty string to leave that
/// block as it is. This keeps a page that needs one exotic grammar off the
/// HTML round trip, instead of surrendering the whole document for it.
#[napi(js_name = "applyPendingHighlights")]
pub fn apply_pending_highlights(html: String, replacements: Vec<String>) -> String {
    ox_content_transform::highlight::apply_pending_highlights(
        &html,
        &replacements,
        ox_content_highlight::supports,
    )
}
