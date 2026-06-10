use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;

use crate::{
    features, highlight, media_embeds, pm, sanitize, tabs, transformer::MarkdownTransformer,
    youtube, JsSourceOptions, JsSourceOrigin, PreparedSourceResult, TransformResult,
};

/// Wiki-link transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsWikiLinkOptions {
    /// Enable `[[target]]` and `[[target|label]]` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Base URL used for site-relative wiki links.
    ///
    /// Default: `"/"`.
    pub base_url: Option<String>,
}

/// Emoji-shortcode transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsEmojiShortcodeOptions {
    /// Enable `:shortcode:` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Custom shortcode map. Values are emitted verbatim.
    ///
    /// Default: `{}`.
    pub custom: Option<HashMap<String, String>>,
}

/// Attribute syntax transform options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsAttrsOptions {
    /// Enable markdown-it-attrs style `{#id .class key=value}`.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,
}

/// Code import / snippet injection options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeImportOptions {
    /// Enable `<<< path{selector}` snippet injection.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Root directory used for `@/` and absolute snippet imports.
    ///
    /// Default: project root from the JavaScript caller.
    pub root_dir: Option<String>,
}

/// HTML sanitizer options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsSanitizeOptions {
    /// Enable sanitizer. When omitted, passing this object enables it.
    ///
    /// Default: `false` when the whole option is omitted; `true` when this object is present.
    pub enabled: Option<bool>,

    /// Allowed tag names. Omit for safe defaults.
    ///
    /// Default: built-in safe tag allow list.
    pub allowed_tags: Option<Vec<String>>,

    /// Allowed attribute names. Omit for safe defaults.
    ///
    /// Default: built-in safe attribute allow list.
    pub allowed_attributes: Option<Vec<String>>,

    /// Allowed URL schemes for URL-bearing attributes. Omit for safe defaults.
    ///
    /// Default: built-in safe URL scheme allow list.
    pub allowed_url_schemes: Option<Vec<String>>,
}

/// Edit-this-page link options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsEditThisPageOptions {
    /// Enable edit link generation.
    ///
    /// Default: `false` unless `repo_url` is provided by the JavaScript resolver.
    pub enabled: Option<bool>,

    /// GitHub repository URL, e.g. `https://github.com/owner/repo`.
    pub repo_url: Option<String>,

    /// Branch used in edit URLs.
    ///
    /// Default: `"main"`.
    pub branch: Option<String>,

    /// Root directory used to relativize `sourcePath`.
    ///
    /// Default: no extra root prefix.
    pub root_dir: Option<String>,

    /// Link label.
    ///
    /// Default: `"Edit this page"`.
    pub label: Option<String>,
}

/// Code block linting options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsCodeBlockLintOptions {
    /// Enable code block linting.
    ///
    /// Default: `false` when the whole option is omitted.
    pub enabled: Option<bool>,

    /// Restrict linting to these language identifiers.
    ///
    /// Default: all fenced block languages.
    pub languages: Option<Vec<String>>,

    /// Report fences without a language identifier.
    ///
    /// Default: `false`.
    pub require_language: Option<bool>,

    /// Report trailing whitespace in code block lines.
    ///
    /// Default: `true`.
    pub trailing_spaces: Option<bool>,
}

/// Docs-as-tests extraction options.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDocsTestOptions {
    /// Enable docs test extraction.
    ///
    /// Default: `false` when the whole option is omitted.
    pub enabled: Option<bool>,

    /// Languages that can be emitted as test cases.
    ///
    /// Default: `["js", "jsx", "ts", "tsx", "mjs", "mts"]`.
    pub languages: Option<Vec<String>>,

    /// Require fence meta such as `test`, `runnable`, or `vitest`.
    ///
    /// Default: `true`.
    pub require_meta: Option<bool>,
}

/// Built-in media embed transform switches.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMediaEmbedsOptions {
    /// Render `<Spotify>` embeds.
    ///
    /// Default: `false`.
    pub spotify: Option<bool>,

    /// Render `<StackBlitz>` embeds.
    ///
    /// Default: `false`.
    pub stack_blitz: Option<bool>,

    /// Render `<Tweet>` / `<XPost>` static cards.
    ///
    /// Default: `false`.
    pub twitter: Option<bool>,

    /// Render `<Bluesky>` static cards.
    ///
    /// Default: `false`.
    pub bluesky: Option<bool>,

    /// Render `<WebContainer>` lazy placeholder blocks.
    ///
    /// Default: `false`.
    pub web_container: Option<bool>,
}

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

/// Transform options for JavaScript.
///
/// Omitted parser flags inherit the GFM profile when `gfm` is `true`; otherwise
/// they use the parser defaults.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsTransformOptions {
    /// Enable the GFM convenience profile.
    ///
    /// Default: `false`.
    pub gfm: Option<bool>,

    /// Enable footnote references and definitions.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub footnotes: Option<bool>,

    /// Enable GFM task-list item markers.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub task_lists: Option<bool>,

    /// Enable GFM pipe tables.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub tables: Option<bool>,

    /// Enable GFM strikethrough spans.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub strikethrough: Option<bool>,

    /// Enable GFM autolinks.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub autolinks: Option<bool>,

    /// Parse YAML frontmatter before transforming.
    ///
    /// Default: `false`.
    pub frontmatter: Option<bool>,

    /// Maximum TOC depth (1-6).
    ///
    /// Default: `3`.
    pub toc_max_depth: Option<u8>,

    /// Convert `.md` links to `.html` links for SSG output.
    ///
    /// Default: `false`.
    pub convert_md_links: Option<bool>,

    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    ///
    /// Default: `"/"`.
    pub base_url: Option<String>,

    /// Source file path for relative link resolution.
    ///
    /// Default: empty string.
    pub source_path: Option<String>,

    /// Enable line annotations for code blocks using fence meta.
    ///
    /// Default: `false`.
    pub code_annotations: Option<bool>,

    /// Fence meta key used to read code annotations.
    ///
    /// Default: `"annotate"`.
    pub code_annotation_meta_key: Option<String>,

    /// Code annotation syntax mode.
    ///
    /// Default: `"attribute"`.
    pub code_annotation_syntax: Option<String>,

    /// Enable line numbers for all code blocks by default.
    ///
    /// Default: `false`.
    pub code_annotation_default_line_numbers: Option<bool>,

    /// Auto-link bare URLs in text. When enabled, the renderer wraps any
    /// text occurrence starting with a registered pattern (default `http://`
    /// and `https://`) in an `<a>` tag.
    ///
    /// Default: `false`.
    pub autolink_urls: Option<bool>,

    /// URL prefix patterns for [`Self::autolink_urls`]. Overrides the
    /// default `["http://", "https://"]` when set.
    ///
    /// Default: `["http://", "https://"]`.
    pub autolink_patterns: Option<Vec<String>>,

    /// Add `target="_blank" rel="noopener noreferrer"` to auto-linked URLs.
    ///
    /// Default: `true`; ignored when [`Self::autolink_urls`] is off.
    pub autolink_target_blank: Option<bool>,

    /// Opt-in Obsidian-style wiki links.
    ///
    /// Default: disabled.
    pub wiki_links: Option<JsWikiLinkOptions>,

    /// Opt-in emoji shortcode expansion.
    ///
    /// Default: disabled.
    pub emoji_shortcodes: Option<JsEmojiShortcodeOptions>,

    /// Opt-in markdown-it-attrs style attributes.
    ///
    /// Default: disabled.
    pub attributes: Option<JsAttrsOptions>,

    /// Opt-in CJK emphasis compatibility flag. The parser is already CJK-friendly;
    /// this keeps the feature explicit in the public API.
    ///
    /// Default: `false`.
    pub cjk_emphasis: Option<bool>,

    /// Opt-in VitePress-style code import/snippet injection.
    ///
    /// Default: disabled.
    pub code_imports: Option<JsCodeImportOptions>,

    /// Opt-in HTML sanitizer.
    ///
    /// Default: disabled.
    pub sanitize: Option<JsSanitizeOptions>,

    /// Opt-in edit-this-page link generation.
    ///
    /// Default: disabled.
    pub edit_this_page: Option<JsEditThisPageOptions>,
}

/// Restores code block metadata after JavaScript-side syntax highlighting.
#[napi]
pub fn merge_highlighted_code_blocks(original_html: String, highlighted_html: String) -> String {
    highlight::merge_highlighted_code_blocks(&original_html, &highlighted_html)
}

/// Options for [`transform_youtube_embeds`]; all optional, matching the TS
/// `YouTubeOptions` defaults when omitted.
#[napi(object)]
pub struct JsYouTubeOptions {
    /// Use privacy-enhanced mode (`youtube-nocookie.com`).
    ///
    /// Default: `true`.
    pub privacy_enhanced: Option<bool>,

    /// Default iframe aspect ratio.
    ///
    /// Default: `"16/9"`.
    pub aspect_ratio: Option<String>,

    /// Allow fullscreen playback.
    ///
    /// Default: `true`.
    pub allow_fullscreen: Option<bool>,

    /// Lazy-load the iframe.
    ///
    /// Default: `true`.
    pub lazy_load: Option<bool>,
}

/// Rewrites `<youtube …>` elements in rendered HTML into responsive,
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

/// Result of [`transform_tabs_embeds`].
#[napi(object)]
pub struct JsTabsTransformResult {
    /// HTML with every `<tabs>` block expanded.
    pub html: String,

    /// Number of tab groups expanded; the caller advances its group counter by
    /// this amount so generated CSS covers exactly the emitted groups.
    pub group_count: u32,
}

/// Rewrites `<tabs><tab>…</tab></tabs>` blocks in rendered HTML into the no-JS
/// CSS tab widget plus a `<details>` fallback. Rust port of the TS
/// `transformTabs`. Groups are numbered from `start_group`.
#[napi]
pub fn transform_tabs_embeds(html: String, start_group: u32) -> JsTabsTransformResult {
    let result = tabs::transform_tabs(&html, start_group);
    JsTabsTransformResult { html: result.html, group_count: result.group_count }
}

/// Options for [`transform_pm_embeds`].
#[napi(object)]
pub struct JsPmOptions {
    /// Enable opt-in synced package-manager tab groups. When `true`, a
    /// `data-ox-tab-group="pkg-manager"` attribute is emitted so the client
    /// runtime keeps every pm tab group on the page in sync via `localStorage`.
    /// Off by default; when omitted/`false` the output has no group attribute
    /// and behaves exactly like a standalone tab group.
    pub sync: Option<bool>,
}

/// Result of [`transform_pm_embeds`].
#[napi(object)]
pub struct JsPmTransformResult {
    /// HTML with every `<pm>` block expanded into a package-manager tab widget.
    pub html: String,

    /// Number of tab groups expanded; the caller advances its shared tab-group
    /// counter by this amount.
    pub group_count: u32,
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
        frontmatter: prepared.frontmatter,
        source_offset: JsSourceOrigin {
            byte_offset: prepared.source_origin.byte_offset,
            offset: prepared.source_origin.offset,
            line: prepared.source_origin.line,
            column: prepared.source_origin.column,
        },
    }
}

// =============================================================================
pub struct TransformTask {
    source: String,
    options: JsTransformOptions,
}

impl Task for TransformTask {
    type Output = TransformResult;
    type JsValue = TransformResult;

    fn compute(&mut self) -> Result<Self::Output> {
        Ok(MarkdownTransformer::from_options(&self.options).transform(&self.source))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
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

// =============================================================================
// OG Image Generation API
// =============================================================================
