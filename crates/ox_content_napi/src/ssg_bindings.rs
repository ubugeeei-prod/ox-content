use std::path::PathBuf;
use std::process::Command;

use napi_derive::napi;

use crate::{
    JsA11y, JsReaderChrome, JsSsgBarePage, JsSsgConfig, JsSsgExternalizedAssets,
    JsSsgGeneratedHtmlPage, JsSsgNavGroup, JsSsgNavigationGroup, JsSsgPageData, JsSsgRoutePaths,
    JsSsgSidebarItem, JsTeamOptions,
};

mod converters;

use converters::{
    convert_entry_page_config, convert_generated_html_page, convert_nav_item,
    convert_navigation_group, convert_pager_override, convert_sidebar_item, convert_theme_config,
    flatten_toc_entries, map_generated_html_page, map_nav_group, map_route_paths, map_shared_asset,
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
    let layout = page_data.layout.unwrap_or_default();
    let content =
        ox_content_ssg::render_team_page(&convert_team(config.team), &layout, &page_data.content);
    let ssg_page_data = ox_content_ssg::PageData {
        title: page_data.title,
        description: page_data.description,
        content,
        toc: flatten_toc_entries(page_data.toc),
        last_updated: page_data
            .last_updated
            .filter(|timestamp| timestamp.is_finite() && *timestamp >= 0.0)
            .map(|timestamp| timestamp as i64),
        path: page_data.path,
        entry_page: convert_entry_page_config(page_data.entry_page),
        prev: convert_pager_override(page_data.prev),
        next: convert_pager_override(page_data.next),
        breadcrumbs: page_data.breadcrumbs,
        chrome: convert_page_chrome_flags(page_data.chrome),
    };

    let ssg_nav_groups: Vec<ox_content_ssg::NavGroup> = nav_groups
        .into_iter()
        .map(|g| ox_content_ssg::NavGroup {
            title: g.title,
            items: g.items.into_iter().map(convert_nav_item).collect(),
            collapsed: g.collapsed,
            sticky_collapsed: g.sticky_collapsed,
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
        pagination: config.pagination.unwrap_or(false),
        breadcrumbs: config.breadcrumbs.unwrap_or(false),
        reader_chrome: convert_reader_chrome(config.reader_chrome),
        locale_switcher: config.locale_switcher.unwrap_or(false),
        locale_paths: config
            .locale_paths
            .unwrap_or_default()
            .into_iter()
            .map(|path| ox_content_ssg::LocalePath {
                code: path.code,
                href: path.href,
                root: path.root,
            })
            .collect(),
        a11y: convert_a11y(config.a11y),
        page_chrome: config.page_chrome.unwrap_or(false),
    };

    ox_content_ssg::generate_html(&ssg_page_data, &ssg_nav_groups, &ssg_config)
}

fn convert_a11y(a11y: Option<JsA11y>) -> ox_content_ssg::A11y {
    match a11y {
        None => ox_content_ssg::A11y::disabled(),
        Some(a11y) => {
            ox_content_ssg::A11y { skip_link_label: Some(a11y.skip_link_label.unwrap_or_default()) }
        }
    }
}

fn convert_team(team: Option<JsTeamOptions>) -> ox_content_ssg::TeamOptions {
    match team {
        None => ox_content_ssg::TeamOptions::disabled(),
        Some(team) => ox_content_ssg::TeamOptions {
            enabled: team.enabled.unwrap_or(true),
            members: team
                .members
                .unwrap_or_default()
                .into_iter()
                .map(|member| ox_content_ssg::TeamMember {
                    name: member.name,
                    role: member.role,
                    avatar: member.avatar,
                    links: member.links.map(|links| {
                        links
                            .into_iter()
                            .map(|link| ox_content_ssg::TeamLink {
                                label: link.label,
                                href: link.href,
                            })
                            .collect()
                    }),
                })
                .collect(),
        },
    }
}

fn convert_page_chrome_flags(
    flags: Option<crate::JsPageChromeFlags>,
) -> ox_content_ssg::PageChromeFlags {
    flags.map_or_else(ox_content_ssg::PageChromeFlags::default, |flags| {
        ox_content_ssg::PageChromeFlags {
            sidebar: flags.sidebar,
            outline: flags.outline,
            aside: flags.aside,
            footer: flags.footer,
            navbar: flags.navbar,
            last_updated: flags.last_updated,
            edit_link: flags.edit_link,
        }
    })
}

fn convert_reader_chrome(chrome: Option<JsReaderChrome>) -> ox_content_ssg::ReaderChrome {
    match chrome {
        None => ox_content_ssg::ReaderChrome::disabled(),
        Some(chrome) => ox_content_ssg::ReaderChrome {
            copy: chrome.copy.unwrap_or(true),
            external_links: chrome.external_links.unwrap_or(true),
            back_to_top: chrome.back_to_top.unwrap_or(true),
        },
    }
}

/// Generates a bare SSG HTML page without navigation or styles.
#[napi(js_name = "generateSsgBareHtml")]
pub fn generate_ssg_bare_html(content: String, title: String) -> String {
    ox_content_ssg::generate_bare_html(&content, &title)
}

/// Generates a bare SSG HTML page carrying head metadata and injected markup.
#[napi(js_name = "generateSsgBarePage")]
pub fn generate_ssg_bare_page(page: JsSsgBarePage) -> String {
    ox_content_ssg::generate_bare_page(&ox_content_ssg::BarePageData {
        title: &page.title,
        content: &page.content,
        lang: page.lang.as_deref().unwrap_or_default(),
        dir: page.dir.as_deref().unwrap_or_default(),
        description: page.description.as_deref(),
        canonical_url: page.canonical_url.as_deref(),
        site_name: page.site_name.as_deref(),
        og_image: page.og_image.as_deref(),
        head: page.head.as_deref().unwrap_or_default(),
        body_start: page.body_start.as_deref().unwrap_or_default(),
        body_end: page.body_end.as_deref().unwrap_or_default(),
    })
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
