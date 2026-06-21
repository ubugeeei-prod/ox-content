//! JavaScript runtime module generation for Ox Content i18n.

use rustc_hash::FxHashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::dictionary::DictionarySet;
use crate::I18nResult;

const RUNTIME_TEMPLATE: &str = include_str!("../templates/i18n-runtime.js");
const DEFAULT_LOCALE_TOKEN: &str = "__OX_I18N_DEFAULT_LOCALE__";
const LOCALES_TOKEN: &str = "__OX_I18N_LOCALES__";
const HIDE_DEFAULT_LOCALE_TOKEN: &str = "__OX_I18N_HIDE_DEFAULT_LOCALE__";
const DICTIONARIES_TOKEN: &str = "__OX_I18N_DICTIONARIES__";

/// Locale metadata embedded in the generated runtime module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nRuntimeLocale {
    /// BCP 47 locale tag.
    pub code: String,
    /// Display name for the locale.
    pub name: String,
    /// Text direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
}

/// Configuration embedded in the generated runtime module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct I18nRuntimeConfig {
    /// Default locale tag.
    pub default_locale: String,
    /// Available locales.
    pub locales: Vec<I18nRuntimeLocale>,
    /// Whether URLs should omit the default locale prefix.
    pub hide_default_locale: bool,
}

/// Flat dictionaries keyed by locale, then translation key.
pub type FlatDictionaries = FxHashMap<String, FxHashMap<String, String>>;

/// Converts a dictionary set into the flat shape consumed by the JS runtime.
#[must_use]
pub fn flatten_dictionary_set(set: &DictionarySet) -> FlatDictionaries {
    let mut result = FxHashMap::default();

    for locale in set.locales() {
        if let Some(dict) = set.get(locale) {
            let flat = dict
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<FxHashMap<_, _>>();
            result.insert(locale.to_string(), flat);
        }
    }

    result
}

/// Loads dictionaries from disk and converts them into the runtime shape.
pub fn load_flat_dictionaries(dir: &Path) -> I18nResult<FlatDictionaries> {
    let set = crate::dictionary::load_from_dir(dir)?;
    Ok(flatten_dictionary_set(&set))
}

/// Generates the virtual JavaScript module used by `virtual:ox-content/i18n`.
#[must_use]
pub fn generate_runtime_module(
    config: &I18nRuntimeConfig,
    dictionaries: &FlatDictionaries,
) -> String {
    let default_locale =
        serde_json::to_string(&config.default_locale).unwrap_or_else(|_| "\"en\"".to_string());
    let locales = serde_json::to_string(&config.locales).unwrap_or_else(|_| "[]".to_string());
    let hide_default_locale =
        serde_json::to_string(&config.hide_default_locale).unwrap_or_else(|_| "true".to_string());
    let dictionaries = serde_json::to_string(dictionaries).unwrap_or_else(|_| "{}".to_string());

    RUNTIME_TEMPLATE
        .replace(DEFAULT_LOCALE_TOKEN, &default_locale)
        .replace(LOCALES_TOKEN, &locales)
        .replace(HIDE_DEFAULT_LOCALE_TOKEN, &hide_default_locale)
        .replace(DICTIONARIES_TOKEN, &dictionaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> I18nRuntimeConfig {
        I18nRuntimeConfig {
            default_locale: "en-US".to_string(),
            locales: vec![
                I18nRuntimeLocale {
                    code: "en-US".to_string(),
                    name: "English".to_string(),
                    dir: None,
                },
                I18nRuntimeLocale {
                    code: "ar".to_string(),
                    name: "Arabic".to_string(),
                    dir: Some("rtl".to_string()),
                },
            ],
            hide_default_locale: true,
        }
    }

    #[test]
    fn runtime_module_embeds_config_and_dictionaries() {
        let mut dictionaries = FlatDictionaries::default();
        dictionaries.insert(
            "en-US".to_string(),
            std::iter::once(("common.greeting".to_string(), "Hello".to_string())).collect(),
        );

        let module = generate_runtime_module(&test_config(), &dictionaries);

        insta::assert_snapshot!(module);
    }
}
