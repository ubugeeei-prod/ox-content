use askama::Template;

use super::a11y::A11Y_CSS;
use super::breadcrumbs::resolve_breadcrumbs;
use super::content_css::push_content_plugin_css;
use super::entry::generate_entry_html;
use super::footer::{FOOTER_CSS, generate_footer_html};
use super::head::{RenderedHead, render_themed_head};
use super::header_chrome::{
    enhance_article_html, markdown_source_chrome_enabled, push_header_chrome_body_classes,
    push_header_chrome_css, push_header_chrome_js, render_announcement, render_header_nav,
    resolve_page_chrome,
};
use super::heading_permalinks::{push_heading_permalink_body_class, push_heading_permalink_css};
use super::json_ld::render_json_ld;
use super::locale_switcher::render_locale_switcher;
use super::mpa_navigation::{MPA_NAVIGATION_CSS, THEME_BOOTSTRAP_JS, view_transitions_enabled};
use super::nav::generate_nav_html;
use super::not_by_ai::push_not_by_ai_css;
use super::pagination::resolve_pager;
use super::reader_chrome::{READER_CHROME_CSS, READER_CHROME_JS};
use super::section_index::SECTION_INDEX_CSS;
use super::social::{generate_mobile_social_links_html, generate_social_links_html};
use super::team::TEAM_CSS;
use super::theme_css::generate_theme_css;
use super::utils::{
    contributor_views, format_last_updated, generate_toc_html, html_locale_attrs,
    page_content_contains_any, wrap_css_section,
};
use super::{
    CONTRIBUTORS_CSS, ENTRY_CSS, FILE_TREE_CSS, GITHUB_CSS, ISLAND_CSS, MERMAID_CSS, NavGroup,
    OGP_CSS, PageData, PageTemplate, SOCIAL_CSS, SOCIAL_TWEET_FULL_CSS, SSG_CSS, SSG_JS, SsgConfig,
    TABS_CSS, TABS_JS, YOUTUBE_CSS,
};

pub(super) struct GeneratedPage {
    pub html: String,
    pub head: RenderedHead,
}

pub(super) fn generate_html_inner(
    page_data: &PageData,
    nav_groups: &[NavGroup],
    config: &SsgConfig,
) -> GeneratedPage {
    let theme = config.theme.as_ref();
    let chrome = resolve_page_chrome(
        config.page_chrome,
        page_data.chrome,
        super::aside::aside_enabled(theme),
    );
    let nav_html = if chrome.show_sidebar {
        generate_nav_html(nav_groups, &page_data.path)
    } else {
        String::new()
    };
    let header_nav_html =
        theme.and_then(|t| t.nav.as_deref()).map(render_header_nav).unwrap_or_default();
    let announcement_html =
        theme.and_then(|t| t.announcement.as_ref()).map(render_announcement).unwrap_or_default();
    let embed = theme.and_then(|t| t.embed.as_ref());

    let theme_css = theme.map_or(String::new(), generate_theme_css);
    let has_footer = chrome.show_footer
        && theme.is_some_and(|t| {
            t.footer.as_ref().is_some_and(|f| f.message.is_some() || f.copyright.is_some())
        });
    let footer_css = if has_footer { FOOTER_CSS } else { "" };
    let is_entry_page = page_data.entry_page.is_some();
    let mut reader_chrome = config.reader_chrome;
    if is_entry_page {
        reader_chrome.back_to_top = false;
    }
    let contributors = contributor_views(&page_data.contributors);
    let mut css_sections = vec![wrap_css_section("base", SSG_CSS.as_str())];

    if view_transitions_enabled(theme) {
        css_sections.push(wrap_css_section("mpa-navigation", MPA_NAVIGATION_CSS));
    }
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
        &[
            "ox-tweet",
            "ox-bluesky",
            "ox-webcontainer",
            "ox-spotify",
            "ox-apple-music",
            "ox-stackblitz",
        ],
    ) {
        css_sections.push(wrap_css_section("plugin-social", SOCIAL_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-tweet--full"]) {
        css_sections.push(wrap_css_section("plugin-social-tweet-full", SOCIAL_TWEET_FULL_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-mermaid"]) {
        css_sections.push(wrap_css_section("plugin-mermaid", MERMAID_CSS));
    }
    if page_content_contains_any(&page_data.content, &["data-ox-island", "ox-island"]) {
        css_sections.push(wrap_css_section("plugin-island", ISLAND_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-team"]) {
        css_sections.push(wrap_css_section("team", TEAM_CSS));
    }
    if !contributors.is_empty() {
        css_sections.push(wrap_css_section("contributors", CONTRIBUTORS_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-section-index"]) {
        css_sections.push(wrap_css_section("section-index", SECTION_INDEX_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-file-tree"]) {
        css_sections.push(wrap_css_section("file-tree", FILE_TREE_CSS));
    }
    push_not_by_ai_css(&mut css_sections, &page_data.content);
    push_content_plugin_css(&mut css_sections, &page_data.content);
    push_heading_permalink_css(&mut css_sections, &page_data.content);
    if has_footer {
        css_sections.push(wrap_css_section("footer", footer_css));
    }
    if reader_chrome.is_enabled() {
        css_sections.push(wrap_css_section("reader-chrome", READER_CHROME_CSS));
    }
    let markdown_source_chrome =
        !is_entry_page && markdown_source_chrome_enabled(page_data.markdown_source.as_deref());
    push_header_chrome_css(
        &mut css_sections,
        &header_nav_html,
        &announcement_html,
        chrome,
        markdown_source_chrome,
    );
    if config.a11y.is_enabled() {
        css_sections.push(wrap_css_section("a11y", A11Y_CSS));
    }
    if !theme_css.is_empty() {
        css_sections.push(wrap_css_section("theme", &theme_css));
    }

    let all_css = css_sections.join("");
    let toc_html = generate_toc_html(&page_data.toc);
    let has_toc = !is_entry_page && super::aside::has_toc(chrome.show_outline, &toc_html);
    let pager = resolve_pager(page_data, nav_groups, config.pagination);
    let breadcrumbs = resolve_breadcrumbs(page_data, nav_groups, config);
    let json_ld = render_json_ld(page_data, config, breadcrumbs.as_ref());
    let last_updated = chrome
        .show_last_updated
        .then(|| page_data.last_updated.and_then(format_last_updated))
        .flatten();

    let embed_head_user = embed.and_then(|e| e.head.as_deref()).unwrap_or("");
    let katex_head =
        if page_content_contains_any(&page_data.content, &["class=\"katex\"", "class='katex'"]) {
            format!("<link rel=\"stylesheet\" href=\"{}__ox_katex__/katex.min.css\">", config.base)
        } else {
            String::new()
        };
    let embed_head = if katex_head.is_empty() {
        embed_head_user.to_string()
    } else if embed_head_user.is_empty() {
        katex_head
    } else {
        format!("{katex_head}\n  {embed_head_user}")
    };
    let embed_header_before = embed.and_then(|e| e.header_before.as_deref()).unwrap_or("");
    let embed_header_after = embed.and_then(|e| e.header_after.as_deref()).unwrap_or("");
    let embed_sidebar_before = embed.and_then(|e| e.sidebar_before.as_deref()).unwrap_or("");
    let embed_sidebar_after = embed.and_then(|e| e.sidebar_after.as_deref()).unwrap_or("");
    let embed_content_before = embed.and_then(|e| e.content_before.as_deref()).unwrap_or("");
    let embed_content_after = embed.and_then(|e| e.content_after.as_deref()).unwrap_or("");
    let embed_footer_before = embed.and_then(|e| e.footer_before.as_deref()).unwrap_or("");

    let footer_html = if !chrome.show_footer {
        String::new()
    } else if let Some(embed_footer) = embed.and_then(|e| e.footer.clone()) {
        embed_footer
    } else if let Some(t) = theme {
        generate_footer_html(t)
    } else {
        String::new()
    };

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

    let logo_src = resolve_theme_asset(logo_url);
    let logo_light_src =
        header_config.and_then(|h| h.logo_light.as_deref()).map(resolve_theme_asset);
    let logo_dark_src = header_config.and_then(|h| h.logo_dark.as_deref()).map(resolve_theme_asset);
    let locale_switcher_html = render_locale_switcher(config);
    let custom_js = theme.and_then(|t| t.js.as_deref()).unwrap_or("");
    let mut all_js =
        format!("{}\n{}\n{}", SSG_JS.replace("{{base}}", &config.base), TABS_JS, custom_js);
    if reader_chrome.needs_js() {
        all_js.push('\n');
        all_js.push_str(READER_CHROME_JS);
    }
    push_header_chrome_js(
        &mut all_js,
        &header_nav_html,
        &announcement_html,
        &locale_switcher_html,
        markdown_source_chrome,
    );
    let self_hosted_icons = theme_has_self_hosted_icons(theme);
    let social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), |links| generate_social_links_html(links, self_hosted_icons));
    let mobile_social_links_html = theme
        .and_then(|t| t.social_links.as_ref())
        .map_or(String::new(), |links| generate_mobile_social_links_html(links, self_hosted_icons));
    let article_html = enhance_article_html(
        &page_data.content,
        reader_chrome,
        page_data.markdown_source.as_deref(),
        is_entry_page,
    );

    let (page_class, main_content) = if let Some(ref entry) = page_data.entry_page {
        let entry_html = generate_entry_html(entry, &config.base, self_hosted_icons);
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
    push_header_chrome_body_classes(&mut body_classes, &announcement_html, chrome);
    push_heading_permalink_body_class(&mut body_classes, theme, &page_data.content);
    let body_class = body_classes.join(" ");
    let page_head = render_themed_head(page_data, config, json_ld.as_deref());
    let (html_lang, html_dir) = html_locale_attrs(config);
    let skip_link = config.a11y.skip_link_html();
    let template = PageTemplate {
        html_lang,
        html_dir,
        site_name: &config.site_name,
        page_head: &page_head.html,
        theme_bootstrap_js: THEME_BOOTSTRAP_JS,
        css: &all_css,
        embed_head: &embed_head,
        body_class: &body_class,
        skip_link: skip_link.as_deref(),
        embed_header_before,
        announcement_html: &announcement_html,
        show_navbar: chrome.show_navbar,
        header_nav_html: &header_nav_html,
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
        reader_chrome: reader_chrome.is_enabled().then_some(&reader_chrome),
        last_updated: last_updated.as_ref(),
        contributors: &contributors,
        embed_content_after,
        embed_footer_before,
        footer_html: &footer_html,
        mobile_social_links: &mobile_social_links_html,
        js: &all_js,
    };

    GeneratedPage { html: template.render().unwrap_or_default(), head: page_head }
}

fn theme_has_self_hosted_icons(theme: Option<&super::ThemeConfig>) -> bool {
    theme
        .and_then(|t| t.embed.as_ref())
        .and_then(|e| e.head.as_deref())
        .is_some_and(|head| head.contains("__ox_icons__/"))
}
