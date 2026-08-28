//! The two halves of a transform, so a JavaScript `transformers` hook can
//! rewrite the tree in between.

use napi_derive::napi;

use ox_content_transform::transformer::MarkdownTransformer;

use crate::TransformResult;

use super::transform_options::JsTransformOptions;

/// A parsed document on its way to a JavaScript `transformers` hook.
#[napi(object)]
pub struct JsMdastTransformResult {
    /// The tree, as mdast JSON.
    pub ast_json: String,
    /// Frontmatter as JSON, which the tree has no room for.
    pub frontmatter: String,
    /// Preprocessing and parse errors collected so far.
    pub errors: Vec<String>,
}

/// Runs a transform up to the point where the tree exists.
///
/// The counterpart to `transformFromMdast`: frontmatter is parsed and the
/// opt-in Markdown features are expanded, then the tree is handed over as
/// JSON for a `transformers` hook to rewrite.
#[napi(js_name = "transformMdast")]
pub fn transform_mdast(
    source: String,
    options: Option<JsTransformOptions>,
) -> JsMdastTransformResult {
    crate::ffi::recover(
        || {
            let core_options = options.unwrap_or_default().into();
            let result =
                MarkdownTransformer::from_options(&core_options).transform_mdast_json(&source);
            JsMdastTransformResult {
                ast_json: result.ast_json,
                frontmatter: result.frontmatter,
                errors: result.errors,
            }
        },
        || JsMdastTransformResult {
            ast_json: String::new(),
            frontmatter: "{}".to_string(),
            errors: vec![crate::ffi::UNEXPECTED_PANIC.to_string()],
        },
    )
}

/// Finishes a transform from an mdast a JavaScript `transformers` hook may
/// have rewritten.
///
/// `transformMdastRaw` produces the tree; this renders it and runs
/// everything that follows rendering — HTML postprocessing, sanitization,
/// the table of contents, and the MDX metadata — so a rewritten tree loses
/// none of it. `frontmatterJson` is carried through untouched.
#[napi(js_name = "transformFromMdast")]
pub fn transform_from_mdast(
    ast_json: String,
    frontmatter_json: String,
    options: Option<JsTransformOptions>,
) -> TransformResult {
    crate::ffi::recover(
        || {
            let core_options = options.unwrap_or_default().into();
            MarkdownTransformer::from_options(&core_options)
                .transform_from_mdast_json(&ast_json, &frontmatter_json)
                .into()
        },
        || TransformResult {
            html: String::new(),
            frontmatter: "{}".to_string(),
            toc: vec![],
            errors: vec![crate::ffi::UNEXPECTED_PANIC.to_string()],
            imports: vec![],
            exports: vec![],
            components: vec![],
        },
    )
}
