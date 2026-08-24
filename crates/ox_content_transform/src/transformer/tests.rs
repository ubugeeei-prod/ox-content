use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

use super::{
    MarkdownTransformer,
    frontmatter::{SourceOrigin, parse_frontmatter_with_origin},
    toc::extract_toc,
};
use crate::TransformOptions;

#[test]
fn transforms_markdown_with_frontmatter_and_toc() {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        toc_max_depth: Some(2),
        ..Default::default()
    });
    let result = transformer.transform("---\ntitle: Example\n---\n# Hello\n\nThis is a paragraph.");

    assert!(result.errors.is_empty());
    insta::assert_snapshot!(format!(
        "html:\n{}\nfrontmatter:\n{}",
        result.html, result.frontmatter
    ));
    assert_eq!(result.toc.len(), 1);
    assert_eq!(result.toc[0].slug, "hello");
}

#[test]
fn mdx_transform_option_reaches_the_parser_and_renderer() {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        mdx: Some(true),
        ..Default::default()
    });
    let result = transformer.transform("import Alert from './Alert'\n\n<Alert title=\"Hi\" />\n");

    assert!(result.errors.is_empty(), "unexpected transform errors: {:?}", result.errors);
    assert!(!result.html.contains("import Alert"));
    assert!(result.html.contains("data-ox-island=\"Alert\""), "{}", result.html);
    assert!(result.html.contains("&quot;Hi&quot;"), "{}", result.html);
}

#[test]
fn leaves_non_frontmatter_documents_untouched() {
    let (content, frontmatter) = super::parse_frontmatter("# Hello");

    assert_eq!(content, "# Hello");
    assert!(frontmatter.is_empty());
}

#[test]
fn skips_frontmatter_extraction_when_disabled() {
    let source = "---\ntitle: Example\n---\n# Hello";
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        frontmatter: Some(false),
        ..Default::default()
    });
    let prepared = transformer.prepare_source(source);

    assert_eq!(prepared.content, source);
    assert!(prepared.frontmatter.is_empty());
}

#[test]
fn tracks_source_origin_after_frontmatter() {
    let prepared = parse_frontmatter_with_origin("---\ntitle: こんにちは\nemoji: 😀\n---\n# Hello");

    assert_eq!(prepared.content, "# Hello");
    assert_eq!(
        prepared.source_origin,
        SourceOrigin { byte_offset: 43, offset: 31, line: 5, column: 1 }
    );
}

#[test]
fn toc_slugs_are_unique_and_match_heading_ids() {
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "## Setup!\n## Setup?\n##\n## Node.js API via N-API\n## Detail tracing (`--detail`)",
    )
    .parse()
    .unwrap();

    let toc = extract_toc(&doc, 3);

    assert_eq!(toc[0].slug, "setup");
    assert_eq!(toc[1].slug, "setup-1");
    assert_eq!(toc[2].slug, "section");
    assert_eq!(toc[3].slug, "node-js-api-via-n-api");
    assert_eq!(toc[4].slug, "detail-tracing-detail");
}

#[test]
fn toc_entries_are_nested_in_rust() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "## Guide\n### Install\n#### CLI\n## API").parse().unwrap();

    let toc = extract_toc(&doc, 4);

    assert_eq!(toc.len(), 2);
    assert_eq!(toc[0].slug, "guide");
    assert_eq!(toc[0].children[0].slug, "install");
    assert_eq!(toc[0].children[0].children[0].slug, "cli");
    assert_eq!(toc[1].slug, "api");
}
