use askama::Template;

use super::a11y::A11Y_CSS;
use super::breadcrumbs::resolve_breadcrumbs;
use super::entry::generate_entry_html;
use super::footer::{FOOTER_CSS, generate_footer_html};
use super::locale_switcher::render_locale_switcher;
use super::nav::generate_nav_html;
use super::pagination::resolve_pager;
use super::reader_chrome::{READER_CHROME_CSS, READER_CHROME_JS, apply_reader_chrome};
use super::social::{generate_mobile_social_links_html, generate_social_links_html};
use super::theme_css::generate_theme_css;
use super::utils::{
    format_last_updated, generate_toc_html, html_locale_attrs, page_content_contains_any,
    wrap_css_section,
};
use super::{
    BarePageTemplate, ENTRY_CSS, GITHUB_CSS, ISLAND_CSS, MERMAID_CSS, NavGroup, OGP_CSS, PageData,
    PageTemplate, SOCIAL_CSS, SSG_CSS, SSG_JS, SsgConfig, TABS_CSS, TABS_JS, YOUTUBE_CSS,
};

/// Generates a complete HTML page for SSG.
///
/// This function creates a full HTML document with navigation sidebar,
/// content area, table of contents, search functionality, and theme toggle.
pub fn generate_html(page_data: &PageData, nav_groups: &[NavGroup], config: &SsgConfig) -> String {
    let nav_html = generate_nav_html(nav_groups, &page_data.path);

    // Theme configuration
    let theme = config.theme.as_ref();
    let embed = theme.and_then(|t| t.embed.as_ref());

    // Generate theme CSS overrides
    let theme_css = theme.map_or(String::new(), generate_theme_css);

    // Check if we have a footer
    let has_footer = theme.is_some_and(|t| {
        t.footer.as_ref().is_some_and(|f| f.message.is_some() || f.copyright.is_some())
    });
    let footer_css = if has_footer { FOOTER_CSS } else { "" };

    // Check if this is an entry page
    let is_entry_page = page_data.entry_page.is_some();
    // Build CSS as named sections instead of one anonymous blob. Shared,
    // content-addressed extraction can then pull out only the sections that are
    // globally cacheable and keep page-specific or relative-url CSS inline.
    let mut css_sections = vec![wrap_css_section("base", SSG_CSS)];

    if is_entry_page {
        css_sections.push(wrap_css_section("entry", ENTRY_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-tabs", "ox-tab-panel"]) {
        css_sections.push(wrap_css_section("plugin-tabs", TABS_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-youtube"]) {
        css_sections.push(wrap_css_section("plugin-youtube", YOUTUBE_CSS));
    }
    if page_content_contains_any(
        &page_data.content,
        &["ox-github-card", "ox-github-code", "ox-github-error"],
    ) {
        css_sections.push(wrap_css_section("plugin-github", GITHUB_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-ogp-card", "ox-ogp-simple"]) {
        css_sections.push(wrap_css_section("plugin-ogp", OGP_CSS));
    }
    if page_content_contains_any(
        &page_data.content,
        &["ox-tweet", "ox-bluesky", "ox-webcontainer", "ox-spotify", "ox-stackblitz"],
    ) {
        css_sections.push(wrap_css_section("plugin-social", SOCIAL_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-mermaid"]) {
        css_sections.push(wrap_css_section("plugin-mermaid", MERMAID_CSS));
    }
    if page_content_contains_any(&page_data.content, &["data-ox-island", "ox-island"]) {
        css_sections.push(wrap_css_section("plugin-island", ISLAND_CSS));
    }
    if has_footer {
        css_sections.push(wrap_css_section("footer", footer_css));
    }
    if config.reader_chrome.is_enabled() {
        css_sections.push(wrap_css_section("reader-chrome", READER_CHROME_CSS));
    }
    if config.a11y.is_enabled() {
        css_sections.push(wrap_css_section("a11y", A11Y_CSS));
    }
    if !theme_css.is_empty() {
        css_sections.push(wrap_css_section("theme", &theme_css));
    }

    let all_css = css_sections.join("");
    let toc_html = generate_toc_html(&page_data.toc);
    let has_toc = super::aside::has_toc(super::aside::aside_enabled(theme), &toc_html);
    let pager = resolve_pager(page_data, nav_groups, config.pagination);
    let breadcrumbs = resolve_breadcrumbs(page_data, nav_groups, config);
    let last_updated = page_data.last_updated.and_then(format_last_updated);

    // Embedded HTML for specific positions
    let embed_head = embed.and_then(|e| e.head.as_deref()).unwrap_or("");
    let embed_header_before = embed.and_then(|e| e.header_before.as_deref()).unwrap_or("");
    let embed_header_after = embed.and_then(|e| e.header_after.as_deref()).unwrap_or("");
    let embed_sidebar_before = embed.and_then(|e| e.sidebar_before.as_deref()).unwrap_or("");
    let embed_sidebar_after = embed.and_then(|e| e.sidebar_after.as_deref()).unwrap_or("");
    let embed_content_before = embed.and_then(|e| e.content_before.as_deref()).unwrap_or("");
    let embed_content_after = embed.and_then(|e| e.content_after.as_deref()).unwrap_or("");
    let embed_footer_before = embed.and_then(|e| e.footer_before.as_deref()).unwrap_or("");

    // Footer HTML
    let footer_html = if let Some(embed_footer) = embed.and_then(|e| e.footer.clone()) {
        embed_footer
    } else if let Some(t) = theme {
        generate_footer_html(t)
    } else {
        String::new()
    };

    // Header logo customization
    let header_config = theme.and_then(|t| t.header.as_ref());
    let logo_url = header_config
        .and_then(|h| h.logo.as_ref())
        .map_or_else(|| "logo.svg", std::string::String::as_str);
    let logo_width = header_config.and_then(|h| h.logo_width).unwrap_or(28);
    let logo_height = header_config.and_then(|h| h.logo_height).unwrap_or(28);
    let show_site_name_text = header_config.and_then(|h| h.show_site_name_text).unwrap_or(true);

    let resolve_theme_asset = |url: &str| {
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with('/') {
            url.to_string()
        } else {
            format!("{}{}", config.base, url)
        }
    };

    // Build logo src (prepend base if not absolute URL)
    let logo_src = resolve_theme_asset(logo_url);
    let logo_light_src =
        header_config.and_then(|h| h.logo_light.as_deref()).map(resolve_theme_asset);
    let logo_dark_src = header_config.and_then(|h| h.logo_dark.as_deref()).map(resolve_theme_asset);

    // Custom JS
    let custom_js = theme.and_then(|t| t.js.as_deref()).unwrap_or("");
    let mut all_js =
        format!("{}\n{}\n{}", SSG_JS.replace("{{base}}", &config.base), TABS_JS, custom_js);
    if config.reader_chrome.needs_js() {
        all_js.push('\n');
        all_js.push_str(READER_CHROME_JS);
    }

    // Social links
    let locale_switcher_html = render_locale_switcher(config);
    let social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), generate_social_links_html);

    // Mobile footer social links
    let mobile_social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), generate_mobile_social_links_html);

    let enhanced_content;
    let article_html = if config.reader_chrome.copy || config.reader_chrome.external_links {
        enhanced_content = apply_reader_chrome(&page_data.content, config.reader_chrome);
        enhanced_content.as_str()
    } else {
        page_data.content.as_str()
    };

    // Generate entry page content if applicable
    let (page_class, main_content) = if let Some(ref entry) = page_data.entry_page {
        let entry_html = generate_entry_html(entry, &config.base);
        // Entry page: hero/features + optional markdown content
        let combined = if article_html.trim().is_empty() {
            entry_html
        } else {
            format!(
                "{entry_html}\n<div class=\"entry-content\">\n  <div class=\"content\">\n{article_html}\n  </div>\n</div>"
            )
        };
        ("entry-page", combined)
    } else {
        ("", format!("<article class=\"content\">\n{article_html}\n      </article>"))
    };

    let mut body_classes = Vec::new();
    if !page_class.is_empty() {
        body_classes.push(page_class.to_string());
    }
    if is_entry_page
        && theme.and_then(|t| t.entry_page.as_ref()).and_then(|entry| entry.mode.as_deref())
            == Some("subtle")
    {
        body_classes.push("entry-page--subtle".to_string());
    }
    let body_class = body_classes.join(" ");

    let document_title = if page_data.title.trim() == config.site_name.trim() {
        config.site_name.clone()
    } else {
        format!("{} - {}", page_data.title, config.site_name)
    };
    let (html_lang, html_dir) = html_locale_attrs(config);

    let skip_link = config.a11y.skip_link_html();
    let template = PageTemplate {
        html_lang,
        html_dir,
        site_name: &config.site_name,
        document_title: &document_title,
        description: page_data.description.as_deref(),
        og_image: config.og_image.as_deref(),
        css: &all_css,
        embed_head,
        body_class: &body_class,
        skip_link: skip_link.as_deref(),
        embed_header_before,
        embed_header_after,
        base: &config.base,
        logo_src: &logo_src,
        logo_light_src: logo_light_src.as_deref(),
        logo_dark_src: logo_dark_src.as_deref(),
        show_site_name_text,
        logo_width,
        logo_height,
        social_links: &social_links_html,
        locale_switcher: &locale_switcher_html,
        is_entry_page,
        embed_sidebar_before,
        navigation: &nav_html,
        embed_sidebar_after,
        embed_content_before,
        breadcrumbs: breadcrumbs.as_ref(),
        main_content: &main_content,
        has_toc,
        toc_html: &toc_html,
        pager: pager.as_ref(),
        reader_chrome: config.reader_chrome.is_enabled().then_some(&config.reader_chrome),
        last_updated: last_updated.as_ref(),
        embed_content_after,
        embed_footer_before,
        footer_html: &footer_html,
        mobile_social_links: &mobile_social_links_html,
        js: &all_js,
    };

    template.render().unwrap_or_default()
}

/// Everything the bare template can put around a rendered page body.
///
/// Bare mode leaves the shell to the consumer, but the metadata below is
/// already computed for the themed page and is not something a consumer can
/// recover afterwards — the generated OG image in particular is only
/// discoverable by guessing at the output directory. Every field is optional,
/// and a `BarePageData` carrying none of them renders exactly the document
/// bare mode emitted before: no `<meta>` beyond charset and viewport, which
/// keeps the no-JS size baseline honest.
#[derive(Default)]
pub struct BarePageData<'a> {
    /// Page title, used for `<title>` and the OG/Twitter title.
    pub title: &'a str,
    /// Rendered page body.
    pub content: &'a str,
    /// `lang` attribute. Defaults to `en` when empty.
    pub lang: &'a str,
    /// `dir` attribute. Omitted entirely when empty.
    pub dir: &'a str,
    /// Page description, used for `description` and the OG/Twitter variants.
    pub description: Option<&'a str>,
    /// Absolute page URL, used for `<link rel="canonical">` and `og:url`.
    pub canonical_url: Option<&'a str>,
    /// Site name for `og:site_name`.
    pub site_name: Option<&'a str>,
    /// Image URL for `og:image` and `twitter:image`.
    pub og_image: Option<&'a str>,
    /// Raw markup appended to `<head>`.
    pub head: &'a str,
    /// Raw markup inserted directly after `<body>`.
    pub body_start: &'a str,
    /// Raw markup inserted directly before `</body>`.
    pub body_end: &'a str,
}

/// Generates a bare HTML page for SSG.
///
/// This page intentionally omits navigation, styles, and scripts.
pub fn generate_bare_html(content: &str, title: &str) -> String {
    generate_bare_page(&BarePageData { title, content, ..BarePageData::default() })
}

/// Generates a bare HTML page with whatever head metadata and injected markup
/// the caller has.
pub fn generate_bare_page(data: &BarePageData<'_>) -> String {
    let has_metadata = data.description.is_some()
        || data.canonical_url.is_some()
        || data.site_name.is_some()
        || data.og_image.is_some();

    BarePageTemplate {
        lang: if data.lang.is_empty() { "en" } else { data.lang },
        dir: data.dir,
        title: data.title,
        content: data.content,
        has_metadata,
        description: data.description,
        canonical_url: data.canonical_url,
        site_name: data.site_name,
        og_image: data.og_image,
        head: data.head,
        body_start: data.body_start,
        body_end: data.body_end,
    }
    .render()
    .unwrap_or_default()
}
