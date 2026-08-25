//! Opt-in `{link:...}` rich magic links.
//!
//! Disabled by default. When enabled, a single source walk rewrites GitHub-user
//! shorthands, configured aliases, and explicit `label|url` forms into escaped
//! HTML. Fences, code, raw HTML, and already-linked text stay literal.
//!
//! Inspired by [markdown-it-magic-link](https://github.com/antfu/markdown-it-magic-link).

use rustc_hash::FxHashMap;

use crate::{MagicLinkAlias, MagicLinkImageOverride, MagicLinkOptions};

mod emit;
mod scan;

#[cfg(test)]
mod tests;

pub(super) const OPEN: &str = "{link:";
const DEFAULT_FAVICON_TEMPLATE: &str = concat!("https://", "{", "host}", "/favicon.ico");

#[derive(Clone)]
pub(super) struct ResolvedMagicLinks {
    aliases: FxHashMap<String, MagicLinkAlias>,
    favicon_template: Option<String>,
    image_overrides: Vec<MagicLinkImageOverride>,
}

pub(super) fn resolve(options: Option<&MagicLinkOptions>) -> Option<ResolvedMagicLinks> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedMagicLinks {
        aliases: options.aliases.clone().unwrap_or_default(),
        favicon_template: if options.favicon == Some(true) {
            Some(
                options
                    .favicon_template
                    .clone()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| DEFAULT_FAVICON_TEMPLATE.to_string()),
            )
        } else {
            None
        },
        image_overrides: options.image_overrides.clone().unwrap_or_default(),
    })
}

pub(super) fn transform(source: &str, options: &ResolvedMagicLinks) -> Option<String> {
    scan::transform(source, options)
}
