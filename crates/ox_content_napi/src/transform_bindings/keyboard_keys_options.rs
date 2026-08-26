use std::collections::HashMap;

use napi_derive::napi;
use ox_content_transform::KeyboardKeysOptions;

/// Opt-in `{kbd:...}` inline keyboard keys.
#[napi(object)]
#[derive(Default, Clone)]
#[allow(clippy::disallowed_types)]
pub struct JsKeyboardKeysOptions {
    /// Enable `{kbd:Ctrl+K}` expansion.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Build-time key aliases. Keys are matched case-insensitively.
    ///
    /// Default: `{}`.
    pub aliases: Option<HashMap<String, String>>,

    /// Label style for built-in aliases: `"words"` or `"symbols"`.
    ///
    /// Default: `"words"`.
    pub style: Option<String>,
}

impl From<JsKeyboardKeysOptions> for KeyboardKeysOptions {
    fn from(value: JsKeyboardKeysOptions) -> Self {
        Self {
            enabled: value.enabled,
            aliases: value.aliases.map(|values| values.into_iter().collect()),
            style: value.style,
        }
    }
}
