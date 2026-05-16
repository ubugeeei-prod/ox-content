//! HTML page generation for SSG.

use askama::Template;
use serde::{Deserialize, Serialize};

// =============================================================================
// Theme Configuration Types
// =============================================================================

/// Theme color configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeColors {
    /// Primary accent color.
    pub primary: Option<String>,
    /// Primary color on hover.
    pub primary_hover: Option<String>,
    /// Background color.
    pub background: Option<String>,
    /// Alternative background color.
    pub background_alt: Option<String>,
    /// Main text color.
    pub text: Option<String>,
    /// Muted text color.
    pub text_muted: Option<String>,
    /// Border color.
    pub border: Option<String>,
    /// Code block background color.
    pub code_background: Option<String>,
    /// Code block text color.
    pub code_text: Option<String>,
}

/// Theme layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeLayout {
    /// Sidebar width (CSS value).
    pub sidebar_width: Option<String>,
    /// Header height (CSS value).
    pub header_height: Option<String>,
    /// Maximum content width (CSS value).
    pub max_content_width: Option<String>,
}

/// Theme font configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeFonts {
    /// Sans-serif font stack.
    pub sans: Option<String>,
    /// Monospace font stack.
    pub mono: Option<String>,
}

/// Theme configuration for entry pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeEntryPage {
    /// Landing page presentation mode.
    pub mode: Option<String>,
}

/// Theme header configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeHeader {
    /// Logo image URL.
    pub logo: Option<String>,
    /// Light mode logo image URL.
    #[serde(rename = "logoLight")]
    pub logo_light: Option<String>,
    /// Dark mode logo image URL.
    #[serde(rename = "logoDark")]
    pub logo_dark: Option<String>,
    /// Whether to render the site name text next to the logo.
    #[serde(rename = "showSiteNameText")]
    pub show_site_name_text: Option<bool>,
    /// Logo width in pixels.
    pub logo_width: Option<u32>,
    /// Logo height in pixels.
    pub logo_height: Option<u32>,
}

/// Theme footer configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeFooter {
    /// Footer message (supports HTML).
    pub message: Option<String>,
    /// Copyright text (supports HTML).
    pub copyright: Option<String>,
}

/// Social links configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialLinks {
    /// GitHub URL.
    pub github: Option<String>,
    /// Twitter/X URL.
    pub twitter: Option<String>,
    /// Discord URL.
    pub discord: Option<String>,
    /// Custom social links.
    pub links: Option<Vec<SocialLink>>,
}

/// Custom social link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    /// Icon label or built HTML icon source.
    pub icon: Option<String>,
    /// Trusted inline SVG icon.
    pub icon_svg: Option<String>,
    /// Link URL.
    pub link: String,
    /// Accessible label.
    pub aria_label: Option<String>,
}

/// Embedded HTML content for specific positions in the page layout.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeEmbed {
    /// Content to inject into `<head>`.
    pub head: Option<String>,
    /// Content before header.
    pub header_before: Option<String>,
    /// Content after header.
    pub header_after: Option<String>,
    /// Content before sidebar navigation.
    pub sidebar_before: Option<String>,
    /// Content after sidebar navigation.
    pub sidebar_after: Option<String>,
    /// Content before main content.
    pub content_before: Option<String>,
    /// Content after main content.
    pub content_after: Option<String>,
    /// Content before footer.
    pub footer_before: Option<String>,
    /// Custom footer content (replaces default footer).
    pub footer: Option<String>,
}

/// Complete theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeConfig {
    /// Light mode colors.
    pub colors: Option<ThemeColors>,
    /// Dark mode colors.
    pub dark_colors: Option<ThemeColors>,
    /// Font configuration.
    pub fonts: Option<ThemeFonts>,
    /// Entry page configuration.
    #[serde(rename = "entryPage")]
    pub entry_page: Option<ThemeEntryPage>,
    /// Layout configuration.
    pub layout: Option<ThemeLayout>,
    /// Header configuration.
    pub header: Option<ThemeHeader>,
    /// Footer configuration.
    pub footer: Option<ThemeFooter>,
    /// Social links configuration.
    pub social_links: Option<SocialLinks>,
    /// Embedded HTML content at specific positions.
    pub embed: Option<ThemeEmbed>,
    /// Additional custom CSS.
    pub css: Option<String>,
    /// Additional custom JavaScript.
    pub js: Option<String>,
}

// =============================================================================
// Entry Page Types
// =============================================================================

/// Hero action button.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroAction {
    /// Button theme: "brand" or "alt".
    pub theme: Option<String>,
    /// Button text.
    pub text: String,
    /// Link URL.
    pub link: String,
}

/// Hero image configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroImage {
    /// Image source URL.
    pub src: String,
    /// Light mode image source URL.
    #[serde(rename = "lightSrc")]
    pub light_src: Option<String>,
    /// Dark mode image source URL.
    #[serde(rename = "darkSrc")]
    pub dark_src: Option<String>,
    /// Alt text.
    pub alt: Option<String>,
    /// Image width.
    pub width: Option<u32>,
    /// Image height.
    pub height: Option<u32>,
}

/// Hero section configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroConfig {
    /// Main title (large, gradient text).
    pub name: Option<String>,
    /// Secondary text.
    pub text: Option<String>,
    /// Tagline.
    pub tagline: Option<String>,
    /// Optional notice shown in the hero.
    pub notice: Option<HeroNoticeConfig>,
    /// Hero image.
    pub image: Option<HeroImage>,
    /// Action buttons.
    pub actions: Option<Vec<HeroAction>>,
}

/// Hero notice configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeroNoticeConfig {
    /// Notice title.
    pub title: Option<String>,
    /// Notice paragraphs.
    pub body: Option<Vec<String>>,
}

/// Feature card configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureConfig {
    /// Icon - supports: "mdi:icon-name" (Iconify), image URL, or emoji.
    pub icon: Option<String>,
    /// Feature title.
    pub title: String,
    /// Feature description.
    pub details: Option<String>,
    /// Optional link.
    pub link: Option<String>,
    /// Link text.
    pub link_text: Option<String>,
}

/// Entry page configuration (for landing pages with hero and features).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntryPageConfig {
    /// Hero section.
    pub hero: Option<HeroConfig>,
    /// Feature cards.
    pub features: Option<Vec<FeatureConfig>>,
}

// =============================================================================
// Navigation and Page Types
// =============================================================================

/// Navigation item for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    /// Display title.
    pub title: String,
    /// URL path.
    pub path: String,
    /// Full href.
    pub href: String,
}

/// Navigation group for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavGroup {
    /// Group title.
    pub title: String,
    /// Navigation items.
    pub items: Vec<NavItem>,
}

/// Table of contents entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Heading depth (1-6).
    pub depth: u8,
    /// Heading text.
    pub text: String,
    /// URL-friendly slug.
    pub slug: String,
}

/// Page data for SSG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// Page title.
    pub title: String,
    /// Page description.
    pub description: Option<String>,
    /// Page content HTML.
    pub content: String,
    /// Table of contents entries.
    pub toc: Vec<TocEntry>,
    /// URL path.
    pub path: String,
    /// Entry page configuration (if layout: entry).
    pub entry_page: Option<EntryPageConfig>,
}

/// SSG configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgConfig {
    /// Site name.
    pub site_name: String,
    /// Base URL path.
    pub base: String,
    /// OG image URL.
    pub og_image: Option<String>,
    /// Theme configuration.
    pub theme: Option<ThemeConfig>,
    /// Current locale (BCP 47 tag) for this page, if i18n is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// All available locales (for generating locale switcher and hreflang tags).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_locales: Option<Vec<LocaleInfo>>,
}

/// Locale information for the locale switcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleInfo {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Text direction.
    pub dir: String,
}

// =============================================================================
// Askama Template Structures
// =============================================================================

/// Navigation template.
#[derive(Template)]
#[template(path = "nav.html")]
struct NavTemplate<'a> {
    nav_groups: &'a [NavGroup],
    current_path: &'a str,
}

/// Social links template (desktop header).
#[derive(Template)]
#[template(path = "social_links.html")]
struct SocialLinksTemplate<'a> {
    github: Option<&'a str>,
    twitter: Option<&'a str>,
    discord: Option<&'a str>,
}

/// Mobile social links template (mobile footer).
#[derive(Template)]
#[template(path = "mobile_social_links.html")]
struct MobileSocialLinksTemplate<'a> {
    github: Option<&'a str>,
    twitter: Option<&'a str>,
    discord: Option<&'a str>,
}

/// Footer template.
#[derive(Template)]
#[template(path = "footer.html")]
struct FooterTemplate<'a> {
    message: Option<&'a str>,
    copyright: Option<&'a str>,
}

/// Hero action for entry template.
pub struct HeroActionView {
    pub href: String,
    pub theme_class: String,
    pub text: String,
}

/// Feature card for entry template.
pub struct FeatureView {
    pub tag: &'static str,
    pub href_attr: String,
    pub icon_html: Option<String>,
    pub title: String,
    pub details: Option<String>,
    pub has_link: bool,
}

/// Hero view for entry template.
pub struct HeroView {
    pub name: Option<String>,
    pub text: Option<String>,
    pub tagline: Option<String>,
    pub notice: Option<HeroNoticeConfig>,
    pub image: Option<HeroImage>,
    pub actions: Option<Vec<HeroActionView>>,
}

/// Entry page template (hero + features).
#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate<'a> {
    hero: Option<&'a HeroView>,
    features: Option<&'a [FeatureView]>,
}

/// Main page template.
#[derive(Template)]
#[template(path = "page.html")]
struct PageTemplate<'a> {
    site_name: &'a str,
    document_title: &'a str,
    description: Option<&'a str>,
    og_image: Option<&'a str>,
    css: &'a str,
    embed_head: &'a str,
    body_class: &'a str,
    embed_header_before: &'a str,
    embed_header_after: &'a str,
    base: &'a str,
    logo_src: &'a str,
    logo_light_src: Option<&'a str>,
    logo_dark_src: Option<&'a str>,
    show_site_name_text: bool,
    logo_width: u32,
    logo_height: u32,
    social_links: &'a str,
    is_entry_page: bool,
    embed_sidebar_before: &'a str,
    navigation: &'a str,
    embed_sidebar_after: &'a str,
    embed_content_before: &'a str,
    main_content: &'a str,
    has_toc: bool,
    toc_html: &'a str,
    embed_content_after: &'a str,
    embed_footer_before: &'a str,
    footer_html: &'a str,
    mobile_social_links: &'a str,
    js: &'a str,
}

/// CSS styles for SSG pages.
const SSG_CSS: &str = include_str!("ssg.css");

/// CSS styles for Entry pages (hero, features).
const ENTRY_CSS: &str = include_str!("entry.css");

/// CSS styles for Tabs plugin.
const TABS_CSS: &str = include_str!("plugins/tabs.css");

/// CSS styles for YouTube plugin.
const YOUTUBE_CSS: &str = include_str!("plugins/youtube.css");

/// CSS styles for GitHub plugin.
const GITHUB_CSS: &str = include_str!("plugins/github.css");

/// CSS styles for OGP plugin.
const OGP_CSS: &str = include_str!("plugins/ogp.css");

/// CSS styles for Mermaid plugin.
const MERMAID_CSS: &str = include_str!("plugins/mermaid.css");

/// CSS styles for Island plugin.
const ISLAND_CSS: &str = include_str!("plugins/island.css");

/// JavaScript for SSG pages.
const SSG_JS: &str = include_str!("ssg.js");

/// Generates CSS variable overrides for theme colors.
fn generate_theme_css(theme: &ThemeConfig) -> String {
    let mut css = String::new();

    // Light mode colors
    if let Some(ref colors) = theme.colors {
        let mut vars = Vec::new();
        if let Some(ref v) = colors.primary {
            vars.push(format!("--octc-color-primary: {v};"));
        }
        if let Some(ref v) = colors.primary_hover {
            vars.push(format!("--octc-color-primary-hover: {v};"));
        }
        if let Some(ref v) = colors.background {
            vars.push(format!("--octc-color-bg: {v};"));
        }
        if let Some(ref v) = colors.background_alt {
            vars.push(format!("--octc-color-bg-alt: {v};"));
        }
        if let Some(ref v) = colors.text {
            vars.push(format!("--octc-color-text: {v};"));
        }
        if let Some(ref v) = colors.text_muted {
            vars.push(format!("--octc-color-text-muted: {v};"));
        }
        if let Some(ref v) = colors.border {
            vars.push(format!("--octc-color-border: {v};"));
        }
        if let Some(ref v) = colors.code_background {
            vars.push(format!("--octc-color-code-bg: {v};"));
        }
        if let Some(ref v) = colors.code_text {
            vars.push(format!("--octc-color-code-text: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Dark mode colors
    if let Some(ref colors) = theme.dark_colors {
        let mut vars = Vec::new();
        if let Some(ref v) = colors.primary {
            vars.push(format!("--octc-color-primary: {v};"));
        }
        if let Some(ref v) = colors.primary_hover {
            vars.push(format!("--octc-color-primary-hover: {v};"));
        }
        if let Some(ref v) = colors.background {
            vars.push(format!("--octc-color-bg: {v};"));
        }
        if let Some(ref v) = colors.background_alt {
            vars.push(format!("--octc-color-bg-alt: {v};"));
        }
        if let Some(ref v) = colors.text {
            vars.push(format!("--octc-color-text: {v};"));
        }
        if let Some(ref v) = colors.text_muted {
            vars.push(format!("--octc-color-text-muted: {v};"));
        }
        if let Some(ref v) = colors.border {
            vars.push(format!("--octc-color-border: {v};"));
        }
        if let Some(ref v) = colors.code_background {
            vars.push(format!("--octc-color-code-bg: {v};"));
        }
        if let Some(ref v) = colors.code_text {
            vars.push(format!("--octc-color-code-text: {v};"));
        }
        if !vars.is_empty() {
            css.push_str("[data-theme=\"dark\"] {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
            css.push_str("@media (prefers-color-scheme: dark) {\n  :root:not([data-theme=\"light\"]) {\n    ");
            css.push_str(&vars.join("\n    "));
            css.push_str("\n  }\n}\n");
        }
    }

    // Layout overrides
    if let Some(ref layout) = theme.layout {
        let mut vars = Vec::new();
        if let Some(ref v) = layout.sidebar_width {
            vars.push(format!("--octc-sidebar-width: {v};"));
        }
        if let Some(ref v) = layout.header_height {
            vars.push(format!("--octc-header-height: {v};"));
        }
        if let Some(ref v) = layout.max_content_width {
            vars.push(format!("--octc-max-content-width: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Font overrides
    if let Some(ref fonts) = theme.fonts {
        let mut vars = Vec::new();
        if let Some(ref v) = fonts.sans {
            vars.push(format!("--octc-font-sans: {v};"));
        }
        if let Some(ref v) = fonts.mono {
            vars.push(format!("--octc-font-mono: {v};"));
        }
        if !vars.is_empty() {
            css.push_str(":root {\n  ");
            css.push_str(&vars.join("\n  "));
            css.push_str("\n}\n");
        }
    }

    // Custom CSS
    if let Some(ref custom_css) = theme.css {
        css.push_str(custom_css);
    }

    css
}

/// Generates footer HTML from theme configuration.
fn generate_footer_html(theme: &ThemeConfig) -> String {
    let footer = match &theme.footer {
        Some(f) if f.message.is_some() || f.copyright.is_some() => f,
        _ => return String::new(),
    };

    let template = FooterTemplate {
        message: footer.message.as_deref(),
        copyright: footer.copyright.as_deref(),
    };
    template.render().unwrap_or_default()
}

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

    // Remove .md extension
    let stem = &path[..path.len() - 3];

    // Entry page is always index.md, so plain relative: getting-started.md -> {base}getting-started/index.html
    let converted = if stem == "index" || stem.ends_with("/index") {
        let dir = stem.trim_end_matches("/index").trim_end_matches("index");
        if dir.is_empty() {
            format!("{base}index.html")
        } else {
            format!("{base}{dir}/index.html")
        }
    } else {
        format!("{base}{stem}/index.html")
    };

    match fragment {
        Some(f) => format!("{converted}#{f}"),
        None => converted,
    }
}

/// Generates the Entry page HTML (hero section and features).
fn generate_entry_html(entry: &EntryPageConfig, base: &str) -> String {
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
            let src = convert_entry_link(&img.src, base);
            let light_src = img.light_src.as_ref().map(|src| convert_entry_link(src, base));
            let dark_src = img.dark_src.as_ref().map(|src| convert_entry_link(src, base));
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

                let icon_html = feature.icon.as_ref().map(|icon| render_icon(icon, base));

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

/// Footer CSS styles (added when footer is used).
const FOOTER_CSS: &str = r"
.site-footer {
  margin-top: 3rem;
  padding: 2rem 1.5rem;
  border-top: 1px solid var(--octc-color-border);
  text-align: center;
  color: var(--octc-color-text-muted);
  font-size: 0.875rem;
}
.site-footer p {
  margin: 0.25rem 0;
}
.site-footer a {
  color: var(--octc-color-primary);
}
.site-footer a:hover {
  color: var(--octc-color-primary-hover);
}
";

fn wrap_css_section(name: &str, css: &str) -> String {
    if css.trim().is_empty() {
        return String::new();
    }

    format!("/* ox-content:css:{name}:start */\n{css}\n/* ox-content:css:{name}:end */\n")
}

fn page_content_contains_any(content: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| content.contains(needle))
}

fn escape_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
    output
}

fn generate_toc_html(toc: &[TocEntry]) -> String {
    let mut html = String::new();

    for entry in toc {
        let depth = entry.depth.clamp(1, 6);
        html.push_str("        <li class=\"toc-item\"><a href=\"#");
        html.push_str(&escape_html(&entry.slug));
        html.push_str("\" class=\"toc-link toc-link--depth-");
        html.push_str(&depth.to_string());
        html.push_str("\">");
        html.push_str(&escape_html(&entry.text));
        html.push_str("</a></li>\n");
    }

    html
}

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
    if page_content_contains_any(&page_data.content, &["ox-github-card", "ox-github-error"]) {
        css_sections.push(wrap_css_section("plugin-github", GITHUB_CSS));
    }
    if page_content_contains_any(&page_data.content, &["ox-ogp-card", "ox-ogp-simple"]) {
        css_sections.push(wrap_css_section("plugin-ogp", OGP_CSS));
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
    let all_js = format!("{}\n{}", SSG_JS.replace("{{base}}", &config.base), custom_js);

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

    let template = PageTemplate {
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
        embed_content_after,
        embed_footer_before,
        footer_html: &footer_html,
        mobile_social_links: &mobile_social_links_html,
        js: &all_js,
    };

    template.render().unwrap_or_default()
}

/// Renders an icon based on its format.
///
/// Supported formats:
/// - `mdi:icon-name` - Material Design Icons via Iconify CDN
/// - `lucide:icon-name` - Lucide icons via Iconify CDN
/// - `{prefix}:{name}` - Any Iconify icon set
/// - URL (http://, https://) - Direct image URL
/// - Path ending with .svg, .png - Local image path
/// - Other - Treated as emoji/text
fn render_icon(icon: &str, base: &str) -> String {
    // Check for Iconify format (prefix:name)
    if let Some((prefix, name)) = icon.split_once(':') {
        // Validate it looks like an icon reference (not a URL scheme)
        if !prefix.contains('/') && !name.starts_with("//") {
            // Convert to Iconify CDN URL
            let iconify_url = format!("https://api.iconify.design/{prefix}/{name}.svg");
            // Use span with mask-image for color control
            return format!(
                "<span class=\"iconify-icon\" style=\"-webkit-mask-image: url('{iconify_url}'); mask-image: url('{iconify_url}')\"></span>"
            );
        }
    }

    // Check if it's an image URL
    if icon.starts_with("http://") || icon.starts_with("https://") {
        return format!("<img src=\"{icon}\" alt=\"\" />");
    }

    // Check if it's a local image path
    if icon.ends_with(".svg") || icon.ends_with(".png") {
        let icon_src =
            if icon.starts_with('/') { icon.to_string() } else { format!("{base}{icon}") };
        return format!("<img src=\"{icon_src}\" alt=\"\" />");
    }

    // Treat as emoji/text
    icon.to_string()
}

fn generate_social_links_html(links: &SocialLinks) -> String {
    let template = SocialLinksTemplate {
        github: links.github.as_deref(),
        twitter: links.twitter.as_deref(),
        discord: links.discord.as_deref(),
    };
    let mut html = template.render().unwrap_or_default();
    if let Some(custom_links) = &links.links {
        for link in custom_links {
            html.push_str(&render_custom_social_link(link, false));
        }
    }
    html
}

fn generate_mobile_social_links_html(links: &SocialLinks) -> String {
    let template = MobileSocialLinksTemplate {
        github: links.github.as_deref(),
        twitter: links.twitter.as_deref(),
        discord: links.discord.as_deref(),
    };
    let mut html = template.render().unwrap_or_default();
    if let Some(custom_links) = &links.links {
        for link in custom_links {
            html.push_str(&render_custom_social_link(link, true));
        }
    }
    html
}

fn render_custom_social_link(link: &SocialLink, mobile: bool) -> String {
    let label = link.aria_label.as_deref().or(link.icon.as_deref()).unwrap_or("Social link");
    let href = escape_html(&link.link);
    let label = escape_html(label);
    let icon_html = render_social_icon(link);

    if mobile {
        format!(
            "<a href=\"{href}\" class=\"mobile-footer-btn\" aria-label=\"{label}\" target=\"_blank\" rel=\"noopener\">{icon_html}<span class=\"mobile-footer-label\">{label}</span></a>\n"
        )
    } else {
        format!(
            "<a href=\"{href}\" class=\"social-link\" aria-label=\"{label}\" target=\"_blank\" rel=\"noopener\">{icon_html}</a>\n"
        )
    }
}

fn render_social_icon(link: &SocialLink) -> String {
    if let Some(svg) = link.icon_svg.as_deref().and_then(validate_social_svg) {
        return svg.to_string();
    }

    link.icon
        .as_deref()
        .map(|icon| format!("<span class=\"social-link-icon\">{}</span>", escape_html(icon)))
        .unwrap_or_default()
}

fn validate_social_svg(svg: &str) -> Option<&str> {
    let trimmed = svg.trim();
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.starts_with("<svg") && !lower.contains("<script") {
        Some(trimmed)
    } else {
        None
    }
}

fn generate_nav_html(nav_groups: &[NavGroup], current_path: &str) -> String {
    let template = NavTemplate { nav_groups, current_path };
    template.render().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_html() {
        let page_data = PageData {
            title: "Test Page".to_string(),
            description: Some("Test description".to_string()),
            content: "<h1>Hello</h1>".to_string(),
            toc: vec![TocEntry { depth: 1, text: "Hello".to_string(), slug: "hello".to_string() }],
            path: "test".to_string(),
            entry_page: None,
        };

        let nav_groups = vec![NavGroup {
            title: "Guide".to_string(),
            items: vec![NavItem {
                title: "Test Page".to_string(),
                path: "test".to_string(),
                href: "/docs/test/index.html".to_string(),
            }],
        }];

        let config = SsgConfig {
            site_name: "Test Site".to_string(),
            base: "/docs/".to_string(),
            og_image: None,
            theme: None,
            locale: None,
            available_locales: None,
        };

        let html = generate_html(&page_data, &nav_groups, &config);

        assert!(html.contains("Test Page - Test Site"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("Guide"));
        assert!(html.contains("class=\"toc\""));
        assert!(html.contains("href=\"#hello\""));
    }

    #[test]
    fn test_generate_html_with_theme() {
        let page_data = PageData {
            title: "Themed Page".to_string(),
            description: None,
            content: "<p>Content</p>".to_string(),
            toc: vec![],
            path: "themed".to_string(),
            entry_page: None,
        };

        let nav_groups = vec![];

        let config = SsgConfig {
            site_name: "Themed Site".to_string(),
            base: "/".to_string(),
            og_image: None,
            locale: None,
            available_locales: None,
            theme: Some(ThemeConfig {
                colors: Some(ThemeColors {
                    primary: Some("#3498db".to_string()),
                    ..Default::default()
                }),
                footer: Some(ThemeFooter {
                    message: Some("Built with ox-content".to_string()),
                    copyright: Some("2025 Test".to_string()),
                }),
                ..Default::default()
            }),
        };

        let html = generate_html(&page_data, &nav_groups, &config);

        // Check theme CSS is applied
        assert!(html.contains("--octc-color-primary: #3498db;"));
        // Check footer is present
        assert!(html.contains("Built with ox-content"));
        assert!(html.contains("2025 Test"));
    }

    #[test]
    fn test_generate_html_with_custom_social_link() {
        let page_data = PageData {
            title: "Social Page".to_string(),
            description: None,
            content: "<p>Content</p>".to_string(),
            toc: vec![],
            path: "social".to_string(),
            entry_page: None,
        };
        let config = SsgConfig {
            site_name: "Social Site".to_string(),
            base: "/".to_string(),
            og_image: None,
            locale: None,
            available_locales: None,
            theme: Some(ThemeConfig {
                social_links: Some(SocialLinks {
                    links: Some(vec![SocialLink {
                        icon: None,
                        icon_svg: Some("<svg viewBox=\"0 0 24 24\"></svg>".to_string()),
                        link: "https://example.com".to_string(),
                        aria_label: Some("Example".to_string()),
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };

        let html = generate_html(&page_data, &[], &config);

        assert!(html.contains("aria-label=\"Example\""));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("<svg viewBox=\"0 0 24 24\"></svg>"));
        assert!(html.contains("<span class=\"mobile-footer-label\">Example</span>"));
    }

    #[test]
    fn test_generate_html_without_toc_omits_outline() {
        let page_data = PageData {
            title: "No TOC".to_string(),
            description: None,
            content: "<p>Content</p>".to_string(),
            toc: vec![],
            path: "no-toc".to_string(),
            entry_page: None,
        };
        let config = SsgConfig {
            site_name: "Test Site".to_string(),
            base: "/".to_string(),
            og_image: None,
            theme: None,
            locale: None,
            available_locales: None,
        };

        let html = generate_html(&page_data, &[], &config);

        assert!(!html.contains("class=\"toc\""));
        assert!(!html.contains("<main class=\"main main--with-toc\">"));
    }

    #[test]
    fn test_generate_toc_html_escapes_entries() {
        let html = generate_toc_html(&[TocEntry {
            depth: 2,
            text: "A <script>".to_string(),
            slug: "a\" onclick=\"alert(1)".to_string(),
        }]);

        assert!(html.contains("A &lt;script&gt;"));
        assert!(html.contains("href=\"#a&quot; onclick=&quot;alert(1)\""));
        assert!(!html.contains("A <script>"));
    }

    #[test]
    fn test_generate_theme_css() {
        let theme = ThemeConfig {
            colors: Some(ThemeColors {
                primary: Some("#ff0000".to_string()),
                background: Some("#ffffff".to_string()),
                ..Default::default()
            }),
            dark_colors: Some(ThemeColors {
                primary: Some("#ff6666".to_string()),
                ..Default::default()
            }),
            layout: Some(ThemeLayout {
                sidebar_width: Some("300px".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let css = generate_theme_css(&theme);

        assert!(css.contains("--octc-color-primary: #ff0000;"));
        assert!(css.contains("--octc-color-bg: #ffffff;"));
        assert!(css.contains("[data-theme=\"dark\"]"));
        assert!(css.contains("--octc-sidebar-width: 300px;"));
    }

    #[test]
    fn test_generate_footer_html() {
        let theme = ThemeConfig {
            footer: Some(ThemeFooter {
                message: Some("Footer message".to_string()),
                copyright: Some("Copyright info".to_string()),
            }),
            ..Default::default()
        };

        let html = generate_footer_html(&theme);

        assert!(html.contains("site-footer"));
        assert!(html.contains("Footer message"));
        assert!(html.contains("Copyright info"));
    }
}
