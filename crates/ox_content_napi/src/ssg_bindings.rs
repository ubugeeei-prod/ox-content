use std::path::PathBuf;
use std::process::Command;

use napi_derive::napi;

use crate::{
    JsEntryPageConfig, JsSsgConfig, JsSsgExternalizedAssets, JsSsgGeneratedHtmlPage, JsSsgNavGroup,
    JsSsgNavItem, JsSsgNavigationGroup, JsSsgNavigationItem, JsSsgPageData, JsSsgRoutePaths,
    JsSsgSharedAsset, JsSsgSidebarItem, JsThemeColors, JsThemeConfig, TocEntry,
};

/// Returns the last git commit timestamp for a file in milliseconds.
#[napi]
pub fn get_git_last_updated(file_path: String, root: Option<String>) -> Option<f64> {
    let root = root.map(PathBuf::from)?;
    let file = PathBuf::from(&file_path);
    let pathspec = file.strip_prefix(&root).ok().and_then(|p| p.to_str()).unwrap_or(&file_path);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%ct", "--"])
        .arg(pathspec)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let seconds = String::from_utf8(output.stdout).ok()?.trim().parse::<f64>().ok()?;
    Some(seconds * 1_000.0)
}

// =============================================================================
// Theme Configuration Types for NAPI
/// Converts JsThemeColors to ox_content_ssg::ThemeColors.
fn convert_theme_colors(colors: Option<JsThemeColors>) -> Option<ox_content_ssg::ThemeColors> {
    colors.map(|c| ox_content_ssg::ThemeColors {
        primary: c.primary,
        primary_hover: c.primary_hover,
        background: c.background,
        background_alt: c.background_alt,
        text: c.text,
        text_muted: c.text_muted,
        border: c.border,
        code_background: c.code_background,
        code_text: c.code_text,
    })
}

/// Converts JsThemeConfig to ox_content_ssg::ThemeConfig.
fn convert_theme_config(theme: Option<JsThemeConfig>) -> Option<ox_content_ssg::ThemeConfig> {
    theme.map(|t| ox_content_ssg::ThemeConfig {
        colors: convert_theme_colors(t.colors),
        dark_colors: convert_theme_colors(t.dark_colors),
        fonts: t.fonts.map(|f| ox_content_ssg::ThemeFonts { sans: f.sans, mono: f.mono }),
        entry_page: t.entry_page.map(|entry| ox_content_ssg::ThemeEntryPage { mode: entry.mode }),
        layout: t.layout.map(|l| ox_content_ssg::ThemeLayout {
            sidebar_width: l.sidebar_width,
            header_height: l.header_height,
            max_content_width: l.max_content_width,
        }),
        header: t.header.map(|h| ox_content_ssg::ThemeHeader {
            logo: h.logo,
            logo_light: h.logo_light,
            logo_dark: h.logo_dark,
            show_site_name_text: h.show_site_name_text,
            logo_width: h.logo_width,
            logo_height: h.logo_height,
        }),
        footer: t
            .footer
            .map(|f| ox_content_ssg::ThemeFooter { message: f.message, copyright: f.copyright }),
        social_links: t.social_links.map(|s| ox_content_ssg::SocialLinks {
            github: s.github,
            twitter: s.twitter,
            discord: s.discord,
            links: s.links.map(|links| {
                links
                    .into_iter()
                    .map(|l| ox_content_ssg::SocialLink {
                        icon: l.icon,
                        icon_svg: l.icon_svg,
                        link: l.link,
                        aria_label: l.aria_label,
                    })
                    .collect()
            }),
        }),
        embed: t.embed.map(|e| ox_content_ssg::ThemeEmbed {
            head: e.head,
            header_before: e.header_before,
            header_after: e.header_after,
            sidebar_before: e.sidebar_before,
            sidebar_after: e.sidebar_after,
            content_before: e.content_before,
            content_after: e.content_after,
            footer_before: e.footer_before,
            footer: e.footer,
        }),
        css: t.css,
        js: t.js,
    })
}

/// Converts JsEntryPageConfig to ox_content_ssg::EntryPageConfig.
fn convert_entry_page_config(
    entry: Option<JsEntryPageConfig>,
) -> Option<ox_content_ssg::EntryPageConfig> {
    entry.map(|e| ox_content_ssg::EntryPageConfig {
        hero: e.hero.map(|h| ox_content_ssg::HeroConfig {
            name: h.name,
            text: h.text,
            tagline: h.tagline,
            notice: h
                .notice
                .map(|n| ox_content_ssg::HeroNoticeConfig { title: n.title, body: n.body }),
            image: h.image.map(|i| ox_content_ssg::HeroImage {
                src: i.src,
                light_src: i.light_src,
                dark_src: i.dark_src,
                alt: i.alt,
                width: i.width,
                height: i.height,
            }),
            actions: h.actions.map(|actions| {
                actions
                    .into_iter()
                    .map(|a| ox_content_ssg::HeroAction {
                        theme: a.theme,
                        text: a.text,
                        link: a.link,
                    })
                    .collect()
            }),
        }),
        features: e.features.map(|features| {
            features
                .into_iter()
                .map(|f| ox_content_ssg::FeatureConfig {
                    icon: f.icon,
                    title: f.title,
                    details: f.details,
                    link: f.link,
                    link_text: f.link_text,
                })
                .collect()
        }),
    })
}

fn convert_nav_item(item: JsSsgNavItem) -> ox_content_ssg::NavItem {
    ox_content_ssg::NavItem {
        title: item.title,
        path: item.path,
        href: item.href,
        children: item.children.unwrap_or_default().into_iter().map(convert_nav_item).collect(),
        collapsed: item.collapsed,
    }
}

fn map_nav_item(item: ox_content_ssg::NavItem) -> JsSsgNavItem {
    JsSsgNavItem {
        title: item.title,
        path: item.path,
        href: item.href,
        children: if item.children.is_empty() {
            None
        } else {
            Some(item.children.into_iter().map(map_nav_item).collect())
        },
        collapsed: item.collapsed,
    }
}

fn map_nav_group(group: ox_content_ssg::NavGroup) -> JsSsgNavGroup {
    JsSsgNavGroup {
        title: group.title,
        items: group.items.into_iter().map(map_nav_item).collect(),
        collapsed: group.collapsed,
    }
}

fn convert_sidebar_item(item: JsSsgSidebarItem) -> ox_content_ssg::SidebarItem {
    ox_content_ssg::SidebarItem {
        text: item.text,
        link: item.link,
        items: item.items.unwrap_or_default().into_iter().map(convert_sidebar_item).collect(),
        collapsed: item.collapsed,
    }
}

fn convert_navigation_item(item: JsSsgNavigationItem) -> ox_content_ssg::ManualNavigationItem {
    ox_content_ssg::ManualNavigationItem { title: item.title, path: item.path, href: item.href }
}

fn convert_navigation_group(group: JsSsgNavigationGroup) -> ox_content_ssg::ManualNavigationGroup {
    ox_content_ssg::ManualNavigationGroup {
        title: group.title,
        items: group.items.into_iter().map(convert_navigation_item).collect(),
    }
}

fn map_route_paths(paths: ox_content_ssg::RoutePaths) -> JsSsgRoutePaths {
    JsSsgRoutePaths {
        output_path: paths.output_path,
        url_path: paths.url_path,
        href: paths.href,
        og_image_path: paths.og_image_path,
        og_image_url: paths.og_image_url,
    }
}

fn convert_generated_html_page(page: JsSsgGeneratedHtmlPage) -> ox_content_ssg::GeneratedHtmlPage {
    ox_content_ssg::GeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

fn map_generated_html_page(page: ox_content_ssg::GeneratedHtmlPage) -> JsSsgGeneratedHtmlPage {
    JsSsgGeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

fn map_shared_asset(asset: ox_content_ssg::SharedAsset) -> JsSsgSharedAsset {
    JsSsgSharedAsset {
        output_path: asset.output_path,
        public_path: asset.public_path,
        content: asset.content,
    }
}

/// Resolves all output and public route paths for an SSG page.
#[napi(js_name = "resolveSsgRoutePaths")]
pub fn resolve_ssg_route_paths(
    input_path: String,
    src_dir: String,
    out_dir: String,
    base: String,
    extension: String,
    site_url: Option<String>,
) -> JsSsgRoutePaths {
    map_route_paths(ox_content_ssg::resolve_route_paths(
        &input_path,
        &src_dir,
        &out_dir,
        &base,
        &extension,
        site_url.as_deref(),
    ))
}

/// Converts a markdown file path to its corresponding SSG HTML output path.
#[napi(js_name = "getSsgOutputPath")]
pub fn get_ssg_output_path(
    input_path: String,
    src_dir: String,
    out_dir: String,
    extension: String,
) -> String {
    ox_content_ssg::get_output_path(&input_path, &src_dir, &out_dir, &extension)
}

/// Converts a markdown file path to a relative SSG URL path.
#[napi(js_name = "getSsgUrlPath")]
pub fn get_ssg_url_path(input_path: String, src_dir: String) -> String {
    ox_content_ssg::get_url_path(&input_path, &src_dir)
}

/// Converts a markdown file path to an SSG href.
#[napi(js_name = "getSsgHref")]
pub fn get_ssg_href(
    input_path: String,
    src_dir: String,
    base: String,
    extension: String,
) -> String {
    ox_content_ssg::get_href(&input_path, &src_dir, &base, &extension)
}

/// Resolves a page locale from an SSG URL path and configured locale codes.
#[napi(js_name = "getSsgPageLocale")]
pub fn get_ssg_page_locale(
    url_path: String,
    default_locale: String,
    locale_codes: Vec<String>,
) -> Option<String> {
    ox_content_ssg::get_page_locale(&url_path, &default_locale, &locale_codes)
}

/// Extracts a page title from frontmatter title or rendered HTML.
#[napi(js_name = "extractSsgTitle")]
pub fn extract_ssg_title(content: String, frontmatter_title: Option<String>) -> String {
    ox_content_ssg::extract_title(&content, frontmatter_title.as_deref())
}

/// Formats a file or directory segment as an SSG title.
#[napi(js_name = "formatSsgTitle")]
pub fn format_ssg_title(name: String) -> String {
    ox_content_ssg::format_title(&name)
}

/// Normalizes VitePress-specific frontmatter into ox-content's entry-page shape.
#[napi(js_name = "normalizeVitePressFrontmatter")]
pub fn normalize_vitepress_frontmatter(frontmatter: serde_json::Value) -> serde_json::Value {
    ox_content_ssg::normalize_vitepress_frontmatter(frontmatter)
}

/// Builds SSG navigation groups from markdown files.
#[napi(js_name = "buildSsgNavItems")]
pub fn build_ssg_nav_items(
    markdown_files: Vec<String>,
    src_dir: String,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    ox_content_ssg::build_nav_items(&markdown_files, &src_dir, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Builds SSG navigation groups from an explicit theme sidebar tree.
#[napi(js_name = "buildSsgThemeNavItems")]
pub fn build_ssg_theme_nav_items(
    sidebar: Vec<JsSsgSidebarItem>,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    let sidebar: Vec<ox_content_ssg::SidebarItem> =
        sidebar.into_iter().map(convert_sidebar_item).collect();
    ox_content_ssg::build_theme_nav_items(&sidebar, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Resolves manual SSG navigation groups.
#[napi(js_name = "resolveSsgNavigationGroups")]
pub fn resolve_ssg_navigation_groups(
    navigation: Vec<JsSsgNavigationGroup>,
    base: String,
    extension: String,
) -> Vec<JsSsgNavGroup> {
    let navigation: Vec<ox_content_ssg::ManualNavigationGroup> =
        navigation.into_iter().map(convert_navigation_group).collect();
    ox_content_ssg::resolve_navigation_groups(&navigation, &base, &extension)
        .into_iter()
        .map(map_nav_group)
        .collect()
}

/// Collects Markdown files for SSG from a source directory.
#[napi(js_name = "collectSsgMarkdownFiles")]
pub fn collect_ssg_markdown_files(src_dir: String, extensions: Vec<String>) -> Vec<String> {
    ox_content_ssg::collect_markdown_files(&src_dir, &extensions)
}

/// Generates SSG HTML page with navigation and search.
#[napi]
pub fn generate_ssg_html(
    page_data: JsSsgPageData,
    nav_groups: Vec<JsSsgNavGroup>,
    config: JsSsgConfig,
) -> String {
    // Convert NAPI types to ox_content_ssg types
    let ssg_page_data = ox_content_ssg::PageData {
        title: page_data.title,
        description: page_data.description,
        content: page_data.content,
        toc: flatten_toc_entries(page_data.toc),
        last_updated: page_data
            .last_updated
            .filter(|timestamp| timestamp.is_finite() && *timestamp >= 0.0)
            .map(|timestamp| timestamp as i64),
        path: page_data.path,
        entry_page: convert_entry_page_config(page_data.entry_page),
    };

    let ssg_nav_groups: Vec<ox_content_ssg::NavGroup> = nav_groups
        .into_iter()
        .map(|g| ox_content_ssg::NavGroup {
            title: g.title,
            items: g.items.into_iter().map(convert_nav_item).collect(),
            collapsed: g.collapsed,
        })
        .collect();

    let ssg_config = ox_content_ssg::SsgConfig {
        site_name: config.site_name,
        base: config.base,
        og_image: config.og_image,
        theme: convert_theme_config(config.theme),
        locale: config.locale,
        available_locales: config.available_locales.map(|locales| {
            locales
                .into_iter()
                .map(|l| ox_content_ssg::LocaleInfo { code: l.code, name: l.name, dir: l.dir })
                .collect()
        }),
    };

    ox_content_ssg::generate_html(&ssg_page_data, &ssg_nav_groups, &ssg_config)
}

fn flatten_toc_entries(entries: Vec<TocEntry>) -> Vec<ox_content_ssg::TocEntry> {
    let mut flat = Vec::new();
    for entry in entries {
        flat.push(ox_content_ssg::TocEntry {
            depth: entry.depth,
            text: entry.text,
            slug: entry.slug,
        });
        flat.extend(flatten_toc_entries(entry.children));
    }
    flat
}

/// Generates a bare SSG HTML page without navigation or styles.
#[napi(js_name = "generateSsgBareHtml")]
pub fn generate_ssg_bare_html(content: String, title: String) -> String {
    ox_content_ssg::generate_bare_html(&content, &title)
}

/// Extracts shared CSS and JavaScript assets from generated SSG pages.
#[napi(js_name = "externalizeSsgAssets")]
pub fn externalize_ssg_assets(
    pages: Vec<JsSsgGeneratedHtmlPage>,
    out_dir: String,
    base: String,
) -> JsSsgExternalizedAssets {
    let result = ox_content_ssg::externalize_shared_page_assets(
        pages.into_iter().map(convert_generated_html_page).collect(),
        &out_dir,
        &base,
    );

    JsSsgExternalizedAssets {
        pages: result.pages.into_iter().map(map_generated_html_page).collect(),
        assets: result.assets.into_iter().map(map_shared_asset).collect(),
    }
}
