//! Shared asset extraction for generated SSG pages.

mod blocks;
mod chunk;
mod css;
mod script;

use serde::{Deserialize, Serialize};

use blocks::{
    BODY_CLOSE, SCRIPT_BLOCK_END, SCRIPT_BLOCK_START, SCRIPT_CLOSE, SCRIPT_OPEN, STYLE_BLOCK_END,
    STYLE_BLOCK_START, STYLE_CLOSE, STYLE_OPEN, find_first_tag_block, find_last_body_script_block,
    find_marked_block,
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

/// Incremental shared-asset extractor for streaming page generation.
///
/// Keeping the chunk caches across pages preserves content de-duplication,
/// while externalizing each page immediately avoids retaining every expanded
/// inline theme in memory at once.
pub struct SharedAssetExternalizer<'a> {
    css_chunks: AssetCache,
    js_chunks: AssetCache,
    out_dir: &'a str,
    base: &'a str,
}

impl<'a> SharedAssetExternalizer<'a> {
    /// Creates an extractor for one SSG build.
    #[must_use]
    pub fn new(out_dir: &'a str, base: &'a str) -> Self {
        Self { css_chunks: AssetCache::default(), js_chunks: AssetCache::default(), out_dir, base }
    }

    /// Externalizes one page while retaining shared chunk state for later pages.
    #[must_use]
    pub fn externalize_page(&mut self, page: GeneratedHtmlPage) -> GeneratedHtmlPage {
        if !contains_inline_asset_tag(&page.html) {
            return page;
        }

        let mut html = page.html;

        if let Some(block) =
            find_marked_block(&html, STYLE_BLOCK_START, STYLE_BLOCK_END, STYLE_OPEN, STYLE_CLOSE)
        {
            let replacement = build_style_replacement(
                &block.content,
                &mut self.css_chunks,
                self.out_dir,
                self.base,
            );
            html.replace_range(block.start..block.end, &replacement);
        } else if let Some(block) = find_first_tag_block(&html, STYLE_OPEN, STYLE_CLOSE) {
            let replacement = build_style_replacement(
                &block.content,
                &mut self.css_chunks,
                self.out_dir,
                self.base,
            );
            html.replace_range(block.start..block.end, &replacement);
        }

        if let Some(block) = find_marked_block(
            &html,
            SCRIPT_BLOCK_START,
            SCRIPT_BLOCK_END,
            SCRIPT_OPEN,
            SCRIPT_CLOSE,
        ) {
            let replacement = build_script_replacement(
                &block.content,
                &mut self.js_chunks,
                self.out_dir,
                self.base,
            );
            html.replace_range(block.start..block.end, &replacement);
        } else if let Some(block) = find_last_body_script_block(&html) {
            let replacement = build_script_replacement(
                &block.content,
                &mut self.js_chunks,
                self.out_dir,
                self.base,
            );
            let replacement = if replacement.is_empty() {
                BODY_CLOSE.to_string()
            } else {
                format!("{replacement}\n{BODY_CLOSE}")
            };
            html.replace_range(block.start..block.end, &replacement);
        }

        GeneratedHtmlPage { html, ..page }
    }

    /// Finishes extraction and returns every unique shared asset.
    #[must_use]
    pub fn finish(self) -> Vec<SharedAsset> {
        self.css_chunks.into_chunks().into_iter().chain(self.js_chunks.into_chunks()).collect()
    }
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
    let mut externalizer = SharedAssetExternalizer::new(out_dir, base);
    let optimized_pages =
        pages.into_iter().map(|page| externalizer.externalize_page(page)).collect();
    let assets = externalizer.finish();

    ExternalizedAssets { pages: optimized_pages, assets }
}

/// Checks both exact opening tags in one pass. Framework hosts normally emit
/// attribute-bearing Vite tags (`<script type=...>`, `<style nonce=...>`),
/// which the extractor intentionally leaves alone. Bailing out here avoids the
/// marker, fallback, and closing-body scans for their often very large HTML.
fn contains_inline_asset_tag(html: &str) -> bool {
    let bytes = html.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = memchr::memchr(b'<', &bytes[cursor..]) {
        let start = cursor + relative;
        let tail = &bytes[start..];
        if tail.starts_with(STYLE_OPEN.as_bytes()) || tail.starts_with(SCRIPT_OPEN.as_bytes()) {
            return true;
        }
        cursor = start + 1;
    }
    false
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

    #[test]
    fn leaves_external_host_assets_unchanged() {
        let html = r#"<html><head><link rel="stylesheet" href="/assets/app.css"></head>
<body><script type="module" src="/assets/app.js"></script></body></html>"#
            .to_string();
        let pages = vec![GeneratedHtmlPage {
            input_path: "host.md".to_string(),
            output_path: "/tmp/site/host/index.html".to_string(),
            html: html.clone(),
        }];

        let result = externalize_shared_page_assets(pages, "/tmp/site", "/");

        assert!(result.assets.is_empty());
        assert_eq!(result.pages[0].html, html);
    }

    #[test]
    fn incrementally_externalizes_pages_with_one_shared_cache() {
        let mut externalizer = SharedAssetExternalizer::new("/tmp/site", "/docs/");
        let first = externalizer.externalize_page(GeneratedHtmlPage {
            input_path: "a.md".to_string(),
            output_path: "/tmp/site/a/index.html".to_string(),
            html: format_page("A"),
        });
        let second = externalizer.externalize_page(GeneratedHtmlPage {
            input_path: "b.md".to_string(),
            output_path: "/tmp/site/b/index.html".to_string(),
            html: format_page("B"),
        });
        let assets = externalizer.finish();

        assert_eq!(assets.len(), 3);
        let shared_css = &assets[0].public_path;
        assert!(first.html.contains(shared_css));
        assert!(second.html.contains(shared_css));
    }

    #[test]
    fn feature_js_sections_emit_only_needed_script_srcs() {
        let pages = vec![
            GeneratedHtmlPage {
                input_path: "plain.md".to_string(),
                output_path: "/tmp/site/plain/index.html".to_string(),
                html: format_sectioned_page("Plain", false),
            },
            GeneratedHtmlPage {
                input_path: "tabs.md".to_string(),
                output_path: "/tmp/site/tabs/index.html".to_string(),
                html: format_sectioned_page("Tabs", true),
            },
        ];

        let result = externalize_shared_page_assets(pages, "/tmp/site", "/docs/");
        let plain = &result.pages[0].html;
        let tabs = &result.pages[1].html;

        assert!(plain.contains("ox-content-core-"), "{plain}");
        assert!(!plain.contains("ox-content-search-"), "{plain}");
        assert!(!plain.contains("ox-content-tabs-"), "{plain}");
        assert!(!plain.contains("ox-code-play"), "{plain}");
        assert!(!plain.contains("island"), "{plain}");
        assert!(tabs.contains("ox-content-tabs-"), "{tabs}");
        assert!(
            result.assets.iter().any(|asset| asset.public_path.contains("ox-content-search-")),
            "{result:?}"
        );
    }

    fn format_sectioned_page(title: &str, tabs: bool) -> String {
        let tabs_js = if tabs {
            "// ox-content:js:tabs:start\nconst tabs = true;\n// ox-content:js:tabs:end\n"
        } else {
            ""
        };
        format!(
            r#"<!doctype html>
<html>
<head></head>
<body>
  <h1>{title}</h1>
  <!-- ox-content:scripts:start -->
  <script>// ox-content:js:core:start
const boot = "__OX_CONTENT_SEARCH_CHUNK__";
// ox-content:search:start
const searchData = true;
// ox-content:search:end
// ox-content:js:core:end
{tabs_js}</script>
  <!-- ox-content:scripts:end -->
</body>
</html>"#
        )
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
