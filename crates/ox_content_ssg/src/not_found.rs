//! Opt-in custom 404 page planned and rendered by the SSG.

use crate::html::{NavGroup, PageData, SsgConfig, generate_html};
use crate::routes::{get_output_path, get_url_path};
use crate::site_maps::SiteMapPage;

/// Default Markdown file under `srcDir`.
pub const DEFAULT_SOURCE: &str = "404.md";
const DEFAULT_TITLE: &str = "Page not found";
const ROBOTS_NOINDEX: &str = r#"<meta name="robots" content="noindex">"#;

/// Switches for the custom 404 page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotFoundOptions {
    /// When false, no 404 page is produced.
    pub enabled: bool,
    /// Markdown path relative to `srcDir`.
    pub source: String,
}

/// Inputs for rendering a 404 page without writing files.
#[derive(Debug, Clone)]
pub struct NotFoundRequest<'a> {
    /// Content root used to resolve output paths.
    pub src_dir: &'a str,
    /// SSG output directory.
    pub out_dir: &'a str,
    /// Generated page extension, including the leading dot.
    pub extension: &'a str,
    /// Site base path.
    pub base: &'a str,
    /// Site name used by the default theme.
    pub site_name: &'a str,
    /// Source Markdown when the file exists.
    pub markdown: Option<&'a str>,
    /// Already-rendered HTML body. When omitted, the Markdown body is used.
    pub rendered_html: Option<&'a str>,
    /// Navigation shown by the default theme chrome.
    pub nav_groups: &'a [NavGroup],
}

/// One 404 page considered for crawl manifests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotFoundPage {
    /// Display title extracted from frontmatter.
    pub title: String,
    /// Route path without extension.
    pub url_path: String,
    /// Always true so search and sitemaps skip this page.
    pub noindex: bool,
    /// Draft-like flag consumed by `generate_site_maps`.
    pub draft: bool,
}

/// Generated 404 HTML and metadata, or a skip warning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotFoundOutput {
    /// Themed HTML document.
    pub html: Option<String>,
    /// Output file path matching the site's SSG URL style.
    pub output_path: Option<String>,
    /// Page metadata used by sitemaps and search.
    pub page: Option<NotFoundPage>,
    /// Non-fatal skip reason. Never used as a panic.
    pub warning: Option<String>,
}

impl Default for NotFoundOptions {
    fn default() -> Self {
        Self { enabled: false, source: DEFAULT_SOURCE.to_string() }
    }
}

/// Builds a 404 page without writing files.
pub fn generate_not_found(
    options: &NotFoundOptions,
    request: &NotFoundRequest<'_>,
) -> NotFoundOutput {
    if !options.enabled {
        return NotFoundOutput::default();
    }

    let Some(markdown) = request.markdown else {
        return NotFoundOutput {
            warning: Some(missing_source_warning(&options.source)),
            ..NotFoundOutput::default()
        };
    };

    let title = extract_frontmatter_title(markdown).unwrap_or_else(|| DEFAULT_TITLE.to_string());
    let body = request.rendered_html.map_or_else(|| strip_frontmatter(markdown), ToOwned::to_owned);
    let virtual_input = virtual_not_found_input(request.src_dir);
    let output_path =
        get_output_path(&virtual_input, request.src_dir, request.out_dir, request.extension);
    let url_path = get_url_path(&virtual_input, request.src_dir);
    let page_data = PageData {
        title: title.clone(),
        description: None,
        content: body,
        toc: Vec::new(),
        last_updated: None,
        path: url_path.clone(),
        entry_page: None,
        prev: None,
        next: None,
    };
    let config = SsgConfig {
        site_name: request.site_name.to_string(),
        base: request.base.to_string(),
        og_image: None,
        theme: None,
        locale: None,
        available_locales: None,
        pagination: false,
    };
    let html = inject_robots_noindex(&generate_html(&page_data, request.nav_groups, &config));

    NotFoundOutput {
        html: Some(html),
        output_path: Some(output_path),
        page: Some(NotFoundPage { title, url_path, noindex: true, draft: true }),
        warning: None,
    }
}

/// Maps a 404 page onto the sitemap input shape so drafts / noindex are skipped.
#[must_use]
pub fn not_found_sitemap_page(page: &NotFoundPage, loc: impl Into<String>) -> SiteMapPage {
    SiteMapPage {
        loc: loc.into(),
        title: page.title.clone(),
        description: None,
        draft: page.draft,
        noindex: page.noindex,
    }
}

/// Warning emitted when the feature is on but the Markdown source is missing.
#[must_use]
pub fn missing_source_warning(source: &str) -> String {
    format!(
        "[ox-content] notFound is enabled but {source} was not found; the 404 page was not written"
    )
}

fn virtual_not_found_input(src_dir: &str) -> String {
    let trimmed = src_dir.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        DEFAULT_SOURCE.to_string()
    } else {
        format!("{trimmed}/{DEFAULT_SOURCE}")
    }
}

fn extract_frontmatter_title(markdown: &str) -> Option<String> {
    let block = frontmatter_block(markdown)?;
    for line in block.lines() {
        let line = line.trim();
        let Some(value) = line.strip_prefix("title:") else {
            continue;
        };
        let value = unwrap_yaml_scalar(value.trim());
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn strip_frontmatter(markdown: &str) -> String {
    let Some(_) = frontmatter_block(markdown) else {
        return markdown.to_string();
    };
    let start = markdown.find("---").unwrap_or(0);
    let after_open = start + 3;
    let close_rel = markdown[after_open..].find("\n---").unwrap_or(0);
    markdown[after_open + close_rel + 4..].trim_start_matches('\n').to_string()
}

fn frontmatter_block(markdown: &str) -> Option<&str> {
    let rest = markdown.trim_start();
    let rest = rest.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(rest[..end].trim_start_matches('\n'))
}

fn unwrap_yaml_scalar(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn inject_robots_noindex(html: &str) -> String {
    if html.contains(ROBOTS_NOINDEX) {
        return html.to_string();
    }
    html.replacen("<head>", &format!("<head>\n  {ROBOTS_NOINDEX}"), 1)
}

#[cfg(test)]
mod tests;
