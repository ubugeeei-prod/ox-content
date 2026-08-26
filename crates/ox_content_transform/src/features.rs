#![allow(clippy::redundant_pub_crate)]

use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::path::PathBuf;

use crate::TransformOptions;

mod attr_tokens;
mod attributes;
mod badges;
mod cards;
pub mod code_blocks;
mod code_groups;
mod code_imports;
mod containers;
mod edit;
mod emoji;
mod emoji_shortcodes;
mod escape;
mod file_tree;
mod images;
mod includes;
mod keyboard_keys;
mod magic;
mod math;
mod not_by_ai;
mod option_resolve;
mod segments;
mod steps;
mod wiki;

use attributes::transform_attribute_syntax;
use cards::ResolvedCardOptions;
pub use code_blocks::{
    CodeBlockDiagnostic, ExtractedCodeBlock, extract_code_blocks, extract_docs_tests,
    lint_code_blocks,
};
use code_groups::ResolvedCodeGroupOptions;
use code_imports::ResolvedCodeImportOptions;
use containers::ResolvedContainerOptions;
use edit::append_edit_this_page;
use emoji_shortcodes::replace_emoji_shortcodes;
pub(super) use escape::{escape_html_attr, escape_html_text};
use file_tree::ResolvedFileTreeOptions;
use images::ResolvedImageOptions;
use includes::ResolvedIncludeOptions;
use magic::ResolvedMagicLinks;
use option_resolve::{
    resolve_attrs, resolve_edit_this_page, resolve_emoji_shortcodes, resolve_wiki_links,
};
use segments::transform_markdown_text_segments;
use steps::ResolvedStepsOptions;
use wiki::replace_wiki_links;

#[derive(Clone, Default)]
pub struct TransformFeatureOptions {
    wiki_links: Option<ResolvedWikiLinkOptions>,
    emoji_shortcodes: Option<ResolvedEmojiShortcodeOptions>,
    code_imports: Option<ResolvedCodeImportOptions>,
    containers: Option<ResolvedContainerOptions>,
    includes: Option<ResolvedIncludeOptions>,
    cards: Option<ResolvedCardOptions>,
    steps: Option<ResolvedStepsOptions>,
    code_groups: Option<ResolvedCodeGroupOptions>,
    file_tree: Option<ResolvedFileTreeOptions>,
    badges: bool,
    not_by_ai: Option<not_by_ai::ResolvedNotByAi>,
    keyboard_keys: Option<keyboard_keys::ResolvedKeyboardKeys>,
    magic_links: Option<ResolvedMagicLinks>,
    images: Option<ResolvedImageOptions>,
    math: bool,
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
        let cards = cards::resolve(options.cards.as_ref());
        let steps = steps::resolve(options.steps.as_ref());
        let code_groups = code_groups::resolve(options.code_groups.as_ref());
        let mut containers = containers::resolve(options.containers.as_ref());
        if (cards.is_some() || steps.is_some() || code_groups.is_some())
            && let Some(containers) = containers.as_mut()
        {
            if cards.is_some() {
                for name in cards::reserved_type_names() {
                    containers.types.remove(*name);
                }
            }
            if steps.is_some() {
                containers.types.remove("steps");
            }
            if code_groups.is_some() {
                for name in code_groups::reserved_type_names() {
                    containers.types.remove(*name);
                }
            }
        }
        let includes = includes::resolve(options.includes.as_ref(), source_path);
        let file_tree = file_tree::resolve(options.file_tree.as_ref());
        let badges = badges::resolve(options.badges.as_ref());
        let not_by_ai = not_by_ai::resolve(options.not_by_ai.as_ref());
        let keyboard_keys = keyboard_keys::resolve(options.keyboard_keys.as_ref());
        let magic_links = magic::resolve(options.magic_links.as_ref());
        let images = images::resolve(options.images.as_ref(), attributes);
        let math = math::resolve(options.math.as_ref());
        let edit_this_page = resolve_edit_this_page(
            options.edit_this_page.as_ref(),
            source_path.unwrap_or_default(),
        );

        Self {
            wiki_links,
            emoji_shortcodes,
            code_imports,
            containers,
            includes,
            cards,
            steps,
            code_groups,
            file_tree,
            badges,
            not_by_ai,
            keyboard_keys,
            magic_links,
            images,
            math,
            attributes,
            edit_this_page,
        }
    }

    pub fn has_preprocess(&self) -> bool {
        self.wiki_links.is_some()
            || self.emoji_shortcodes.is_some()
            || self.code_imports.is_some()
            || self.containers.is_some()
            || self.includes.is_some()
            || self.cards.is_some()
            || self.steps.is_some()
            || self.code_groups.is_some()
            || self.file_tree.is_some()
            || self.badges
            || self.not_by_ai.is_some()
            || self.keyboard_keys.is_some()
            || self.magic_links.is_some()
            || self.images.is_some()
            || self.math
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

    if let Some(includes) = &options.includes
        && current.contains("<!--")
    {
        let replaced = includes::transform(&current, includes, &mut errors);
        current = Cow::Owned(replaced);
    }

    if let Some(code_imports) = &options.code_imports
        && current.contains("<<<")
    {
        let replaced = code_imports::transform(&current, code_imports, &mut errors);
        current = Cow::Owned(replaced);
    }

    if let Some(wiki_links) = &options.wiki_links
        && current.contains("[[")
    {
        let replaced = transform_markdown_text_segments(&current, |segment, out| {
            replace_wiki_links(segment, wiki_links, out);
        });
        if let Some(replaced) = replaced {
            current = Cow::Owned(replaced);
        }
    }

    if let Some(emoji) = &options.emoji_shortcodes
        && current.contains(':')
    {
        let replaced = transform_markdown_text_segments(&current, |segment, out| {
            replace_emoji_shortcodes(segment, emoji, out);
        });
        if let Some(replaced) = replaced {
            current = Cow::Owned(replaced);
        }
    }

    if current.contains(":::") {
        if options.cards.is_some() {
            current = Cow::Owned(cards::transform(&current));
        }
        if options.steps.is_some() {
            current = Cow::Owned(steps::transform(&current));
        }
        if options.code_groups.is_some() {
            current = Cow::Owned(code_groups::transform(&current, &mut errors));
        }
        if let Some(containers) = &options.containers {
            current = Cow::Owned(containers::transform(&current, containers));
        }
    }

    if let Some(file_tree) = &options.file_tree
        && current.contains("file-tree")
    {
        current = Cow::Owned(file_tree::transform(&current, file_tree));
    }

    if options.badges && current.contains("{badge:") {
        let replaced = transform_markdown_text_segments(&current, |segment, out| {
            badges::replace(segment, out);
        });
        if let Some(replaced) = replaced {
            current = Cow::Owned(replaced);
        }
    }
    keyboard_keys::apply(&mut current, options.keyboard_keys.as_ref());
    not_by_ai::apply(&mut current, options.not_by_ai.as_ref());
    if let Some(magic_links) = &options.magic_links
        && current.contains("{link:")
        && let Some(replaced) = magic::transform(&current, magic_links)
    {
        current = Cow::Owned(replaced);
    }
    if let Some(images) = &options.images
        && let Some(replaced) = images::preprocess(&current, images)
    {
        current = Cow::Owned(replaced);
    }

    math::apply(&mut current, options.math);

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

#[cfg(test)]
mod wiring_tests;
