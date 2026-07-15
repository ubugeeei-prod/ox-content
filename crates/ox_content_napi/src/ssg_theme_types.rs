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
