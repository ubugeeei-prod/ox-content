use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{
    features, highlight, media_embeds, pm, sanitize, tabs, transformer::MarkdownTransformer,
    youtube, JsSourceOptions, JsSourceOrigin, PreparedSourceResult, TransformResult,
};

mod async_task;
mod code_blocks;
mod embed_types;
mod feature_options;
mod transform_options;

pub use async_task::TransformTask;
pub use code_blocks::{JsCodeBlock, JsCodeBlockDiagnostic};
pub use embed_types::{JsPmOptions, JsPmTransformResult, JsTabsTransformResult, JsYouTubeOptions};
pub use feature_options::{
    JsAttrsOptions, JsCodeBlockLintOptions, JsCodeImportOptions, JsDocsTestOptions,
    JsEditThisPageOptions, JsEmojiShortcodeOptions, JsMediaEmbedsOptions, JsSanitizeOptions,
    JsWikiLinkOptions,
};
pub use transform_options::JsTransformOptions;

/// Restores code block metadata after JavaScript-side syntax highlighting.
#[napi]
pub fn merge_highlighted_code_blocks(original_html: String, highlighted_html: String) -> String {
    highlight::merge_highlighted_code_blocks(&original_html, &highlighted_html)
}

/// Rewrites `<youtube ...>` elements in rendered HTML into responsive,
/// privacy-enhanced iframe embeds. Rust port of the TS `transformYouTube`.
#[napi]
pub fn transform_youtube_embeds(html: String, options: Option<JsYouTubeOptions>) -> String {
    let defaults = youtube::YouTubeEmbedOptions::default();
    let resolved = match options {
        Some(options) => youtube::YouTubeEmbedOptions {
            privacy_enhanced: options.privacy_enhanced.unwrap_or(defaults.privacy_enhanced),
            aspect_ratio: options.aspect_ratio.unwrap_or(defaults.aspect_ratio),
            allow_fullscreen: options.allow_fullscreen.unwrap_or(defaults.allow_fullscreen),
            lazy_load: options.lazy_load.unwrap_or(defaults.lazy_load),
        },
        None => defaults,
    };
    youtube::transform_youtube(&html, &resolved)
}

/// Rewrites `<tabs><tab>...</tab></tabs>` blocks in rendered HTML into the no-JS
/// CSS tab widget plus a `<details>` fallback. Rust port of the TS
/// `transformTabs`. Groups are numbered from `start_group`.
#[napi]
pub fn transform_tabs_embeds(html: String, start_group: u32) -> JsTabsTransformResult {
    let result = tabs::transform_tabs(&html, start_group);
    JsTabsTransformResult { html: result.html, group_count: result.group_count }
}

/// Expand `<pm>` blocks in rendered HTML into npm/pnpm/yarn/bun install tabs.
///
/// The single npm-style command inside each `<pm>` element is converted to the
/// equivalent command for every package manager and rendered into the shared
/// `ox-tabs` widget. Groups are numbered from `start_group`. Syncing is opt-in
/// via `options.sync` and off by default.
#[napi]
pub fn transform_pm_embeds(
    html: String,
    start_group: u32,
    options: Option<JsPmOptions>,
) -> JsPmTransformResult {
    let resolved =
        pm::PmOptions { sync: options.and_then(|options| options.sync).unwrap_or(false) };
    let result = pm::transform_pm(&html, start_group, resolved);
    JsPmTransformResult { html: result.html, group_count: result.group_count }
}

/// Transforms Markdown source into HTML, frontmatter, and TOC.
///
/// This is the main entry point for @ox-content/unplugin.
#[napi]
pub fn transform(source: String, options: Option<JsTransformOptions>) -> TransformResult {
    let opts = options.unwrap_or_default();
    MarkdownTransformer::from_options(&opts).transform(&source)
}

/// Sanitize an HTML string with safe defaults or an explicit allow-list.
#[napi(js_name = "sanitizeHtml")]
pub fn sanitize_html_binding(html: String, options: Option<JsSanitizeOptions>) -> String {
    sanitize::sanitize_html(&html, options.as_ref())
}

/// Transform opt-in static media embed components in already-rendered HTML.
#[napi(js_name = "transformMediaEmbeds")]
pub fn transform_media_embeds(html: String, options: Option<JsMediaEmbedsOptions>) -> String {
    media_embeds::transform_media_embeds(&html, options.as_ref())
}

/// Extract fenced code blocks from Markdown.
#[napi(js_name = "extractCodeBlocks")]
pub fn extract_code_blocks(source: String) -> Vec<JsCodeBlock> {
    features::extract_code_blocks(&source).into_iter().map(Into::into).collect()
}

/// Lint fenced code blocks in Markdown.
#[napi(js_name = "lintCodeBlocks")]
pub fn lint_code_blocks(
    source: String,
    options: Option<JsCodeBlockLintOptions>,
) -> Vec<JsCodeBlockDiagnostic> {
    features::lint_code_blocks(&source, options.as_ref()).into_iter().map(Into::into).collect()
}

/// Extract runnable documentation examples for Vitest harness generation.
#[napi(js_name = "extractDocsTests")]
pub fn extract_docs_tests(source: String, options: Option<JsDocsTestOptions>) -> Vec<JsCodeBlock> {
    features::extract_docs_tests(&source, options.as_ref()).into_iter().map(Into::into).collect()
}

/// Transforms Markdown into a raw mdast transfer buffer.
///
/// This keeps frontmatter parsing and mdast generation on the Rust side and
/// transfers a single external memory block to JavaScript for deserialization.
#[napi]
pub fn transform_mdast_raw(
    source: String,
    options: Option<JsTransformOptions>,
) -> Result<Uint8Array> {
    let opts = options.unwrap_or_default();
    MarkdownTransformer::from_options(&opts).transform_mdast_raw(&source)
}

/// Splits Markdown source into content and frontmatter in a raw transfer buffer.
///
/// This is used by JavaScript-side markdown-it and custom unified parser paths so
/// frontmatter stripping can stay on the Rust side even when parsing continues in JS.
#[napi]
pub fn prepare_source_raw(source: String, options: Option<JsSourceOptions>) -> Result<Uint8Array> {
    let frontmatter = options.unwrap_or_default().frontmatter.unwrap_or(true);
    MarkdownTransformer::with_frontmatter(frontmatter).prepare_source_raw(&source)
}

/// Splits Markdown source into content and parsed frontmatter.
#[napi(js_name = "prepareSource")]
pub fn prepare_source(source: String, options: Option<JsSourceOptions>) -> PreparedSourceResult {
    let frontmatter = options.unwrap_or_default().frontmatter.unwrap_or(true);
    let prepared = MarkdownTransformer::with_frontmatter(frontmatter).prepare_source(&source);

    PreparedSourceResult {
        content: prepared.content,
        frontmatter: prepared.frontmatter.into_iter().collect(),
        source_offset: JsSourceOrigin {
            byte_offset: prepared.source_origin.byte_offset,
            offset: prepared.source_origin.offset,
            line: prepared.source_origin.line,
            column: prepared.source_origin.column,
        },
    }
}

/// Transforms Markdown source asynchronously (runs on worker thread).
#[napi]
pub fn transform_async(
    source: String,
    options: Option<JsTransformOptions>,
) -> AsyncTask<TransformTask> {
    let opts = options.unwrap_or_default();
    AsyncTask::new(TransformTask { source, options: opts })
}
