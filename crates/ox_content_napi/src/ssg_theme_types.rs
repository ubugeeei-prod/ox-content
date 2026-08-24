use napi_derive::napi;

/// Theme colors for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeColors {
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
    /// Code block gradient color at the top.
    pub code_background_top: Option<String>,
    /// Code block text color.
    pub code_text: Option<String>,
}

/// Theme fonts for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeFonts {
    /// Sans-serif font stack.
    pub sans: Option<String>,
    /// Monospace font stack.
    pub mono: Option<String>,
}

/// Entry page theme configuration for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeEntryPage {
    /// Landing page presentation mode.
    pub mode: Option<String>,
}

/// Theme layout for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeLayout {
    /// Sidebar width (CSS value).
    pub sidebar_width: Option<String>,
    /// Header height (CSS value).
    pub header_height: Option<String>,
    /// Maximum content width (CSS value).
    pub max_content_width: Option<String>,
}

/// Theme header for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeHeader {
    /// Logo image URL.
    pub logo: Option<String>,
    /// Light mode logo image URL.
    pub logo_light: Option<String>,
    /// Dark mode logo image URL.
    pub logo_dark: Option<String>,
    /// Whether to render the site name text next to the logo.
    pub show_site_name_text: Option<bool>,
    /// Logo width in pixels.
    pub logo_width: Option<u32>,
    /// Logo height in pixels.
    pub logo_height: Option<u32>,
}

/// Theme footer for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeFooter {
    /// Footer message (supports HTML).
    pub message: Option<String>,
    /// Copyright text (supports HTML).
    pub copyright: Option<String>,
}

/// Social links for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsSocialLinks {
    /// GitHub URL.
    pub github: Option<String>,
    /// Twitter/X URL.
    pub twitter: Option<String>,
    /// Discord URL.
    pub discord: Option<String>,
    /// Custom social links.
    pub links: Option<Vec<JsSocialLink>>,
}

/// Custom social link for JavaScript.
#[napi(object)]
#[derive(Clone)]
pub struct JsSocialLink {
    /// Icon label.
    pub icon: Option<String>,
    /// Inline SVG icon.
    pub icon_svg: Option<String>,
    /// Link URL.
    pub link: String,
    /// Accessible label.
    pub aria_label: Option<String>,
}

/// Embedded HTML content for specific positions.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeEmbed {
    /// Content to embed into `<head>`.
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
    /// Custom footer content.
    pub footer: Option<String>,
}

/// Theme configuration for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeConfig {
    /// Progressive cross-document transitions for same-origin MPA navigation.
    pub view_transitions: Option<bool>,
    /// Right-hand "On this page" outline. Omitted and `false` hide it.
    pub aside: Option<bool>,
    /// Breadcrumb trail from the site root through sidebar ancestors.
    pub breadcrumbs: Option<bool>,
    /// Light mode colors.
    pub colors: Option<JsThemeColors>,
    /// Dark mode colors.
    pub dark_colors: Option<JsThemeColors>,
    /// Font configuration.
    pub fonts: Option<JsThemeFonts>,
    /// Entry page configuration.
    pub entry_page: Option<JsThemeEntryPage>,
    /// Layout configuration.
    pub layout: Option<JsThemeLayout>,
    /// Header configuration.
    pub header: Option<JsThemeHeader>,
    /// Opt-in header nav items.
    pub nav: Option<Vec<JsHeaderNavItem>>,
    /// Opt-in announcement bar.
    pub announcement: Option<JsThemeAnnouncement>,
    /// Footer configuration.
    pub footer: Option<JsThemeFooter>,
    /// Social links configuration.
    pub social_links: Option<JsSocialLinks>,
    /// Embedded HTML content at specific positions.
    pub embed: Option<JsThemeEmbed>,
    /// Additional custom CSS.
    pub css: Option<String>,
    /// Additional custom JavaScript.
    pub js: Option<String>,
}

/// SSG configuration.
#[napi(object)]
#[derive(Clone)]
pub struct JsSsgConfig {
    /// Site name.
    pub site_name: String,
    /// Base URL path.
    pub base: String,
    /// OG image URL.
    pub og_image: Option<String>,
    /// Theme configuration.
    pub theme: Option<JsThemeConfig>,
    /// Current locale for this page.
    pub locale: Option<String>,
    /// Available locales for locale switcher.
    pub available_locales: Option<Vec<JsLocaleInfo>>,
    /// When true, render previous/next page links after the article.
    pub pagination: Option<bool>,
    /// When true, render a breadcrumb trail above the article.
    pub breadcrumbs: Option<bool>,
    /// Opt-in copy, external-link, and back-to-top chrome.
    pub reader_chrome: Option<JsReaderChrome>,
    /// Opt-in header locale switcher.
    pub locale_switcher: Option<bool>,
    /// Existing sibling hrefs and locale roots.
    pub locale_paths: Option<Vec<JsLocalePath>>,
    /// Opt-in skip link and print styles. Presence enables the feature.
    #[napi(js_name = "a11y")]
    pub a11y: Option<JsA11y>,
    /// Opt-in team / members page. Omitted stays off.
    pub team: Option<JsTeamOptions>,
    /// When true, honor per-page frontmatter chrome flags.
    pub page_chrome: Option<bool>,
}

/// Opt-in skip link and print styles. Presence of the object enables the feature.
#[napi(object, js_name = "JsA11y")]
#[derive(Clone)]
pub struct JsA11y {
    /// Override for the skip-link label. Empty / omitted uses "Skip to content".
    pub skip_link_label: Option<String>,
}

/// One link on a team member card.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsTeamLink {
    /// Visible label.
    pub label: String,
    /// Destination URL.
    pub href: String,
}

/// One person on the team page.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsTeamMember {
    /// Display name.
    pub name: String,
    /// Optional role or title.
    pub role: Option<String>,
    /// Avatar URL.
    pub avatar: Option<String>,
    /// Optional profile or social links.
    pub links: Option<Vec<JsTeamLink>>,
}

/// Opt-in team / members page. Presence of the object enables the feature.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsTeamOptions {
    /// When false, `layout: team` is ignored.
    pub enabled: Option<bool>,
    /// Members rendered as static cards.
    pub members: Option<Vec<JsTeamMember>>,
}

/// Header nav link or dropdown.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsHeaderNavItem {
    /// Display label.
    pub text: String,
    /// Link URL.
    pub link: Option<String>,
    /// Dropdown children.
    pub items: Option<Vec<JsHeaderNavItem>>,
}

/// Announcement bar for JavaScript.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsThemeAnnouncement {
    /// Escaped bar text.
    pub text: String,
    /// Optional https or same-origin link.
    pub link: Option<String>,
    /// localStorage key used to persist dismiss.
    pub dismiss_key: Option<String>,
}

/// Opt-in reader chrome flags. Presence of the object enables the feature.
#[napi(object)]
#[derive(Clone)]
pub struct JsReaderChrome {
    /// Copy button on fenced code blocks.
    pub copy: Option<bool>,
    /// Icon and `rel` on outbound links.
    pub external_links: Option<bool>,
    /// Back-to-top control that appears after scroll.
    pub back_to_top: Option<bool>,
}

/// Locale information for the locale switcher.
#[napi(object)]
#[derive(Clone)]
pub struct JsLocaleInfo {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Text direction.
    pub dir: String,
}

/// Sibling page or locale-root href for one locale.
#[napi(object)]
#[derive(Clone)]
pub struct JsLocalePath {
    /// BCP 47 locale tag.
    pub code: String,
    /// Href of the same page in this locale, when that translation exists.
    pub href: Option<String>,
    /// Locale home href used when `href` is missing.
    pub root: Option<String>,
}
