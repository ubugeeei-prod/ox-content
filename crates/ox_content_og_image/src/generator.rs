//! OG image generator.

use crate::config::OgImageConfig;
use crate::template::{OgImageData, OgImageTemplate};
use thiserror::Error;

/// Result type for OG image operations.
pub type OgImageResult<T> = Result<T, OgImageError>;

/// Errors that can occur during OG image generation.
#[derive(Debug, Error)]
pub enum OgImageError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Font loading error.
    #[error("Failed to load font: {0}")]
    FontLoad(String),

    /// Image encoding error.
    #[error("Failed to encode image: {0}")]
    Encode(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// OG image generator.
pub struct OgImageGenerator {
    config: OgImageConfig,
    template: OgImageTemplate,
}

impl OgImageGenerator {
    /// Creates a new generator with the given configuration.
    #[must_use]
    pub fn new(config: OgImageConfig) -> Self {
        Self { config, template: OgImageTemplate::default() }
    }

    /// Creates a new generator with custom config and template.
    #[must_use]
    pub fn with_template(config: OgImageConfig, template: OgImageTemplate) -> Self {
        Self { config, template }
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &OgImageConfig {
        &self.config
    }

    /// Returns the current template.
    #[must_use]
    pub fn template(&self) -> &OgImageTemplate {
        &self.template
    }

    /// Generates an OG image for the given data.
    ///
    /// Returns the image as a byte vector.
    pub fn generate(&self, data: &OgImageData) -> OgImageResult<Vec<u8>> {
        // TODO: Implement actual image generation using an image library
        // For now, return a placeholder
        let _ = data;
        Err(OgImageError::Encode("Image generation not yet implemented".to_string()))
    }

    /// Generates an OG image and saves it to a file.
    pub fn generate_to_file(
        &self,
        data: &OgImageData,
        output_path: &std::path::Path,
    ) -> OgImageResult<()> {
        let bytes = self.generate(data)?;
        std::fs::write(output_path, bytes)?;
        Ok(())
    }

    /// Generates an SVG representation of the OG image.
    ///
    /// This can be useful for debugging or for rendering in web contexts.
    #[must_use]
    pub fn generate_svg(&self, data: &OgImageData) -> String {
        let width = self.config.width;
        let height = self.config.height;
        let bg = &self.config.background_color;
        let text_color = &self.config.text_color;
        let font_family =
            self.config.font_family.as_deref().unwrap_or("IBM Plex Sans, system-ui, sans-serif");
        let brand = data
            .site_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Ox Content");
        let is_brand_card = normalize_for_compare(&data.title) == normalize_for_compare(brand);
        let hero_title =
            if is_brand_card { "cargo doc for JavaScript".to_string() } else { data.title.clone() };
        let hero_description = if is_brand_card {
            "Rust-powered docs and high-performance Markdown tooling.".to_string()
        } else {
            data.description
                .as_deref()
                .filter(|description| !description.trim().is_empty())
                .unwrap_or("Rust-powered docs and Markdown tooling.")
                .to_string()
        };

        let title_lines = wrap_text_limited(&hero_title, 28, 2);
        let title_line_height = u64::from(self.config.title_font_size) + 10;
        let title_svg = title_lines.iter().enumerate().fold(String::new(), |mut acc, (i, line)| {
            use std::fmt::Write;
            let line_index = u64::try_from(i).unwrap_or(u64::MAX);
            let y = 300_u64.saturating_add(line_index.saturating_mul(title_line_height));
            let _ = write!(
                acc,
                r#"<text x="64" y="{y}" fill="{text_color}" font-size="{}" font-weight="700" letter-spacing="-3.8px" font-family="{font_family}">{}</text>"#,
                self.config.title_font_size,
                escape_xml(line)
            );
            acc
        });

        let description_lines = wrap_text_limited(&hero_description, 56, 2);
        let title_line_offset =
            u64::try_from(title_lines.len().saturating_sub(1)).unwrap_or(u64::MAX);
        let description_start_y = 300_u64
            .saturating_add(title_line_offset.saturating_mul(title_line_height))
            .saturating_add(58);
        let description_line_height = u64::from(self.config.description_font_size) + 14;
        let description_svg =
            description_lines
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, line)| {
                    use std::fmt::Write;
                    let line_index = u64::try_from(i).unwrap_or(u64::MAX);
                    let y = description_start_y
                        .saturating_add(line_index.saturating_mul(description_line_height));
                    let _ = write!(
                        acc,
                        r##"<text x="64" y="{y}" fill="#93a4c3" font-size="{}" font-family="{font_family}">{}</text>"##,
                        self.config.description_font_size,
                        escape_xml(line)
                    );
                    acc
                });

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <rect x="0.5" y="0.5" width="{border_width}" height="{border_height}" fill="none" stroke="#223252"/>
  <rect width="100%" height="4" fill="#4f6fae"/>

  <defs>
    <linearGradient id="brand_mark_gradient" x1="138" y1="118" x2="360" y2="392" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#355cff"/>
      <stop offset="100%" stop-color="#74c7ff"/>
    </linearGradient>
  </defs>

  <g transform="translate(64 56) scale(1.5926)">
    <text x="2" y="43" fill="#eff6ff" font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif" font-size="34" font-weight="700" letter-spacing="-1.4px">OXCONTENT</text>
    <text x="213" y="43.5" fill="#eff6ff" font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif" font-size="40" font-weight="400">(</text>
    <g transform="translate(216 9) scale(0.089) rotate(-7 256 256)">
      <path d="M161 96H286C298 96 309 101 318 110L352 144C361 153 366 164 366 176V386C366 399 355 410 342 410H161C148 410 138 399 138 386V120C138 107 148 96 161 96Z" fill="url(#brand_mark_gradient)"/>
    </g>
    <text x="252" y="43.5" fill="#eff6ff" font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif" font-size="40" font-weight="400">)</text>
  </g>

  {title_svg}
  {description_svg}
</svg>"##,
            border_width = width.saturating_sub(1),
            border_height = height.saturating_sub(1)
        )
    }
}

fn normalize_for_compare(value: &str) -> String {
    value.chars().filter(|ch| !ch.is_whitespace()).flat_map(char::to_lowercase).collect()
}

/// Wraps text into lines of approximately max_chars length.
fn wrap_text_limited(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            if last.len() > 3 {
                last.truncate(last.len() - 3);
                last.push_str("...");
            }
        }
    }

    lines
}

impl Default for OgImageGenerator {
    fn default() -> Self {
        Self::new(OgImageConfig::default())
    }
}

/// Escapes special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_svg() {
        let generator = OgImageGenerator::default();
        let data = OgImageData {
            title: "Test Title".to_string(),
            description: Some("Test description".to_string()),
            site_name: None,
            author: None,
            date: None,
            tags: vec![],
        };

        let svg = generator.generate_svg(&data);
        assert!(svg.contains("Test Title"));
        assert!(svg.contains("Test description"));
    }
}
