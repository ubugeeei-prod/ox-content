//! Public configuration types for HTML rendering.
//!
//! Keeping options separate from the renderer implementation makes the public API easy
//! to scan: this module owns only user-supplied configuration and lightweight enums.

/// HTML renderer options.
#[derive(Debug, Clone)]
pub struct HtmlRendererOptions {
    /// Use XHTML-style self-closing tags (e.g., `<br />`).
    pub xhtml: bool,
    /// Add soft breaks between inline elements.
    pub soft_break: String,
    /// Add hard breaks.
    pub hard_break: String,
    /// Enable syntax highlighting for code blocks.
    pub highlight: bool,
    /// Sanitize HTML output.
    pub sanitize: bool,
    /// Convert `.md` links to `.html` links for SSG output.
    pub convert_md_links: bool,
    /// Base URL for absolute link conversion (e.g., "/" or "/docs/").
    pub base_url: String,
    /// Source file path for relative link resolution.
    /// Used to determine if the current file is an index file.
    pub source_path: String,
    /// Enable line annotations for code blocks using fence meta.
    pub code_annotations: bool,
    /// Fence meta key used to read code annotations.
    pub code_annotation_meta_key: String,
    /// Code annotation syntax mode.
    pub code_annotation_syntax: CodeAnnotationSyntax,
    /// Enable line numbers for all code blocks by default.
    pub code_annotation_default_line_numbers: bool,
    /// Maximum heading depth included in inline TOCs.
    pub toc_max_depth: u8,
    /// Auto-link bare URLs in text. When enabled, any occurrence in a text
    /// node that starts with one of [`Self::autolink_patterns`] is wrapped
    /// in an `<a>` tag. Auto-linking is suppressed inside an existing link.
    pub autolink_urls: bool,
    /// URL prefix patterns recognised by [`Self::autolink_urls`]. Defaults
    /// to `["http://", "https://"]`. Register additional schemes (e.g.
    /// `"ftp://"`, `"mailto:"`) by pushing onto this vec.
    pub autolink_patterns: Vec<String>,
    /// When auto-linking, emit `target="_blank" rel="noopener noreferrer"`.
    /// Independent from the existing markdown-link behaviour, which always
    /// adds the attributes for http/https hrefs.
    pub autolink_target_blank: bool,
}

impl HtmlRendererOptions {
    /// Creates new options with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            xhtml: false,
            soft_break: "\n".to_string(),
            hard_break: "<br>\n".to_string(),
            highlight: false,
            sanitize: false,
            convert_md_links: false,
            base_url: "/".to_string(),
            source_path: String::new(),
            code_annotations: false,
            code_annotation_meta_key: "annotate".to_string(),
            code_annotation_syntax: CodeAnnotationSyntax::Attribute,
            code_annotation_default_line_numbers: false,
            toc_max_depth: 3,
            autolink_urls: false,
            autolink_patterns: Vec::from([String::from("http://"), String::from("https://")]),
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
