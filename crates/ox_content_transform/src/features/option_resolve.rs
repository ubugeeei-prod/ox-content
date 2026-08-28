use std::path::PathBuf;

use crate::{AttrsOptions, EditThisPageOptions, EmojiShortcodeOptions, WikiLinkOptions};

use super::edit::provider::resolve_pattern;
use super::{ResolvedEditThisPageOptions, ResolvedEmojiShortcodeOptions, ResolvedWikiLinkOptions};

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
