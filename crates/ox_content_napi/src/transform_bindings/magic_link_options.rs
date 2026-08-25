use std::collections::HashMap;

use napi_derive::napi;
use ox_content_transform::{MagicLinkAlias, MagicLinkImageOverride, MagicLinkOptions};

/// One configured magic-link alias.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMagicLinkAlias {
    /// Absolute `http:` / `https:` URL.
    pub href: String,
    /// Optional display label. Defaults to the alias key.
    pub label: Option<String>,
    /// Optional avatar or favicon URL.
    pub image: Option<String>,
}

impl From<JsMagicLinkAlias> for MagicLinkAlias {
    fn from(value: JsMagicLinkAlias) -> Self {
        Self { href: value.href, label: value.label, image: value.image }
    }
}

/// Replace the resolved image for matching hrefs.
#[napi(object)]
#[derive(Default, Clone)]
pub struct JsMagicLinkImageOverride {
    /// Exact href to replace.
    pub href: Option<String>,
    /// Href prefix to replace.
    pub prefix: Option<String>,
    /// Replacement image URL. Must be `http:` or `https:`.
    pub image: String,
}

impl From<JsMagicLinkImageOverride> for MagicLinkImageOverride {
    fn from(value: JsMagicLinkImageOverride) -> Self {
        Self { href: value.href, prefix: value.prefix, image: value.image }
    }
}

/// Opt-in `{link:...}` rich magic links.
#[napi(object)]
#[derive(Default, Clone)]
#[allow(clippy::disallowed_types)]
pub struct JsMagicLinkOptions {
    /// Enable `{link:@user}`, `{link:alias}`, and `{link:label|url}`.
    ///
    /// Default: `false`.
    pub enabled: Option<bool>,

    /// Named aliases mapped to href / label / image.
    pub aliases: Option<HashMap<String, JsMagicLinkAlias>>,

    /// Emit a favicon URL for explicit links that have no image.
    ///
    /// Default: `false`. No transform-time network request.
    pub favicon: Option<bool>,

    /// Favicon URL template with a `{host}` placeholder.
    ///
    /// Default: `https://{host}/favicon.ico` when `favicon` is true.
    pub favicon_template: Option<String>,

    /// Image replacements applied after alias / GitHub / favicon resolution.
    pub image_overrides: Option<Vec<JsMagicLinkImageOverride>>,
}

impl From<JsMagicLinkOptions> for MagicLinkOptions {
    fn from(value: JsMagicLinkOptions) -> Self {
        Self {
            enabled: value.enabled,
            aliases: value
                .aliases
                .map(|aliases| aliases.into_iter().map(|(key, spec)| (key, spec.into())).collect()),
            favicon: value.favicon,
            favicon_template: value.favicon_template,
            image_overrides: value
                .image_overrides
                .map(|rules| rules.into_iter().map(Into::into).collect()),
        }
    }
}
