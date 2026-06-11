//! Shared asset extraction for generated SSG pages.

use rustc_hash::FxHashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STYLE_BLOCK_START: &str = "<!-- ox-content:styles:start -->";
const STYLE_BLOCK_END: &str = "<!-- ox-content:styles:end -->";
const SCRIPT_BLOCK_START: &str = "<!-- ox-content:scripts:start -->";
const SCRIPT_BLOCK_END: &str = "<!-- ox-content:scripts:end -->";
const STYLE_OPEN: &str = "<style>";
const STYLE_CLOSE: &str = "</style>";
const SCRIPT_OPEN: &str = "<script>";
const SCRIPT_CLOSE: &str = "</script>";
const BODY_CLOSE: &str = "</body>";
const CSS_SECTION_PREFIX: &str = "/* ox-content:css:";
const CSS_SECTION_START_SUFFIX: &str = ":start */";
const CSS_SECTION_END_SUFFIX: &str = ":end */";
const SEARCH_CHUNK_START: &str = "// ox-content:search:start";
const SEARCH_CHUNK_END: &str = "// ox-content:search:end";
const SEARCH_CHUNK_PLACEHOLDER: &str = "__OX_CONTENT_SEARCH_CHUNK__";
const CORE_CSS_SECTION_NAMES: [&str; 2] = ["base", "footer"];
const THEME_INLINE_CSS_MAX_BYTES: usize = 2048;

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

#[derive(Debug, Clone)]
struct CssSection {
    name: String,
    content: String,
}

#[derive(Debug, Clone, Copy)]
enum AssetKind {
    Css,
    Js,
}

impl AssetKind {
    const fn extension(self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Js => "js",
        }
    }
}

struct BlockMatch {
    start: usize,
    end: usize,
    content: String,
}

#[derive(Default)]
struct AssetCache {
    indexes_by_content: FxHashMap<String, usize>,
    chunks: Vec<SharedAsset>,
}

impl AssetCache {
    /// Returns an existing content-addressed chunk or creates it once.
    ///
    /// Multiple pages often share identical base CSS, plugin CSS, and client
    /// runtime JS. Keying by full content means each unique payload is hashed
    /// and written once, while every page receives the same public URL.
    fn get_or_create(
        &mut self,
        kind: AssetKind,
        label: &str,
        content: &str,
        out_dir: &str,
        base: &str,
    ) -> &SharedAsset {
        if let Some(index) = self.indexes_by_content.get(content).copied() {
            return &self.chunks[index];
        }

        let index = self.chunks.len();
        self.chunks.push(create_shared_asset_chunk(kind, label, content, out_dir, base));
        self.indexes_by_content.insert(content.to_string(), index);
        &self.chunks[index]
    }

    fn into_chunks(self) -> Vec<SharedAsset> {
        self.chunks
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

fn build_style_replacement(
    css_content: &str,
    css_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // CSS is split by `ox-content:css:*` section markers so globally shared
    // base/footer/plugin styles can become cacheable files while page-specific
    // theme overrides may stay inline. Without this split every page's full
    // `<style>` block would differ as soon as theme CSS differed.
    let sections = extract_css_sections(css_content);
    let effective_sections = if sections.is_empty() {
        vec![CssSection { name: "css".to_string(), content: css_content.trim().to_string() }]
    } else {
        sections
    };

    let core_content = effective_sections
        .iter()
        .filter(|section| CORE_CSS_SECTION_NAMES.contains(&section.name.as_str()))
        .map(|section| section.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    let mut fragments = Vec::new();
    if !core_content.is_empty() {
        let core_chunk =
            css_chunks.get_or_create(AssetKind::Css, "core", &core_content, out_dir, base);
        fragments.push(format!("  <link rel=\"stylesheet\" href=\"{}\">", core_chunk.public_path));
    }

    for section in effective_sections {
        if CORE_CSS_SECTION_NAMES.contains(&section.name.as_str()) {
            continue;
        }

        let has_relative_urls = has_relative_css_urls(&section.content);
        let should_inline_theme = section.name == "theme"
            && (has_relative_urls || section.content.len() <= THEME_INLINE_CSS_MAX_BYTES);
        if should_inline_theme || has_relative_urls {
            fragments.push(format!("  <style>{}</style>", section.content));
            continue;
        }

        let chunk = css_chunks.get_or_create(
            AssetKind::Css,
            &section.name,
            &section.content,
            out_dir,
            base,
        );
        fragments.push(format!("  <link rel=\"stylesheet\" href=\"{}\">", chunk.public_path));
    }

    fragments.join("\n")
}

fn build_script_replacement(
    js_content: &str,
    js_chunks: &mut AssetCache,
    out_dir: &str,
    base: &str,
) -> String {
    // The search UI runtime is large and only needed after the user opens
    // search. Split it into its own deferred chunk and replace the placeholder
    // in the core runtime with that chunk URL. If the marker is absent, fall
    // back to one shared JS asset for the whole script block.
    if let Some(search_chunk) = find_search_chunk(js_content) {
        if js_content.contains(SEARCH_CHUNK_PLACEHOLDER) {
            let search_content = search_chunk.content.trim();
            if !search_content.is_empty() {
                let search_public_path = js_chunks
                    .get_or_create(AssetKind::Js, "search", search_content, out_dir, base)
                    .public_path
                    .clone();
                let mut core_content = String::new();
                core_content.push_str(&js_content[..search_chunk.start]);
                core_content.push_str(&js_content[search_chunk.end..]);
                let core_content = core_content
                    .replace(SEARCH_CHUNK_PLACEHOLDER, &search_public_path)
                    .trim()
                    .to_string();

                if !core_content.is_empty() {
                    let core_chunk = js_chunks.get_or_create(
                        AssetKind::Js,
                        "core",
                        &core_content,
                        out_dir,
                        base,
                    );
                    return format!("  <script defer src=\"{}\"></script>", core_chunk.public_path);
                }
            }
        }
    }

    let fallback_content = js_content.trim();
    if fallback_content.is_empty() {
        return String::new();
    }

    let chunk = js_chunks.get_or_create(AssetKind::Js, "js", fallback_content, out_dir, base);
    format!("  <script defer src=\"{}\"></script>", chunk.public_path)
}

fn create_shared_asset_chunk(
    kind: AssetKind,
    label: &str,
    content: &str,
    out_dir: &str,
    base: &str,
) -> SharedAsset {
    let hash = create_content_hash(content);
    let file_name =
        format!("ox-content-{}-{hash}.{}", sanitize_chunk_label(label), kind.extension());
    let output_path =
        Path::new(out_dir).join("assets").join(&file_name).to_string_lossy().to_string();

    SharedAsset {
        output_path,
        public_path: to_public_asset_path(base, &file_name),
        content: content.to_string(),
    }
}

fn create_content_hash(content: &str) -> String {
    // Five SHA-256 bytes are enough for stable cache-busting filenames here:
    // chunks are generated per site build, not used as a security boundary, and
    // the content map above still de-duplicates exact matches before hashing.
    let hash = Sha256::digest(content.as_bytes());
    let mut output = String::with_capacity(10);
    for byte in hash.iter().take(5) {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn sanitize_chunk_label(label: &str) -> String {
    let mut output = String::new();
    let mut pending_dash = false;

    for ch in label.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !output.is_empty() {
                output.push('-');
            }
            output.push(ch);
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }

    if output.is_empty() {
        "asset".to_string()
    } else {
        output
    }
}

fn to_public_asset_path(base: &str, file_name: &str) -> String {
    let normalized_base = if base.ends_with('/') { base.to_string() } else { format!("{base}/") };
    format!("{normalized_base}assets/{file_name}")
}

fn extract_css_sections(css_content: &str) -> Vec<CssSection> {
    // Section markers are emitted as CSS comments, so they survive template
    // rendering and minification-free builds. Parse with string searches rather
    // than regex to keep this pass cheap over the full page stylesheet.
    let mut sections = Vec::new();
    let mut cursor = 0;

    while let Some(start_rel) = css_content[cursor..].find(CSS_SECTION_PREFIX) {
        let marker_start = cursor + start_rel;
        let name_start = marker_start + CSS_SECTION_PREFIX.len();
        let Some(name_end_rel) = css_content[name_start..].find(CSS_SECTION_START_SUFFIX) else {
            break;
        };
        let name_end = name_start + name_end_rel;
        let name = &css_content[name_start..name_end];
        if !name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
            cursor = name_end;
            continue;
        }

        let content_start = name_end + CSS_SECTION_START_SUFFIX.len();
        let end_marker = format!("{CSS_SECTION_PREFIX}{name}{CSS_SECTION_END_SUFFIX}");
        let Some(end_rel) = css_content[content_start..].find(&end_marker) else {
            break;
        };
        let content_end = content_start + end_rel;
        let content = css_content[content_start..content_end].trim();
        if !content.is_empty() {
            sections.push(CssSection { name: name.to_string(), content: content.to_string() });
        }
        cursor = content_end + end_marker.len();
    }

    sections
}

fn has_relative_css_urls(css: &str) -> bool {
    // Relative URLs are resolved relative to the HTML page when CSS is inline,
    // but relative to `/assets/...` after extraction. Detect those sections and
    // leave them inline rather than rewriting every CSS URL.
    let mut cursor = 0;
    let bytes = css.as_bytes();

    while let Some(url_rel) = css[cursor..].find("url(") {
        let url_index = cursor + url_rel;
        let mut value_start = url_index + 4;
        while value_start < bytes.len() && bytes[value_start].is_ascii_whitespace() {
            value_start += 1;
        }

        let quote = match bytes.get(value_start).copied() {
            Some(b'"' | b'\'') => {
                let quote = bytes[value_start];
                value_start += 1;
                Some(quote)
            }
            _ => None,
        };

        let mut value_end = value_start;
        while value_end < bytes.len() {
            let byte = bytes[value_end];
            if let Some(quote) = quote {
                if byte == b'\\' {
                    value_end = value_end.saturating_add(2);
                    continue;
                }
                if byte == quote {
                    break;
                }
            } else if byte == b')' {
                break;
            }
            value_end += 1;
        }

        let value = css[value_start..value_end].trim();
        if !value.is_empty()
            && !value.starts_with("data:")
            && !value.starts_with("http:")
            && !value.starts_with("https:")
            && !value.starts_with("//")
            && !value.starts_with('/')
            && !value.starts_with('#')
            && !value.starts_with("blob:")
            && !value.starts_with("var(")
        {
            return true;
        }

        cursor = value_end.saturating_add(1);
    }

    false
}

fn find_marked_block(
    html: &str,
    block_start_marker: &str,
    block_end_marker: &str,
    inner_open: &str,
    inner_close: &str,
) -> Option<BlockMatch> {
    let marker_start = html.find(block_start_marker)?;
    let start = include_leading_horizontal_ws(html, marker_start);
    let open_start = marker_start + html[marker_start..].find(inner_open)?;
    let content_start = open_start + inner_open.len();
    let content_end = content_start + html[content_start..].find(inner_close)?;
    let close_end = content_end + inner_close.len();
    let block_end = close_end + html[close_end..].find(block_end_marker)? + block_end_marker.len();

    Some(BlockMatch {
        start,
        end: block_end,
        content: html[content_start..content_end].to_string(),
    })
}

fn find_first_tag_block(html: &str, open: &str, close: &str) -> Option<BlockMatch> {
    let open_start = html.find(open)?;
    let start = include_leading_horizontal_ws(html, open_start);
    let content_start = open_start + open.len();
    let content_end = content_start + html[content_start..].find(close)?;
    Some(BlockMatch {
        start,
        end: content_end + close.len(),
        content: html[content_start..content_end].to_string(),
    })
}

fn find_last_body_script_block(html: &str) -> Option<BlockMatch> {
    let body_start = html.rfind(BODY_CLOSE)?;
    let before_body = &html[..body_start];
    let script_start = before_body.rfind(SCRIPT_OPEN)?;
    let content_start = script_start + SCRIPT_OPEN.len();
    let content_end = content_start + html[content_start..body_start].find(SCRIPT_CLOSE)?;
    let script_end = content_end + SCRIPT_CLOSE.len();
    if !html[script_end..body_start].trim().is_empty() {
        return None;
    }

    Some(BlockMatch {
        start: include_leading_horizontal_ws(html, script_start),
        end: body_start + BODY_CLOSE.len(),
        content: html[content_start..content_end].to_string(),
    })
}

fn find_search_chunk(js_content: &str) -> Option<BlockMatch> {
    let start = js_content.find(SEARCH_CHUNK_START)?;
    let content_start = start + SEARCH_CHUNK_START.len();
    let end_start = content_start + js_content[content_start..].find(SEARCH_CHUNK_END)?;
    Some(BlockMatch {
        start,
        end: end_start + SEARCH_CHUNK_END.len(),
        content: js_content[content_start..end_start].trim().to_string(),
    })
}

fn include_leading_horizontal_ws(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor > 0 && matches!(bytes[cursor - 1], b' ' | b'\t') {
        cursor -= 1;
    }
    cursor
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
        assert!(result.pages[0]
            .html
            .contains("<link rel=\"stylesheet\" href=\"/docs/assets/ox-content-core-"));
        assert!(result.pages[0].html.contains("<script defer src=\"/docs/assets/ox-content-core-"));
        assert!(!result.pages[0].html.contains("ox-content:styles:start"));
        assert!(!result.pages[0].html.contains("const searchData = true;"));
        assert!(result.assets.iter().any(|asset| asset.output_path.ends_with(".css")));
        assert!(result.assets.iter().any(|asset| asset.output_path.ends_with(".js")));
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
        assert!(result.pages[0]
            .html
            .contains("<style>.hero{background:url('./hero.png')}</style>"));
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
