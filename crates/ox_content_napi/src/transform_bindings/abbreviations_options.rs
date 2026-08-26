use std::collections::HashMap;

use napi_derive::napi;
use ox_content_transform::AbbreviationsOptions;

/// Opt-in abbreviation and glossary expansion.
#[napi(object)]
#[derive(Default, Clone)]
#[allow(clippy::disallowed_types)]
pub struct JsAbbreviationsOptions {
    /// Enable `*[TERM]: expansion` and config-term rewriting.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Central glossary terms. Keys are matched with Unicode word boundaries.
    ///
    /// Default: `{}`.
    pub terms: Option<HashMap<String, String>>,

    /// Wrap only the first occurrence of each term.
    ///
    /// Default: `false` (every occurrence).
    pub first_use_only: Option<bool>,
}

impl From<JsAbbreviationsOptions> for AbbreviationsOptions {
    fn from(value: JsAbbreviationsOptions) -> Self {
        Self {
            enabled: value.enabled,
            terms: value.terms.map(|values| values.into_iter().collect()),
            first_use_only: value.first_use_only,
        }
    }
}
