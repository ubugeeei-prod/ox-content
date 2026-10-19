use wasm_bindgen::prelude::*;

use ox_content_parser::ParserOptions;

/// Parser and renderer options exposed to JavaScript.
///
/// `new WasmParserOptions()` disables optional Markdown extensions by default
/// and uses renderer defaults for TOC and auto-link handling.
#[wasm_bindgen]
#[derive(Default)]
pub struct WasmParserOptions {
    pub(crate) gfm: bool,
    pub(crate) mdx: bool,
    // The extension flags are tri-state: `None` means "not set from JS",
    // which lets the `gfm` profile supply its own defaults instead of the
    // field defaults silently overwriting them (see the `From` impl).
    pub(crate) footnotes: Option<bool>,
    pub(crate) task_lists: Option<bool>,
    pub(crate) tables: Option<bool>,
    pub(crate) strikethrough: Option<bool>,
    pub(crate) autolinks: Option<bool>,
    pub(crate) superscript: bool,
    pub(crate) subscript: bool,
    pub(crate) smart_punctuation: bool,
    pub(crate) math: bool,
    pub(crate) definition_lists: bool,
    pub(crate) heading_attributes: bool,
    pub(crate) wiki_links: bool,
    pub(crate) toc_max_depth: u8,
    pub(crate) autolink_urls: bool,
    pub(crate) autolink_patterns: Vec<String>,
    pub(crate) autolink_target_blank: bool,
    pub(crate) link_target_blank: bool,
    pub(crate) source_spans: bool,
    pub(crate) semantic_footnotes: bool,
    pub(crate) heading_permalinks: bool,
    pub(crate) sanitize: bool,
}

#[wasm_bindgen]
impl WasmParserOptions {
    /// Creates options with all Markdown extension flags disabled.
    ///
    /// Defaults: `gfm = false`, `mdx = false`, `tocMaxDepth = 3`, `autolinkUrls = true`,
    /// `autolinkPatterns = ["http://", "https://"]`,
    /// `autolinkTargetBlank = true`, `linkTargetBlank = true`,
    /// `sourceSpans = false`, `semanticFootnotes = false`, and the non-GFM
    /// Markdown extensions (`superscript`, `subscript`, `smartPunctuation`,
    /// `math`, and `definitionLists`) disabled, and `sanitize = false`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            gfm: false,
            mdx: false,
            footnotes: None,
            task_lists: None,
            tables: None,
            strikethrough: None,
            autolinks: None,
            superscript: false,
            subscript: false,
            smart_punctuation: false,
            math: false,
            definition_lists: false,
            heading_attributes: false,
            wiki_links: false,
            toc_max_depth: 3,
            autolink_urls: true,
            autolink_patterns: vec!["http://".to_string(), "https://".to_string()],
            autolink_target_blank: true,
            link_target_blank: true,
            source_spans: false,
            semantic_footnotes: false,
            heading_permalinks: false,
            sanitize: false,
        }
    }

    /// Enables the GFM convenience profile.
    #[wasm_bindgen(setter)]
    pub fn set_gfm(&mut self, value: bool) {
        self.gfm = value;
    }

    /// Enables MDX JSX, ESM, and expression nodes.
    #[wasm_bindgen(setter)]
    pub fn set_mdx(&mut self, value: bool) {
        self.mdx = value;
    }

    /// Enables footnote references and definitions.
    #[wasm_bindgen(setter)]
    pub fn set_footnotes(&mut self, value: bool) {
        self.footnotes = Some(value);
    }

    /// Enables GFM task-list item markers such as `- [x]`.
    #[wasm_bindgen(setter = taskLists)]
    pub fn set_task_lists(&mut self, value: bool) {
        self.task_lists = Some(value);
    }

    /// Enables GFM pipe tables.
    #[wasm_bindgen(setter)]
    pub fn set_tables(&mut self, value: bool) {
        self.tables = Some(value);
    }

    /// Enables GFM strikethrough spans.
    #[wasm_bindgen(setter)]
    pub fn set_strikethrough(&mut self, value: bool) {
        self.strikethrough = Some(value);
    }

    /// Enables GFM autolinks in the parser.
    #[wasm_bindgen(setter)]
    pub fn set_autolinks(&mut self, value: bool) {
        self.autolinks = Some(value);
    }

    /// Enables `^text^` superscript spans.
    #[wasm_bindgen(setter)]
    pub fn set_superscript(&mut self, value: bool) {
        self.superscript = value;
    }

    /// Enables `~text~` subscript spans.
    #[wasm_bindgen(setter)]
    pub fn set_subscript(&mut self, value: bool) {
        self.subscript = value;
    }

    /// Enables smart punctuation replacement.
    #[wasm_bindgen(setter = smartPunctuation)]
    pub fn set_smart_punctuation(&mut self, value: bool) {
        self.smart_punctuation = value;
    }

    /// Enables `$...$` inline math and `$$...$$` block math.
    #[wasm_bindgen(setter)]
    pub fn set_math(&mut self, value: bool) {
        self.math = value;
    }

    /// Enables definition list blocks.
    #[wasm_bindgen(setter = definitionLists)]
    pub fn set_definition_lists(&mut self, value: bool) {
        self.definition_lists = value;
    }

    /// Enables Pandoc-style `{#id .class}` heading attribute blocks.
    #[wasm_bindgen(setter = headingAttributes)]
    pub fn set_heading_attributes(&mut self, value: bool) {
        self.heading_attributes = value;
    }

    /// Enables Obsidian-style wiki links as link nodes.
    #[wasm_bindgen(setter = wikiLinks)]
    pub fn set_wiki_links(&mut self, value: bool) {
        self.wiki_links = value;
    }

    /// Sets the maximum heading depth included in inline TOCs.
    #[wasm_bindgen(setter = tocMaxDepth)]
    pub fn set_toc_max_depth(&mut self, value: u8) {
        self.toc_max_depth = value;
    }

    /// Enables the renderer's URL auto-linking builtin.
    #[wasm_bindgen(setter = autolinkUrls)]
    pub fn set_autolink_urls(&mut self, value: bool) {
        self.autolink_urls = value;
    }

    /// Replaces the URL prefix patterns used by auto-linking.
    #[wasm_bindgen(setter = autolinkPatterns)]
    pub fn set_autolink_patterns(&mut self, value: Vec<String>) {
        self.autolink_patterns = value;
    }

    /// Toggles `target="_blank" rel="noopener noreferrer"` on auto-linked URLs.
    #[wasm_bindgen(setter = autolinkTargetBlank)]
    pub fn set_autolink_target_blank(&mut self, value: bool) {
        self.autolink_target_blank = value;
    }

    /// Toggles `target="_blank" rel="noopener noreferrer"` on parsed Markdown links.
    #[wasm_bindgen(setter = linkTargetBlank)]
    pub fn set_link_target_blank(&mut self, value: bool) {
        self.link_target_blank = value;
    }

    /// Emits `data-source-span="start-end"` on rendered block elements.
    #[wasm_bindgen(setter = sourceSpans)]
    pub fn set_source_spans(&mut self, value: bool) {
        self.source_spans = value;
    }

    /// Renders footnotes as a semantic ordered section with numeric markers.
    #[wasm_bindgen(setter = semanticFootnotes)]
    pub fn set_semantic_footnotes(&mut self, value: bool) {
        self.semantic_footnotes = value;
    }

    /// Appends a visible heading permalink.
    #[wasm_bindgen(setter = headingPermalinks)]
    pub fn set_heading_permalinks(&mut self, value: bool) {
        self.heading_permalinks = value;
    }

    /// Escapes raw HTML and neutralises unsafe link and image URLs
    /// (`javascript:` and friends), so untrusted Markdown can be inserted
    /// into a page with `innerHTML`.
    #[wasm_bindgen(setter)]
    pub fn set_sanitize(&mut self, value: bool) {
        self.sanitize = value;
    }
}

impl From<&WasmParserOptions> for ParserOptions {
    fn from(opts: &WasmParserOptions) -> Self {
        let mut options = if opts.gfm { ParserOptions::gfm() } else { ParserOptions::default() };

        if let Some(footnotes) = opts.footnotes {
            options.footnotes = footnotes;
        }
        if let Some(task_lists) = opts.task_lists {
            options.task_lists = task_lists;
        }
        if let Some(tables) = opts.tables {
            options.tables = tables;
        }
        if let Some(strikethrough) = opts.strikethrough {
            options.strikethrough = strikethrough;
        }
        if let Some(autolinks) = opts.autolinks {
            options.autolinks = autolinks;
        }
        options.mdx = opts.mdx;
        options.superscript = opts.superscript;
        options.subscript = opts.subscript;
        options.smart_punctuation = opts.smart_punctuation;
        options.math = opts.math;
        options.definition_lists = opts.definition_lists;
        options.heading_attributes = opts.heading_attributes;
        options.wiki_links = opts.wiki_links;

        options
    }
}

#[cfg(test)]
mod tests {
    use ox_content_parser::ParserOptions;

    use super::WasmParserOptions;

    #[test]
    fn mdx_setter_maps_to_parser_options_without_changing_the_default() {
        let defaults = ParserOptions::from(&WasmParserOptions::new());
        assert!(!defaults.mdx);

        let mut enabled = WasmParserOptions::new();
        enabled.set_mdx(true);
        assert!(ParserOptions::from(&enabled).mdx);
    }

    #[test]
    fn wiki_links_setter_maps_to_parser_options_without_gfm() {
        let defaults = ParserOptions::from(&WasmParserOptions::new());
        assert!(!defaults.wiki_links);

        let mut enabled = WasmParserOptions::new();
        enabled.set_gfm(true);
        enabled.set_wiki_links(true);
        let options = ParserOptions::from(&enabled);
        assert!(options.gfm);
        assert!(options.wiki_links);
    }
}
