//! Configuration for OG image generation.

use serde::{Deserialize, Serialize};

/// Configuration for OG image generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OgImageConfig {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Background color (hex).
    pub background_color: String,
    /// Text color (hex).
    pub text_color: String,
    /// Title font size.
    pub title_font_size: u32,
    /// Description font size.
    pub description_font_size: u32,
    /// Font family name.
    pub font_family: Option<String>,
    /// Logo path.
    pub logo_path: Option<String>,
    /// Output format.
    pub format: ImageFormat,
}

impl Default for OgImageConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 630,
            background_color: "#0b1220".to_string(),
            text_color: "#eff6ff".to_string(),
            title_font_size: 70,
            description_font_size: 28,
            font_family: None,
            logo_path: None,
            format: ImageFormat::Png,
        }
    }
}

/// Image output format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// PNG format.
    #[default]
    Png,
    /// JPEG format.
    Jpeg,
    /// WebP format.
    WebP,
}
