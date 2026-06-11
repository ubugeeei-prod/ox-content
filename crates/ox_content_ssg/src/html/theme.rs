use serde::{Deserialize, Serialize};

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
