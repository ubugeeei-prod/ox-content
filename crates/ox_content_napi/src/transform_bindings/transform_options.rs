use napi::bindgen_prelude::Either;
use napi_derive::napi;
use ox_content_transform::{MathOptions, TransformOptions};

use super::{
    JsAttrsOptions, JsBadgeOptions, JsCardOptions, JsCodeGroupOptions, JsCodeImportOptions,
    JsContainerOptions, JsEditThisPageOptions, JsEmojiShortcodeOptions, JsFileTreeOptions,
    JsImageOptions, JsIncludeOptions, JsMagicLinkOptions, JsMathOptions, JsSanitizeOptions,
    JsStepsOptions, JsWikiLinkOptions,
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

    /// Enable MDX JSX, ESM, and expression nodes.
    ///
    /// Default: `false`.
    pub mdx: Option<bool>,

    /// Enable footnote references and definitions.
    ///
    /// Default: `false`, or `true` when `gfm` is `true`.
    pub footnotes: Option<bool>,

    /// Render footnotes as a semantic ordered section with numeric markers.
    ///
    /// Default: `false`.
    pub semantic_footnotes: Option<bool>,

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
    /// Default: `true`.
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

    /// Opt-in visible heading permalinks (`<a class="header-anchor" href="#id">`).
    ///
    /// Default: disabled. Existing HTML stays unchanged until this is enabled.
    pub heading_permalinks: Option<bool>,

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

    /// Recognize emphasis whose delimiters sit against East Asian punctuation.
    ///
    /// CommonMark's flanking rules let punctuation on the outside of a `*`/`_`
    /// run block it, and they read Unicode punctuation as a whole — so
    /// `A**強調。**B` stays literal text. CJK sets punctuation directly against
    /// the words it follows, which is why this bites there and rarely in Latin
    /// text. Enabling this classifies East Asian punctuation as an ordinary
    /// character for that decision only; halfwidth ASCII punctuation is
    /// untouched, so Latin documents parse identically.
    ///
    /// Emphasis merely adjacent to CJK *characters* (`これは**重要**です`)
    /// needs no option — CommonMark already allows it.
    ///
    /// Off by default because it is a deliberate deviation from the
    /// specification.
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

    /// Opt-in `::: tip` custom containers.
    ///
    /// Default: disabled.
    pub containers: Option<JsContainerOptions>,

    /// Opt-in Markdown file includes via `<!-- @include: PATH -->`.
    ///
    /// Default: disabled.
    pub includes: Option<JsIncludeOptions>,

    /// Opt-in `::: steps` ordered lists.
    ///
    /// Default: disabled.
    pub steps: Option<JsStepsOptions>,

    /// Opt-in `::: code-group` fence groups.
    ///
    /// Default: disabled.
    pub code_groups: Option<JsCodeGroupOptions>,

    /// Opt-in `{badge:variant}` inline badges.
    ///
    /// Default: disabled.
    pub badges: Option<JsBadgeOptions>,

    /// Opt-in `{link:...}` rich magic links.
    ///
    /// Default: disabled.
    pub magic_links: Option<JsMagicLinkOptions>,

    /// Opt-in figures, captions, and lazy images.
    ///
    /// Default: disabled.
    pub images: Option<JsImageOptions>,

    /// Opt-in `::: card` / `::: link-card` / `::: card-grid` blocks.
    ///
    /// Default: disabled.
    pub cards: Option<JsCardOptions>,

    /// Opt-in `$…$` inline and `$$…$$` block math.
    ///
    /// Default: disabled.
    pub math: Option<Either<bool, JsMathOptions>>,

    /// Opt-in static `file-tree` fences.
    ///
    /// Default: disabled.
    pub file_tree: Option<JsFileTreeOptions>,
}

impl From<JsTransformOptions> for TransformOptions {
    fn from(value: JsTransformOptions) -> Self {
        Self {
            gfm: value.gfm,
            mdx: value.mdx,
            footnotes: value.footnotes,
            semantic_footnotes: value.semantic_footnotes,
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
            heading_permalinks: value.heading_permalinks,
            wiki_links: value.wiki_links.map(Into::into),
            emoji_shortcodes: value.emoji_shortcodes.map(Into::into),
            attributes: value.attributes.map(Into::into),
            cjk_emphasis: value.cjk_emphasis,
            code_imports: value.code_imports.map(Into::into),
            sanitize: value.sanitize.map(Into::into),
            edit_this_page: value.edit_this_page.map(Into::into),
            containers: value.containers.map(Into::into),
            includes: value.includes.map(Into::into),
            steps: value.steps.map(Into::into),
            code_groups: value.code_groups.map(Into::into),
            badges: value.badges.map(Into::into),
            magic_links: value.magic_links.map(Into::into),
            images: value.images.map(Into::into),
            cards: value.cards.map(Into::into),
            math: match value.math {
                Some(Either::A(enabled)) => Some(MathOptions { enabled: Some(enabled) }),
                Some(Either::B(options)) => Some(options.into()),
                None => None,
            },
            file_tree: value.file_tree.map(Into::into),
        }
    }
}
