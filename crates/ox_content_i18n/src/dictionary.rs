pub mod json;
pub mod yaml;

use std::collections::HashMap;
use std::path::Path;

use crate::error::{I18nError, I18nResult};
use crate::key::KeyPath;
use crate::locale::Locale;

/// A flat map of translation keys to their MF2 message strings for one locale.
#[derive(Debug, Clone, Default)]
pub struct Dictionary {
    entries: HashMap<String, String>,
}

impl Dictionary {
    /// Creates an empty dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a translation entry.
    pub fn insert(&mut self, key: KeyPath, value: String) {
        self.entries.insert(key.as_str().to_string(), value);
    }

    /// Looks up a translation by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the dictionary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// Returns an iterator over all (key, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// A collection of dictionaries, one per locale.
#[derive(Debug, Clone, Default)]
pub struct DictionarySet {
    dictionaries: HashMap<String, Dictionary>,
    default_locale: Option<Locale>,
}

impl DictionarySet {
    /// Creates an empty dictionary set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the default locale.
    pub fn set_default_locale(&mut self, locale: Locale) {
        self.default_locale = Some(locale);
    }

    /// Returns the default locale if set.
    #[must_use]
    pub fn default_locale(&self) -> Option<&Locale> {
        self.default_locale.as_ref()
    }

    /// Inserts a dictionary for a given locale.
    pub fn insert(&mut self, locale: Locale, dict: Dictionary) {
        self.dictionaries.insert(locale.as_str().to_string(), dict);
    }

    /// Returns the dictionary for the given locale.
    #[must_use]
    pub fn get(&self, locale: &str) -> Option<&Dictionary> {
        self.dictionaries.get(locale)
    }

    /// Returns all locale tags that have dictionaries.
    pub fn locales(&self) -> impl Iterator<Item = &str> {
        self.dictionaries.keys().map(String::as_str)
    }

    /// Returns the number of locales.
    #[must_use]
    pub fn locale_count(&self) -> usize {
        self.dictionaries.len()
    }

    /// Translates a key for the given locale, falling back to the default locale.
    #[must_use]
    pub fn translate(&self, locale: &str, key: &str) -> Option<&str> {
        // Try the requested locale
        if let Some(dict) = self.dictionaries.get(locale) {
            if let Some(value) = dict.get(key) {
                return Some(value);
            }
        }

        // Fall back to default locale
        if let Some(default) = &self.default_locale {
            if default.as_str() != locale {
                if let Some(dict) = self.dictionaries.get(default.as_str()) {
                    return dict.get(key);
                }
            }
        }

        None
    }
}

/// Loads dictionaries from a directory structure.
///
/// Expected layout:
/// ```text
/// dir/
///   en/
///     common.json
///     navigation.json
///   ja/
///     common.json
///     navigation.json
/// ```
pub fn load_from_dir(dir: &Path) -> I18nResult<DictionarySet> {
    let mut set = DictionarySet::new();

    let entries = std::fs::read_dir(dir).map_err(|e| I18nError::DictionaryLoad {
        locale: dir.display().to_string(),
        message: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| I18nError::DictionaryLoad {
            locale: dir.display().to_string(),
            message: e.to_string(),
        })?;

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let locale_str =
            path.file_name().and_then(|n| n.to_str()).ok_or_else(|| I18nError::DictionaryLoad {
                locale: path.display().to_string(),
                message: "invalid directory name".to_string(),
            })?;

        // Skip directories starting with `_` (config files)
        if locale_str.starts_with('_') {
            continue;
        }

        let locale = Locale::new(locale_str)?;
        let dict = load_locale_dir(&path, locale_str)?;
        set.insert(locale, dict);
    }

    Ok(set)
}

/// Loads all dictionary files from a single locale directory.
fn load_locale_dir(dir: &Path, locale: &str) -> I18nResult<Dictionary> {
    let mut dict = Dictionary::new();

    let entries = std::fs::read_dir(dir).map_err(|e| I18nError::DictionaryLoad {
        locale: locale.to_string(),
        message: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| I18nError::DictionaryLoad {
            locale: locale.to_string(),
            message: e.to_string(),
        })?;

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let namespace = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");

        match ext {
            "json" => {
                let content = std::fs::read_to_string(&path)?;
                json::load_into(&content, namespace, &mut dict).map_err(|e| {
                    I18nError::DictionaryLoad {
                        locale: locale.to_string(),
                        message: format!("{}: {e}", path.display()),
                    }
                })?;
            }
            "yaml" | "yml" => {
                let content = std::fs::read_to_string(&path)?;
                yaml::load_into(&content, namespace, &mut dict).map_err(|e| {
                    I18nError::DictionaryLoad {
                        locale: locale.to_string(),
                        message: format!("{}: {e}", path.display()),
                    }
                })?;
            }
            _ => {
                // Skip unsupported formats
            }
        }
    }

    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_basic_ops() {
        let mut dict = Dictionary::new();
        assert!(dict.is_empty());

        dict.insert(KeyPath::new("greeting"), "Hello".to_string());
        assert_eq!(dict.len(), 1);
        assert_eq!(dict.get("greeting"), Some("Hello"));
        assert_eq!(dict.get("missing"), None);
    }

    #[test]
    fn dictionary_set_translate() {
        let mut set = DictionarySet::new();
        set.set_default_locale(Locale::new("en").unwrap());

        let mut en = Dictionary::new();
        en.insert(KeyPath::new("greeting"), "Hello".to_string());
        en.insert(KeyPath::new("farewell"), "Goodbye".to_string());
        set.insert(Locale::new("en").unwrap(), en);

        let mut ja = Dictionary::new();
        ja.insert(KeyPath::new("greeting"), "こんにちは".to_string());
        set.insert(Locale::new("ja").unwrap(), ja);

        // Direct translation
        assert_eq!(set.translate("ja", "greeting"), Some("こんにちは"));

        // Fallback to default locale
        assert_eq!(set.translate("ja", "farewell"), Some("Goodbye"));

        // Missing key
        assert_eq!(set.translate("ja", "nonexistent"), None);
    }

    #[test]
    fn dictionary_set_locales() {
        let mut set = DictionarySet::new();
        set.insert(Locale::new("en").unwrap(), Dictionary::new());
        set.insert(Locale::new("ja").unwrap(), Dictionary::new());

        let mut locales: Vec<&str> = set.locales().collect();
        locales.sort_unstable();
        assert_eq!(locales, vec!["en", "ja"]);
    }
}
