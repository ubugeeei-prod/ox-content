use askama::Template;

use super::urls::with_base;
use super::{EntryPageConfig, EntryTemplate, FeatureView, HeroActionView, HeroImage, HeroView};

/// Converts a `.md` link to an HTML path for entry page frontmatter links.
/// Entry pages are always `index.md`, so relative links like `getting-started.md`
/// become `{base}getting-started/index.html`.
fn convert_entry_link(link: &str, base: &str) -> String {
    // Don't touch external URLs or anchor-only links
    if link.starts_with("http://") || link.starts_with("https://") || link.starts_with('#') {
        return link.to_string();
    }

    // Split into path and fragment
    let (path, fragment) = match link.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (link, None),
    };

    let is_md =
        std::path::Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md"));

    if !is_md {
        return link.to_string();
    }

    // Remove a trailing `.md` without slicing through a multibyte character.
    let stem = strip_ascii_suffix_ignore_case(path, b".md").unwrap_or(path).trim_start_matches('/');

    // Entry page is always index.md, so plain relative: getting-started.md -> {base}getting-started/index.html
    let converted = if stem == "index" || stem.ends_with("/index") {
        let dir = stem.trim_end_matches("/index").trim_end_matches("index");
        if dir.is_empty() { format!("{base}index.html") } else { format!("{base}{dir}/index.html") }
    } else {
        format!("{base}{stem}/index.html")
    };

    match fragment {
        Some(f) => format!("{converted}#{f}"),
        None => converted,
    }
}

fn convert_entry_asset(src: &str, base: &str) -> String {
    with_base(base, src)
}

/// Generates the Entry page HTML (hero section and features).
pub(super) fn generate_entry_html(
    entry: &EntryPageConfig,
    base: &str,
    self_hosted_icons: bool,
) -> String {
    // Convert hero config to view
    let hero_view = entry.hero.as_ref().map(|hero| {
        let actions = hero.actions.as_ref().map(|actions| {
            actions
                .iter()
                .map(|action| {
                    let theme_class = match action.theme.as_deref() {
                        Some("brand") | None => "hero-action-brand",
                        Some("alt") => "hero-action-alt",
                        _ => "hero-action-brand",
                    };
                    let href = convert_entry_link(&action.link, base);
                    HeroActionView {
                        href,
                        theme_class: theme_class.to_string(),
                        text: action.text.clone(),
                    }
                })
                .collect()
        });

        // Process hero image src
        let image = hero.image.as_ref().map(|img| {
            let src = convert_entry_asset(&img.src, base);
            let light_src = img.light_src.as_ref().map(|src| convert_entry_asset(src, base));
            let dark_src = img.dark_src.as_ref().map(|src| convert_entry_asset(src, base));
            HeroImage {
                src,
                light_src,
                dark_src,
                alt: img.alt.clone(),
                width: img.width,
                height: img.height,
            }
        });

        HeroView {
            name: hero.name.clone(),
            text: hero.text.clone(),
            tagline: hero.tagline.clone(),
            notice: hero.notice.clone(),
            image,
            actions,
        }
    });

    // Convert features config to view
    let features_view: Option<Vec<FeatureView>> = entry.features.as_ref().map(|features| {
        features
            .iter()
            .map(|feature| {
                let has_link = feature.link.is_some();
                let tag = if has_link { "a" } else { "div" };
                let href_attr = feature
                    .link
                    .as_ref()
                    .map(|link| {
                        let href = convert_entry_link(link, base);
                        format!(" href=\"{href}\"")
                    })
                    .unwrap_or_default();

                let icon_html =
                    feature.icon.as_ref().map(|icon| render_icon(icon, base, self_hosted_icons));

                FeatureView {
                    tag,
                    href_attr,
                    icon_html,
                    title: feature.title.clone(),
                    details: feature.details.clone(),
                    has_link,
                }
            })
            .collect()
    });

    let template = EntryTemplate { hero: hero_view.as_ref(), features: features_view.as_deref() };
    template.render().unwrap_or_default()
}

/// Parses `prefix:name` Iconify refs. Rejects URL schemes.
pub(super) fn parse_iconify_name(icon: &str) -> Option<(&str, &str)> {
    let (prefix, name) = icon.split_once(':')?;
    if prefix.contains('/')
        || name.starts_with("//")
        || prefix.eq_ignore_ascii_case("http")
        || prefix.eq_ignore_ascii_case("https")
        || prefix.eq_ignore_ascii_case("data")
    {
        return None;
    }
    Some((prefix, name))
}

/// Renders an icon based on its format.
///
/// Supported formats:
/// - `mdi:icon-name` - Material Design Icons via Iconify
/// - `lucide:icon-name` - Lucide icons via Iconify
/// - `{prefix}:{name}` - Any Iconify icon set
/// - URL (http://, https://) - Direct image URL
/// - Path ending with .svg, .png - Local image path
/// - Other - Treated as emoji/text
///
/// When `self_hosted` is true, Iconify names become local CSS classes
/// (`icon-[prefix--name]`) instead of `api.iconify.design` mask URLs.
fn render_icon(icon: &str, base: &str, self_hosted: bool) -> String {
    if let Some((prefix, name)) = parse_iconify_name(icon) {
        if self_hosted {
            return format!("<span class=\"iconify-icon icon-[{prefix}--{name}]\"></span>");
        }
        let iconify_url = format!("https://api.iconify.design/{prefix}/{name}.svg");
        return format!(
            "<span class=\"iconify-icon\" style=\"-webkit-mask-image: url('{iconify_url}'); mask-image: url('{iconify_url}')\"></span>"
        );
    }

    // Check if it's an image URL
    if icon.starts_with("http://") || icon.starts_with("https://") {
        return format!("<img src=\"{icon}\" alt=\"\" />");
    }

    // Check if it's a local image path
    if icon.ends_with(".svg") || icon.ends_with(".png") {
        return format!("<img src=\"{}\" alt=\"\" />", with_base(base, icon));
    }

    // Treat as emoji/text
    icon.to_string()
}

fn strip_ascii_suffix_ignore_case<'a>(value: &'a str, suffix: &[u8]) -> Option<&'a str> {
    let bytes = value.as_bytes();
    let start = bytes.len().checked_sub(suffix.len())?;
    bytes[start..].eq_ignore_ascii_case(suffix).then(|| &value[..start])
}

#[cfg(test)]
mod tests {
    use super::{convert_entry_asset, convert_entry_link, generate_entry_html, parse_iconify_name};
    use crate::{EntryPageConfig, FeatureConfig};

    #[test]
    fn entry_links_join_root_markdown_paths_without_double_slashes() {
        assert_eq!(
            convert_entry_link("/ja/getting-started.md", "/ox-content/"),
            "/ox-content/ja/getting-started/index.html"
        );
        assert_eq!(
            convert_entry_link("/ja/\u{3042}.md", "/ox-content/"),
            "/ox-content/ja/\u{3042}/index.html"
        );
        assert_eq!(convert_entry_link("\u{1F600}", "/ox-content/"), "\u{1F600}");
    }

    #[test]
    fn entry_assets_are_rooted_at_the_deployment_base() {
        assert_eq!(
            convert_entry_asset("oxcontent-dark.svg", "/ox-content/"),
            "/ox-content/oxcontent-dark.svg"
        );
        // A root-absolute path is written against the site root, the way the
        // renderer already reads one in Markdown. Left as authored it 404s
        // on a site deployed under a sub-path.
        assert_eq!(
            convert_entry_asset("/shared/logo.svg", "/ox-content/"),
            "/ox-content/shared/logo.svg"
        );
        assert_eq!(convert_entry_asset("/shared/logo.svg", "/"), "/shared/logo.svg");
    }

    #[test]
    fn entry_assets_outside_the_site_are_left_alone() {
        for src in [
            "https://cdn.example.com/logo.svg",
            "//cdn.example.com/logo.svg",
            "data:image/svg+xml;base64,PHN2Zy8+",
        ] {
            assert_eq!(convert_entry_asset(src, "/ox-content/"), src);
        }
    }

    #[test]
    fn parse_iconify_name_rejects_url_schemes() {
        assert_eq!(parse_iconify_name("ox:mark"), Some(("ox", "mark")));
        assert_eq!(parse_iconify_name("https://example.com/x.svg"), None);
    }

    fn feature_entry(icon: &str) -> EntryPageConfig {
        EntryPageConfig {
            features: Some(vec![FeatureConfig {
                icon: Some(icon.to_string()),
                title: "Mark".to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        }
    }

    #[test]
    fn iconify_uses_cdn_when_self_host_is_off() {
        let html = generate_entry_html(&feature_entry("ox:mark"), "/", false);
        assert!(html.contains("https://api.iconify.design/ox/mark.svg"));
    }

    #[test]
    fn iconify_uses_local_class_when_self_host_is_on() {
        let html = generate_entry_html(&feature_entry("ox:mark"), "/", true);
        assert!(html.contains("icon-[ox--mark]"));
        assert!(!html.contains("api.iconify.design"));
    }
}
