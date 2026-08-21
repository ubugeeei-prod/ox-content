//! Public configuration types for HTML rendering.
//!
//! Keeping options separate from the renderer implementation makes the public API easy
//! to scan: this module owns only user-supplied configuration and lightweight enums.

/// HTML renderer options.
///
/// Use [`HtmlRendererOptions::new`] or [`Default::default`] for the documented
/// defaults.
#[derive(Debug, Clone)]
pub struct HtmlRendererOptions {
    /// Use XHTML-style self-closing tags (e.g., `<br />`).
    ///
    /// Default: `false`.
    pub xhtml: bool,

    /// Add soft breaks between inline elements.
    ///
    /// Default: `"\n"`.
    pub soft_break: String,

    /// Add hard breaks.
    ///
    /// Default: `"<br>\n"`.
    pub hard_break: String,

    /// Enable syntax highlighting for code blocks.
    ///
    /// Default: `false`.
    pub highlight: bool,

    /// Sanitize HTML output.
    ///
    /// Default: `false`.
    pub sanitize: bool,

    /// Apply the GFM `tagfilter` extension ("Disallowed Raw HTML"):
    /// neutralize `<title>`, `<textarea>`, `<style>`, `<xmp>`, `<iframe>`,
    /// `<noembed>`, `<noframes>`, `<script>`, and `<plaintext>` by escaping
    /// their leading `<`, leaving all other raw HTML untouched.
    ///
    /// Unlike [`Self::sanitize`], which escapes every raw HTML node, this
    /// keeps ordinary markup working. It is off by default because raw HTML
    /// passthrough is standard Markdown behaviour that embeds rely on.
    ///
    /// Default: `false`.
    pub disallow_raw_html: bool,

    /// Convert `.md` links to `.html` links for SSG output.
    ///
    /// Default: `false`.
    pub convert_md_links: bool,

    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    ///
    /// Default: `"/"`.
    pub base_url: String,

    /// Source file path for relative link resolution.
    /// Used to determine if the current file is an index file.
    ///
    /// Default: empty string.
    pub source_path: String,

    /// Enable line annotations for code blocks using fence meta.
    ///
    /// Default: `false`.
    pub code_annotations: bool,

    /// Fence meta key used to read code annotations.
    ///
    /// Default: `"annotate"`.
    pub code_annotation_meta_key: String,

    /// Code annotation syntax mode.
    ///
    /// Default: [`CodeAnnotationSyntax::Attribute`].
    pub code_annotation_syntax: CodeAnnotationSyntax,

    /// Enable line numbers for all code blocks by default.
    ///
    /// Default: `false`.
    pub code_annotation_default_line_numbers: bool,

    /// Maximum heading depth included in inline TOCs.
    ///
    /// Default: `3`.
    pub toc_max_depth: u8,

    /// Auto-link bare URLs in text. When enabled, any occurrence in a text
    /// node that starts with one of [`Self::autolink_patterns`] is wrapped
    /// in an `<a>` tag. Auto-linking is suppressed inside an existing link.
    ///
    /// Default: `true`.
    pub autolink_urls: bool,

    /// URL prefix patterns recognised by [`Self::autolink_urls`]. Defaults
    /// to `["http://", "https://"]`. Register additional schemes (e.g.
    /// `"ftp://"`, `"mailto:"`) by pushing onto this vec.
    ///
    /// Default: `["http://", "https://"]`.
    pub autolink_patterns: Vec<String>,

    /// When auto-linking, emit `target="_blank" rel="noopener noreferrer"`.
    /// Independent from the existing markdown-link behaviour, which always
    /// adds the attributes for http/https hrefs.
    ///
    /// Default: `true`.
    pub autolink_target_blank: bool,
}

const DEFAULT_SOFT_BREAK: &str = "\n";
const DEFAULT_HARD_BREAK: &str = "<br>\n";
const DEFAULT_BASE_URL: &str = "/";
const DEFAULT_CODE_ANNOTATION_META_KEY: &str = "annotate";
const DEFAULT_AUTOLINK_PATTERNS: [&str; 2] = ["http://", "https://"];

/// Allocation-free internal form of [`HtmlRendererOptions`].
///
/// Public options stay ergonomic owned values, while [`super::HtmlRenderer::new`]
/// can represent every default string and pattern with static data. Custom options
/// are moved in without cloning and keep their exact values, including empty strings
/// and an empty pattern list.
pub(super) struct RendererOptions {
    pub(super) xhtml: bool,
    hard_break: Option<String>,
    pub(super) sanitize: bool,
    pub(super) disallow_raw_html: bool,
    pub(super) convert_md_links: bool,
    base_url: Option<String>,
    source_path: Option<String>,
    pub(super) code_annotations: bool,
    code_annotation_meta_key: Option<String>,
    pub(super) code_annotation_syntax: CodeAnnotationSyntax,
    pub(super) code_annotation_default_line_numbers: bool,
    pub(super) toc_max_depth: u8,
    pub(super) autolink_urls: bool,
    autolink_patterns: Option<Vec<String>>,
    pub(super) autolink_target_blank: bool,
}

impl RendererOptions {
    pub(super) const fn defaults() -> Self {
        Self {
            xhtml: false,
            hard_break: None,
            sanitize: false,
            disallow_raw_html: false,
            convert_md_links: false,
            base_url: None,
            source_path: None,
            code_annotations: false,
            code_annotation_meta_key: None,
            code_annotation_syntax: CodeAnnotationSyntax::Attribute,
            code_annotation_default_line_numbers: false,
            toc_max_depth: 3,
            autolink_urls: true,
            autolink_patterns: None,
            autolink_target_blank: true,
        }
    }

    pub(super) fn hard_break(&self) -> &str {
        self.hard_break.as_deref().unwrap_or(DEFAULT_HARD_BREAK)
    }

    pub(super) fn base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
    }

    pub(super) fn source_path(&self) -> &str {
        self.source_path.as_deref().unwrap_or("")
    }

    pub(super) fn code_annotation_meta_key(&self) -> &str {
        self.code_annotation_meta_key.as_deref().unwrap_or(DEFAULT_CODE_ANNOTATION_META_KEY)
    }

    pub(super) fn autolink_patterns(&self) -> AutolinkPatterns<'_> {
        self.autolink_patterns.as_deref().map_or(
            AutolinkPatterns::Defaults(&DEFAULT_AUTOLINK_PATTERNS),
            AutolinkPatterns::Custom,
        )
    }
}

impl From<HtmlRendererOptions> for RendererOptions {
    fn from(options: HtmlRendererOptions) -> Self {
        Self {
            xhtml: options.xhtml,
            hard_break: Some(options.hard_break),
            sanitize: options.sanitize,
            disallow_raw_html: options.disallow_raw_html,
            convert_md_links: options.convert_md_links,
            base_url: Some(options.base_url),
            source_path: Some(options.source_path),
            code_annotations: options.code_annotations,
            code_annotation_meta_key: Some(options.code_annotation_meta_key),
            code_annotation_syntax: options.code_annotation_syntax,
            code_annotation_default_line_numbers: options.code_annotation_default_line_numbers,
            toc_max_depth: options.toc_max_depth,
            autolink_urls: options.autolink_urls,
            autolink_patterns: Some(options.autolink_patterns),
            autolink_target_blank: options.autolink_target_blank,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum AutolinkPatterns<'a> {
    Defaults(&'a [&'static str]),
    Custom(&'a [String]),
}

impl<'a> AutolinkPatterns<'a> {
    pub(super) fn is_empty(self) -> bool {
        match self {
            Self::Defaults(patterns) => patterns.is_empty(),
            Self::Custom(patterns) => patterns.is_empty(),
        }
    }
}

impl HtmlRendererOptions {
    /// Creates new options with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            xhtml: false,
            soft_break: DEFAULT_SOFT_BREAK.to_string(),
            hard_break: DEFAULT_HARD_BREAK.to_string(),
            highlight: false,
            sanitize: false,
            disallow_raw_html: false,
            convert_md_links: false,
            base_url: DEFAULT_BASE_URL.to_string(),
            source_path: String::new(),
            code_annotations: false,
            code_annotation_meta_key: DEFAULT_CODE_ANNOTATION_META_KEY.to_string(),
            code_annotation_syntax: CodeAnnotationSyntax::Attribute,
            code_annotation_default_line_numbers: false,
            toc_max_depth: 3,
            autolink_urls: true,
            autolink_patterns: DEFAULT_AUTOLINK_PATTERNS.iter().map(ToString::to_string).collect(),
            autolink_target_blank: true,
        }
    }
}

impl Default for HtmlRendererOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeAnnotationSyntax {
    /// Read `annotate="kind:line"` style metadata from the code-fence info string.
    ///
    /// This is the stable ox-content syntax and is useful when authored Markdown should
    /// stay independent from a particular documentation theme.
    Attribute,

    /// Read VitePress-compatible fence metadata and inline `// [!code ...]` directives.
    ///
    /// Use this when importing or sharing Markdown with VitePress projects that already
    /// use `{1,3}`, `[title]`, `:line-numbers`, or inline diff/focus annotations.
    VitePress,

    /// Accept both ox-content attributes and VitePress-compatible directives.
    ///
    /// Attribute annotations are applied first, then VitePress metadata can add titles,
    /// line numbers, and inline directives without replacing existing classes.
    Both,
}

impl CodeAnnotationSyntax {
    pub(super) fn includes_attribute(self) -> bool {
        matches!(self, Self::Attribute | Self::Both)
    }

    pub(super) fn includes_vitepress(self) -> bool {
        matches!(self, Self::VitePress | Self::Both)
    }
}
