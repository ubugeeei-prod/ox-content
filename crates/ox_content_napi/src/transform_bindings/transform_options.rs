use napi_derive::napi;
use ox_content_transform::TransformOptions;

use super::{
    JsAttrsOptions, JsCodeImportOptions, JsEditThisPageOptions, JsEmojiShortcodeOptions,
    JsSanitizeOptions, JsWikiLinkOptions,
};

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

impl From<JsTransformOptions> for TransformOptions {
    fn from(value: JsTransformOptions) -> Self {
        Self {
            gfm: value.gfm,
            footnotes: value.footnotes,
            task_lists: value.task_lists,
            tables: value.tables,
            strikethrough: value.strikethrough,
            autolinks: value.autolinks,
            frontmatter: value.frontmatter,
            toc_max_depth: value.toc_max_depth,
            convert_md_links: value.convert_md_links,
            base_url: value.base_url,
            source_path: value.source_path,
            code_annotations: value.code_annotations,
            code_annotation_meta_key: value.code_annotation_meta_key,
            code_annotation_syntax: value.code_annotation_syntax,
            code_annotation_default_line_numbers: value.code_annotation_default_line_numbers,
            autolink_urls: value.autolink_urls,
            autolink_patterns: value.autolink_patterns,
            autolink_target_blank: value.autolink_target_blank,
            wiki_links: value.wiki_links.map(Into::into),
            emoji_shortcodes: value.emoji_shortcodes.map(Into::into),
            attributes: value.attributes.map(Into::into),
            cjk_emphasis: value.cjk_emphasis,
            code_imports: value.code_imports.map(Into::into),
            sanitize: value.sanitize.map(Into::into),
            edit_this_page: value.edit_this_page.map(Into::into),
        }
    }
}
