use crate::{
    JsEntryPageConfig, JsSsgGeneratedHtmlPage, JsSsgNavGroup, JsSsgNavItem, JsSsgNavigationGroup,
    JsSsgNavigationItem, JsSsgRoutePaths, JsSsgSharedAsset, JsSsgSidebarItem, JsThemeColors,
    JsThemeConfig, TocEntry,
};

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
pub(super) fn convert_theme_config(
    theme: Option<JsThemeConfig>,
) -> Option<ox_content_ssg::ThemeConfig> {
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
pub(super) fn convert_entry_page_config(
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

pub(super) fn convert_nav_item(item: JsSsgNavItem) -> ox_content_ssg::NavItem {
    ox_content_ssg::NavItem {
        title: item.title,
        path: item.path,
        href: item.href,
        children: item.children.unwrap_or_default().into_iter().map(convert_nav_item).collect(),
        collapsed: item.collapsed,
        sticky_collapsed: item.sticky_collapsed,
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
        sticky_collapsed: item.sticky_collapsed,
    }
}

pub(super) fn map_nav_group(group: ox_content_ssg::NavGroup) -> JsSsgNavGroup {
    JsSsgNavGroup {
        title: group.title,
        items: group.items.into_iter().map(map_nav_item).collect(),
        collapsed: group.collapsed,
        sticky_collapsed: group.sticky_collapsed,
    }
}

pub(super) fn convert_sidebar_item(item: JsSsgSidebarItem) -> ox_content_ssg::SidebarItem {
    ox_content_ssg::SidebarItem {
        text: item.text,
        link: item.link,
        items: item.items.unwrap_or_default().into_iter().map(convert_sidebar_item).collect(),
        collapsed: item.collapsed,
        sticky_collapsed: item.sticky_collapsed,
    }
}

fn convert_navigation_item(item: JsSsgNavigationItem) -> ox_content_ssg::ManualNavigationItem {
    ox_content_ssg::ManualNavigationItem { title: item.title, path: item.path, href: item.href }
}

pub(super) fn convert_navigation_group(
    group: JsSsgNavigationGroup,
) -> ox_content_ssg::ManualNavigationGroup {
    ox_content_ssg::ManualNavigationGroup {
        title: group.title,
        items: group.items.into_iter().map(convert_navigation_item).collect(),
    }
}

pub(super) fn map_route_paths(paths: ox_content_ssg::RoutePaths) -> JsSsgRoutePaths {
    JsSsgRoutePaths {
        output_path: paths.output_path,
        url_path: paths.url_path,
        href: paths.href,
        og_image_path: paths.og_image_path,
        og_image_url: paths.og_image_url,
    }
}

pub(super) fn convert_generated_html_page(
    page: JsSsgGeneratedHtmlPage,
) -> ox_content_ssg::GeneratedHtmlPage {
    ox_content_ssg::GeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

pub(super) fn map_generated_html_page(
    page: ox_content_ssg::GeneratedHtmlPage,
) -> JsSsgGeneratedHtmlPage {
    JsSsgGeneratedHtmlPage {
        input_path: page.input_path,
        output_path: page.output_path,
        html: page.html,
    }
}

pub(super) fn map_shared_asset(asset: ox_content_ssg::SharedAsset) -> JsSsgSharedAsset {
    JsSsgSharedAsset {
        output_path: asset.output_path,
        public_path: asset.public_path,
        content: asset.content,
    }
}

pub(super) fn flatten_toc_entries(entries: Vec<TocEntry>) -> Vec<ox_content_ssg::TocEntry> {
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
