#![allow(clippy::redundant_pub_crate)]

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::{
    AttrsOptions, EditThisPageOptions, EmojiShortcodeOptions, TransformOptions, WikiLinkOptions,
};

mod attr_tokens;
mod attributes;
pub mod code_blocks;
mod code_imports;
mod edit;
mod emoji;
mod emoji_shortcodes;
mod segments;
mod wiki;

use attributes::transform_attribute_syntax;
pub use code_blocks::{
    extract_code_blocks, extract_docs_tests, lint_code_blocks, CodeBlockDiagnostic,
    ExtractedCodeBlock,
};
use code_imports::ResolvedCodeImportOptions;
use edit::append_edit_this_page;
use emoji_shortcodes::replace_emoji_shortcodes;
use segments::transform_markdown_text_segments;
use wiki::replace_wiki_links;

#[derive(Clone, Default)]
pub struct TransformFeatureOptions {
    wiki_links: Option<ResolvedWikiLinkOptions>,
    emoji_shortcodes: Option<ResolvedEmojiShortcodeOptions>,
    code_imports: Option<ResolvedCodeImportOptions>,
    attributes: bool,
    edit_this_page: Option<ResolvedEditThisPageOptions>,
}

#[derive(Clone)]
struct ResolvedWikiLinkOptions {
    base_url: String,
}

#[derive(Clone)]
struct ResolvedEmojiShortcodeOptions {
    custom: FxHashMap<String, String>,
}

#[derive(Clone)]
struct ResolvedEditThisPageOptions {
    repo_url: String,
    branch: String,
    root_dir: PathBuf,
    source_path: String,
    label: String,
}

pub struct PreprocessResult<'a> {
    pub source: Cow<'a, str>,
    pub errors: Vec<String>,
}

pub struct PostprocessResult {
    pub html: String,
    pub errors: Vec<String>,
}

impl TransformFeatureOptions {
    pub fn from_options(options: &TransformOptions) -> Self {
        let wiki_links = resolve_wiki_links(options.wiki_links.as_ref(), options.base_url.as_ref());
        let emoji_shortcodes = resolve_emoji_shortcodes(options.emoji_shortcodes.as_ref());
        let source_path = options.source_path.as_deref().filter(|value| !value.is_empty());
        let code_imports = code_imports::resolve(options.code_imports.as_ref(), source_path);
        let attributes = resolve_attrs(options.attributes.as_ref());
        let edit_this_page = resolve_edit_this_page(
            options.edit_this_page.as_ref(),
            source_path.unwrap_or_default(),
        );

        Self { wiki_links, emoji_shortcodes, code_imports, attributes, edit_this_page }
    }

    pub fn has_preprocess(&self) -> bool {
        self.wiki_links.is_some() || self.emoji_shortcodes.is_some() || self.code_imports.is_some()
    }

    pub fn has_postprocess(&self) -> bool {
        self.attributes || self.edit_this_page.is_some()
    }
}

pub fn preprocess_markdown<'a>(
    source: &'a str,
    options: &TransformFeatureOptions,
) -> PreprocessResult<'a> {
    if !options.has_preprocess() {
        return PreprocessResult { source: Cow::Borrowed(source), errors: Vec::new() };
    }

    let mut current = Cow::Borrowed(source);
    let mut errors = Vec::new();

    if let Some(code_imports) = &options.code_imports {
        if current.contains("<<<") {
            let replaced = code_imports::transform(&current, code_imports, &mut errors);
            current = Cow::Owned(replaced);
        }
    }

    if let Some(wiki_links) = &options.wiki_links {
        if current.contains("[[") {
            let replaced = transform_markdown_text_segments(&current, |segment, out| {
                replace_wiki_links(segment, wiki_links, out);
            });
            if let Some(replaced) = replaced {
                current = Cow::Owned(replaced);
            }
        }
    }

    if let Some(emoji) = &options.emoji_shortcodes {
        if current.contains(':') {
            let replaced = transform_markdown_text_segments(&current, |segment, out| {
                replace_emoji_shortcodes(segment, emoji, out);
            });
            if let Some(replaced) = replaced {
                current = Cow::Owned(replaced);
            }
        }
    }

    PreprocessResult { source: current, errors }
}

pub fn postprocess_html(html: &str, options: &TransformFeatureOptions) -> PostprocessResult {
    if !options.has_postprocess() {
        return PostprocessResult { html: html.to_string(), errors: Vec::new() };
    }

    let mut current = Cow::Borrowed(html);
    let errors = Vec::new();

    if options.attributes && current.contains('{') {
        let transformed = transform_attribute_syntax(&current);
        if let Some(transformed) = transformed {
            current = Cow::Owned(transformed);
        }
    }

    if let Some(edit) = &options.edit_this_page {
        let transformed = append_edit_this_page(&current, edit);
        current = Cow::Owned(transformed);
    }

    PostprocessResult { html: current.into_owned(), errors }
}

fn resolve_wiki_links(
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

fn resolve_emoji_shortcodes(
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

fn resolve_attrs(options: Option<&AttrsOptions>) -> bool {
    options.is_some_and(|options| options.enabled != Some(false))
}

fn resolve_edit_this_page(
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

    Some(ResolvedEditThisPageOptions {
        repo_url,
        branch: options.branch.clone().unwrap_or_else(|| "main".to_string()),
        root_dir: options.root_dir.as_deref().filter(|value| !value.is_empty()).map_or_else(
            || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from,
        ),
        source_path: source_path.to_string(),
        label: options.label.clone().unwrap_or_else(|| "Edit this page".to_string()),
    })
}

fn escape_html_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn escape_html_attr(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_links_become_markdown_links() {
        let options = ResolvedWikiLinkOptions { base_url: "/docs/".to_string() };
        let mut out = String::new();
        replace_wiki_links("See [[Guide Page#Install|the guide]].", &options, &mut out);
        assert_eq!(out, "See [the guide](/docs/Guide%20Page#install).");
    }

    #[test]
    fn emoji_shortcodes_use_defaults_and_custom_values() {
        let options = ResolvedEmojiShortcodeOptions {
            custom: std::iter::once(("shipit".to_string(), "ship".to_string())).collect(),
        };
        let mut out = String::new();
        replace_emoji_shortcodes(":smile: :shipit: :octocat: :unknown:", &options, &mut out);
        assert_eq!(out, "\u{1F604} ship \u{1F431} :unknown:");
    }

    #[test]
    fn extracts_docs_test_blocks_by_meta() {
        let blocks = extract_docs_tests(
            "```ts test\nexpect(1).toBe(1)\n```\n```js\nnoop()\n```",
            Some(&crate::DocsTestOptions {
                enabled: Some(true),
                languages: None,
                require_meta: Some(true),
            }),
        );
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "ts");
    }

    #[test]
    fn lints_code_block_trailing_spaces() {
        let diagnostics = lint_code_blocks(
            "```ts\nconst x = 1;  \n```",
            Some(&crate::CodeBlockLintOptions {
                enabled: Some(true),
                languages: None,
                require_language: Some(false),
                trailing_spaces: Some(true),
            }),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, 2);
    }
}
