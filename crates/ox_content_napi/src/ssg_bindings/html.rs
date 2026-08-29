use napi_derive::napi;

use crate::{JsHeadDiagnostic, JsSsgConfig, JsSsgHtmlResult, JsSsgNavGroup, JsSsgPageData};

use super::converters::{
    convert_a11y, convert_entry_page_config, convert_json_ld, convert_nav_item,
    convert_pager_override, convert_theme_config, flatten_toc_entries,
};
use super::head::convert_head_validation;
use super::reader_chrome::convert_reader_chrome;

/// Generates SSG HTML page with navigation and search.
#[napi]
pub fn generate_ssg_html(
    page_data: JsSsgPageData,
    nav_groups: Vec<JsSsgNavGroup>,
    config: JsSsgConfig,
) -> JsSsgHtmlResult {
    let team = convert_team(config.team.clone());
    let ssg_page_data = convert_ssg_page_data(page_data, &team);
    let ssg_nav_groups = convert_nav_groups(nav_groups);
    let ssg_config = convert_ssg_config(config);
    map_html_result(ox_content_ssg::generate_html_result(
        &ssg_page_data,
        &ssg_nav_groups,
        &ssg_config,
    ))
}

/// Generates multiple themed SSG HTML pages while sharing navigation/config conversion.
#[napi(js_name = "generateSsgHtmlPages")]
pub fn generate_ssg_html_pages(
    page_datas: Vec<JsSsgPageData>,
    nav_groups: Vec<JsSsgNavGroup>,
    config: JsSsgConfig,
) -> Vec<JsSsgHtmlResult> {
    let team = convert_team(config.team.clone());
    let ssg_nav_groups = convert_nav_groups(nav_groups);
    let ssg_config = convert_ssg_config(config);

    page_datas
        .into_iter()
        .map(|page_data| {
            let ssg_page_data = convert_ssg_page_data(page_data, &team);
            map_html_result(ox_content_ssg::generate_html_result(
                &ssg_page_data,
                &ssg_nav_groups,
                &ssg_config,
            ))
        })
        .collect()
}

fn convert_ssg_page_data(
    page_data: JsSsgPageData,
    team: &ox_content_ssg::TeamOptions,
) -> ox_content_ssg::PageData {
    let layout = page_data.layout.unwrap_or_default();
    let content = ox_content_ssg::render_team_page(team, &layout, &page_data.content);
    ox_content_ssg::PageData {
        title: page_data.title,
        description: page_data.description,
        content,
        toc: flatten_toc_entries(page_data.toc),
        last_updated: page_data
            .last_updated
            .filter(|timestamp| timestamp.is_finite() && *timestamp >= 0.0)
            .map(|timestamp| timestamp as i64),
        contributors: page_data
            .contributors
            .unwrap_or_default()
            .into_iter()
            .map(|contributor| ox_content_ssg::Contributor {
                name: contributor.name,
                avatar: contributor.avatar,
            })
            .collect(),
        path: page_data.path,
        entry_page: convert_entry_page_config(page_data.entry_page),
        prev: convert_pager_override(page_data.prev),
        next: convert_pager_override(page_data.next),
        breadcrumbs: page_data.breadcrumbs,
        chrome: convert_page_chrome_flags(page_data.chrome),
        robots: page_data.robots,
        canonical: page_data.canonical,
        markdown_source: page_data.markdown_source,
    }
}

fn convert_nav_groups(nav_groups: Vec<JsSsgNavGroup>) -> Vec<ox_content_ssg::NavGroup> {
    nav_groups
        .into_iter()
        .map(|g| ox_content_ssg::NavGroup {
            title: g.title,
            items: g.items.into_iter().map(convert_nav_item).collect(),
            collapsed: g.collapsed,
            sticky_collapsed: g.sticky_collapsed,
        })
        .collect()
}

fn convert_ssg_config(config: JsSsgConfig) -> ox_content_ssg::SsgConfig {
    ox_content_ssg::SsgConfig {
        site_name: config.site_name,
        base: config.base,
        breadcrumb_root_href: config.breadcrumb_root_href,
        og_image: config.og_image,
        site_url: config.site_url.and_then(|url| {
            let trimmed = url.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        }),
        head_validation: convert_head_validation(config.head_validation),
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
        json_ld: convert_json_ld(config.json_ld),
    }
}

fn map_html_result(generated: ox_content_ssg::GeneratedHtml) -> JsSsgHtmlResult {
    JsSsgHtmlResult {
        html: generated.html,
        diagnostics: generated
            .diagnostics
            .into_iter()
            .map(|d| JsHeadDiagnostic { strict: d.strict, message: d.message })
            .collect(),
    }
}

fn convert_team(team: Option<crate::JsTeamOptions>) -> ox_content_ssg::TeamOptions {
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
