//! Vite plugin implementation.

use crate::config::OxContentConfig;

/// Ox Content Vite plugin.
pub struct OxContentPlugin {
    config: OxContentConfig,
}

impl OxContentPlugin {
    /// Creates a new plugin instance.
    #[must_use]
    pub fn new(config: OxContentConfig) -> Self {
        Self { config }
    }

    /// Returns the plugin configuration.
    #[must_use]
    pub fn config(&self) -> &OxContentConfig {
        &self.config
    }

    /// Resolves a Markdown file path.
    #[must_use]
    pub fn resolve_markdown(&self, path: &str) -> Option<String> {
        let src_dir = &self.config.src_dir;
        let is_markdown = std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if path.starts_with(src_dir) && is_markdown { Some(path.to_string()) } else { None }
    }

    /// Transforms a Markdown file.
    pub fn transform(&self, _path: &str, _content: &str) -> Result<String, String> {
        // TODO: Implement transformation
        Ok(String::new())
    }
}

impl Default for OxContentPlugin {
    fn default() -> Self {
        Self::new(OxContentConfig::default())
    }
}
