use std::path::PathBuf;

use rustc_hash::FxHashMap;

use crate::{AttrsOptions, EditThisPageOptions, EmojiShortcodeOptions, WikiLinkOptions};

#[derive(Clone)]
pub(in crate::features) struct ResolvedWikiLinkOptions {
    pub(in crate::features) base_url: String,
}

#[derive(Clone)]
pub(in crate::features) struct ResolvedEmojiShortcodeOptions {
    pub(in crate::features) custom: FxHashMap<String, String>,
}

#[derive(Clone)]
pub(in crate::features) struct ResolvedEditThisPageOptions {
    pub(in crate::features) repo_url: String,
    pub(in crate::features) branch: String,
    /// `rootDir` as configured, trimmed of blanks and surrounding slashes.
    pub(in crate::features) root_dir: Option<String>,
    /// Absolute source root, when the build supplied one.
    pub(in crate::features) src_dir: Option<PathBuf>,
    /// The build's working directory, resolved once per page.
    pub(in crate::features) working_dir: PathBuf,
    /// Edit-URL template for the forge the repository is hosted on.
    pub(in crate::features) url_pattern: String,
    pub(in crate::features) source_path: String,
    pub(in crate::features) label: String,
}

use super::edit::provider::resolve_pattern;

pub(super) fn resolve_wiki_links(
    options: Option<&WikiLinkOptions>,
    default_base_url: Option<&String>,
) -> Option<ResolvedWikiLinkOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedWikiLinkOptions {
        base_url: options
            .base_url
            .clone()
            .or_else(|| default_base_url.cloned())
            .unwrap_or_else(|| "/".to_string()),
    })
}

pub(super) fn resolve_emoji_shortcodes(
    options: Option<&EmojiShortcodeOptions>,
) -> Option<ResolvedEmojiShortcodeOptions> {
    let options = options?;
    if options.enabled == Some(false) {
        return None;
    }
    Some(ResolvedEmojiShortcodeOptions {
        custom: options.custom.clone().unwrap_or_default().into_iter().collect(),
    })
}

pub(super) fn resolve_attrs(options: Option<&AttrsOptions>) -> bool {
    options.is_some_and(|options| options.enabled != Some(false))
}

pub(super) fn resolve_edit_this_page(
    options: Option<&EditThisPageOptions>,
    source_path: &str,
) -> Option<ResolvedEditThisPageOptions> {
    let options = options?;
    if options.enabled == Some(false) || source_path.is_empty() {
        return None;
    }
    let repo_url = options.repo_url.as_deref()?.trim_end_matches('/').to_string();
    if repo_url.is_empty() {
        return None;
    }

    let root_dir = options
        .root_dir
        .as_deref()
        .map(str::trim)
        .map(|value| value.strip_prefix("./").unwrap_or(value))
        .filter(|value| !value.is_empty() && *value != ".")
        .map(ToOwned::to_owned);

    let url_pattern =
        resolve_pattern(options.url_pattern.as_deref(), options.provider.as_deref(), &repo_url);

    Some(ResolvedEditThisPageOptions {
        repo_url,
        url_pattern,
        root_dir,
        src_dir: options.src_dir.as_deref().filter(|value| !value.is_empty()).map(PathBuf::from),
        working_dir: working_directory(),
        branch: options.branch.clone().unwrap_or_else(|| "main".to_string()),
        source_path: source_path.to_string(),
        label: options.label.clone().unwrap_or_else(|| "Edit this page".to_string()),
    })
}

fn working_directory() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
