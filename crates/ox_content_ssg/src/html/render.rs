use askama::Template;

use super::entry::generate_entry_html;
use super::footer::{generate_footer_html, FOOTER_CSS};
use super::nav::generate_nav_html;
use super::social::{generate_mobile_social_links_html, generate_social_links_html};
use super::theme_css::generate_theme_css;
use super::utils::{
    format_last_updated, generate_toc_html, html_locale_attrs, page_content_contains_any,
    wrap_css_section,
};
use super::{
    BarePageTemplate, NavGroup, PageData, PageTemplate, SsgConfig, ENTRY_CSS, GITHUB_CSS,
    ISLAND_CSS, MERMAID_CSS, OGP_CSS, SOCIAL_CSS, SSG_CSS, SSG_JS, TABS_CSS, TABS_JS, YOUTUBE_CSS,
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
    if !theme_css.is_empty() {
        css_sections.push(wrap_css_section("theme", &theme_css));
    }

    let all_css = css_sections.join("");
    let toc_html = generate_toc_html(&page_data.toc);
    let has_toc = !toc_html.is_empty();
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
    let all_js =
        format!("{}\n{}\n{}", SSG_JS.replace("{{base}}", &config.base), TABS_JS, custom_js);

    // Social links
    let social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), generate_social_links_html);

    // Mobile footer social links
    let mobile_social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), generate_mobile_social_links_html);

    // Generate entry page content if applicable
    let (page_class, main_content) = if let Some(ref entry) = page_data.entry_page {
        let entry_html = generate_entry_html(entry, &config.base);
        // Entry page: hero/features + optional markdown content
        let combined = if page_data.content.trim().is_empty() {
            entry_html
        } else {
            format!(
                "{}\n<div class=\"entry-content\">\n  <div class=\"content\">\n{}\n  </div>\n</div>",
                entry_html, page_data.content
            )
        };
        ("entry-page", combined)
    } else {
        ("", format!("<article class=\"content\">\n{}\n      </article>", page_data.content))
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
        is_entry_page,
        embed_sidebar_before,
        navigation: &nav_html,
        embed_sidebar_after,
        embed_content_before,
        main_content: &main_content,
        has_toc,
        toc_html: &toc_html,
        last_updated: last_updated.as_ref(),
        embed_content_after,
        embed_footer_before,
        footer_html: &footer_html,
        mobile_social_links: &mobile_social_links_html,
        js: &all_js,
    };

    template.render().unwrap_or_default()
}

/// Generates a bare HTML page for SSG.
///
/// This page intentionally omits navigation, styles, and scripts.
pub fn generate_bare_html(content: &str, title: &str) -> String {
    BarePageTemplate { title, content }.render().unwrap_or_default()
}
