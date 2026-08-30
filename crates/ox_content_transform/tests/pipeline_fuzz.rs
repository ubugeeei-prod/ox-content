//! A bounded fuzz lane for the whole transform pipeline.
//!
//! [issue #774] asks that malformed Markdown return errors or diagnostics
//! instead of aborting the host process, and lists "bounded fuzz /
//! property lanes in CI" as remaining work. Release artifacts build with
//! `panic = "abort"`, so a slice at a bad index in any feature pass kills
//! Node — a hand-written case list only ever covers the inputs someone
//! thought of.
//!
//! Two generators run against every feature at once. One assembles token
//! soup out of the fragments that break scanners: half-open fences and
//! containers, stray delimiters, multi-byte characters, control
//! characters, and bidi overrides. The other assembles documents out of
//! real block templates with real inline content, so the passes are
//! reached through the shapes they are written for rather than bouncing
//! off a parse failure at the first line.
//!
//! Both are deterministic: a failure prints the seed and the document.
//!
//! This lane found two aborts on ordinary input — a second `{.class}`
//! block in a paragraph, and a line starting with `<` followed by
//! Japanese after a definition list.
//!
//! [issue #774]: https://github.com/ubugeeei-prod/ox-content/issues/774

use std::panic::{AssertUnwindSafe, catch_unwind};

use ox_content_transform::transformer::MarkdownTransformer;
use ox_content_transform::*;

#[path = "pipeline_fuzz/corpus.rs"]
mod corpus;

use corpus::document;

/// Every feature the transformer can run, so one document exercises all
/// of the passes rather than the handful a focused test would enable.
fn all_features() -> TransformOptions {
    TransformOptions {
        gfm: Some(true),
        mdx: Some(true),
        footnotes: Some(true),
        task_lists: Some(true),
        tables: Some(true),
        strikethrough: Some(true),
        autolinks: Some(true),
        frontmatter: Some(true),
        toc_max_depth: Some(6),
        convert_md_links: Some(true),
        base_url: Some("/base/".into()),
        source_path: Some("docs/page.md".into()),
        code_annotations: Some(true),
        code_annotation_meta_key: Some("meta".into()),
        code_annotation_syntax: Some("shiki".into()),
        code_annotation_default_line_numbers: Some(true),
        autolink_urls: Some(true),
        autolink_patterns: Some(vec!["\\bOX-\\d+".into()]),
        autolink_target_blank: Some(true),
        semantic_footnotes: Some(true),
        heading_permalinks: Some(true),
        wiki_links: Some(WikiLinkOptions { enabled: Some(true), base_url: Some("/w/".into()) }),
        emoji_shortcodes: Some(EmojiShortcodeOptions { enabled: Some(true), custom: None }),
        math: Some(MathOptions { enabled: Some(true) }),
        attributes: Some(AttrsOptions { enabled: Some(true) }),
        cjk_emphasis: Some(true),
        code_imports: None,
        sanitize: Some(SanitizeOptions::default()),
        edit_this_page: Some(EditThisPageOptions {
            enabled: Some(true),
            repo_url: Some("https://github.com/a/b".into()),
            branch: Some("main".into()),
            ..Default::default()
        }),
        containers: Some(ContainerOptions { enabled: Some(true), types: None }),
        // Left off: both read files from disk.
        includes: None,
        partials: None,
        steps: Some(StepsOptions { enabled: Some(true) }),
        code_groups: Some(CodeGroupOptions { enabled: Some(true) }),
        badges: Some(BadgeOptions { enabled: Some(true) }),
        not_by_ai: Some(NotByAiOptions { enabled: Some(true), label: None, href: None }),
        keyboard_keys: Some(KeyboardKeysOptions {
            enabled: Some(true),
            aliases: None,
            style: None,
        }),
        abbreviations: Some(AbbreviationsOptions {
            enabled: Some(true),
            terms: None,
            first_use_only: Some(true),
        }),
        definition_lists: Some(DefinitionListOptions { enabled: Some(true) }),
        magic_links: Some(MagicLinkOptions {
            enabled: Some(true),
            aliases: None,
            favicon: Some(true),
            favicon_template: None,
            image_overrides: None,
        }),
        images: Some(ImageOptions { enabled: Some(true), lazy: Some(true) }),
        image_galleries: Some(ImageGalleryOptions {
            enabled: Some(true),
            lazy: Some(true),
            missing_alt: None,
            empty: None,
        }),
        timelines: Some(TimelineOptions {
            enabled: Some(true),
            ordered: Some(true),
            invalid_date: None,
            unknown_meta: None,
            empty: None,
        }),
        conditional_blocks: Some(ConditionalBlockOptions { enabled: Some(true), values: None }),
        cards: Some(CardOptions { enabled: Some(true) }),
        file_tree: Some(FileTreeOptions {
            enabled: Some(true),
            default_open: Some(true),
            icons: Some(true),
            icon_folder: None,
            icon_folder_open: None,
            icon_file: None,
            icon_files: None,
        }),
        data_tables: None,
    }
}

#[test]
fn no_document_aborts_the_pipeline() {
    let transformer = MarkdownTransformer::from_options(&all_features());
    let mut failures = Vec::new();
    for seed in 1..30_000u64 {
        let source = document(seed);
        match catch_unwind(AssertUnwindSafe(|| transformer.transform(&source))) {
            Err(_) => failures.push(format!("seed {seed} panicked on {source:?}")),
            Ok(result) => {
                // Debug builds convert an unexpected panic into a
                // diagnostic; release builds abort outright, so treat one
                // as the same failure.
                if result.errors.iter().any(|error| error.to_lowercase().contains("panic")) {
                    let errors = result.errors;
                    failures.push(format!("seed {seed} reported {errors:?} on {source:?}"));
                }
            }
        }
        if failures.len() > 4 {
            break;
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn reusing_a_transformer_gives_the_same_html() {
    // One transformer serves every page of a build, so anything it carries
    // from one document to the next shows up as a page that renders
    // differently depending on what was rendered before it.
    let shared = MarkdownTransformer::from_options(&all_features());
    let mut failures = Vec::new();
    for seed in 1..8_000u64 {
        let source = document(seed);
        let fresh = MarkdownTransformer::from_options(&all_features()).transform(&source);
        let reused = shared.transform(&source);
        if fresh.html != reused.html {
            failures.push(format!("seed {seed} differs after earlier documents: {source:?}"));
        }
        let again = shared.transform(&source);
        if again.html != reused.html {
            failures.push(format!("seed {seed} differs on a second call: {source:?}"));
        }
        if failures.len() > 4 {
            break;
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
