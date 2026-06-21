//! Shared asset extraction for generated SSG pages.

mod blocks;
mod chunk;
mod css;
mod script;

use serde::{Deserialize, Serialize};

use blocks::{
    find_first_tag_block, find_last_body_script_block, find_marked_block, BODY_CLOSE,
    SCRIPT_BLOCK_END, SCRIPT_BLOCK_START, SCRIPT_CLOSE, SCRIPT_OPEN, STYLE_BLOCK_END,
    STYLE_BLOCK_START, STYLE_CLOSE, STYLE_OPEN,
};
use chunk::AssetCache;
use css::build_style_replacement;
use script::build_script_replacement;

/// Generated HTML page before or after asset extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedHtmlPage {
    /// Source Markdown path.
    pub input_path: String,
    /// Output HTML path.
    pub output_path: String,
    /// HTML content.
    pub html: String,
}

/// Shared CSS or JavaScript asset extracted from generated pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedAsset {
    /// Output file path.
    pub output_path: String,
    /// Public URL path used from HTML.
    pub public_path: String,
    /// Asset content.
    pub content: String,
}

/// Result of shared asset extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalizedAssets {
    /// HTML pages with inline assets replaced.
    pub pages: Vec<GeneratedHtmlPage>,
    /// Extracted shared assets.
    pub assets: Vec<SharedAsset>,
}

/// Extracts shared CSS/JS chunks from generated HTML pages.
///
/// The Rust HTML generator emits marked style/script regions. This pass walks
/// each generated page once, replaces those inline regions with links/scripts
/// to content-addressed shared files, and preserves page-local inline CSS when
/// externalizing would break relative `url(...)` references.
#[must_use]
pub fn externalize_shared_page_assets(
    pages: Vec<GeneratedHtmlPage>,
    out_dir: &str,
    base: &str,
) -> ExternalizedAssets {
    let mut css_chunks = AssetCache::default();
    let mut js_chunks = AssetCache::default();

    let optimized_pages = pages
        .into_iter()
        .map(|page| {
            let mut html = page.html;

            if let Some(block) = find_marked_block(
                &html,
                STYLE_BLOCK_START,
                STYLE_BLOCK_END,
                STYLE_OPEN,
                STYLE_CLOSE,
            ) {
                let replacement =
                    build_style_replacement(&block.content, &mut css_chunks, out_dir, base);
                html.replace_range(block.start..block.end, &replacement);
            } else if let Some(block) = find_first_tag_block(&html, STYLE_OPEN, STYLE_CLOSE) {
                let replacement =
                    build_style_replacement(&block.content, &mut css_chunks, out_dir, base);
                html.replace_range(block.start..block.end, &replacement);
            }

            if let Some(block) = find_marked_block(
                &html,
                SCRIPT_BLOCK_START,
                SCRIPT_BLOCK_END,
                SCRIPT_OPEN,
                SCRIPT_CLOSE,
            ) {
                let replacement =
                    build_script_replacement(&block.content, &mut js_chunks, out_dir, base);
                html.replace_range(block.start..block.end, &replacement);
            } else if let Some(block) = find_last_body_script_block(&html) {
                let replacement =
                    build_script_replacement(&block.content, &mut js_chunks, out_dir, base);
                let replacement = if replacement.is_empty() {
                    BODY_CLOSE.to_string()
                } else {
                    format!("{replacement}\n{BODY_CLOSE}")
                };
                html.replace_range(block.start..block.end, &replacement);
            }

            GeneratedHtmlPage { html, ..page }
        })
        .collect();

    let assets = css_chunks.into_chunks().into_iter().chain(js_chunks.into_chunks()).collect();

    ExternalizedAssets { pages: optimized_pages, assets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_shared_style_and_script_assets() {
        let pages = vec![
            GeneratedHtmlPage {
                input_path: "a.md".to_string(),
                output_path: "/tmp/site/a/index.html".to_string(),
                html: format_page("A"),
            },
            GeneratedHtmlPage {
                input_path: "b.md".to_string(),
                output_path: "/tmp/site/b/index.html".to_string(),
                html: format_page("B"),
            },
        ];

        let result = externalize_shared_page_assets(pages, "/tmp/site", "/docs/");

        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.assets.len(), 3);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn keeps_relative_url_css_inline() {
        let pages = vec![GeneratedHtmlPage {
            input_path: "a.md".to_string(),
            output_path: "/tmp/site/a/index.html".to_string(),
            html: "<html><head><style>.hero{background:url('./hero.png')}</style></head><body></body></html>"
                .to_string(),
        }];

        let result = externalize_shared_page_assets(pages, "/tmp/site", "/docs/");

        assert!(result.assets.is_empty());
        insta::assert_debug_snapshot!(result);
    }

    fn format_page(title: &str) -> String {
        format!(
            r#"<!doctype html>
<html>
<head>
  <!-- ox-content:styles:start -->
  <style>/* ox-content:css:base:start */
body {{ color: red; }}
/* ox-content:css:base:end */
/* ox-content:css:theme:start */
:root {{ --title: "{title}"; }}
/* ox-content:css:theme:end */</style>
  <!-- ox-content:styles:end -->
</head>
<body>
  <h1>{title}</h1>
  <!-- ox-content:scripts:start -->
  <script>const boot = "__OX_CONTENT_SEARCH_CHUNK__";
// ox-content:search:start
const searchData = true;
// ox-content:search:end</script>
  <!-- ox-content:scripts:end -->
</body>
</html>"#
        )
    }
}
