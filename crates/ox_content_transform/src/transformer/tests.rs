use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

use super::{
    frontmatter::{parse_frontmatter_with_origin, SourceOrigin},
    toc::extract_toc,
    MarkdownTransformer,
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
    assert!(result.html.contains("<h1 id=\"hello\">Hello</h1>"));
    assert!(result.frontmatter.contains("\"title\":\"Example\""));
    assert_eq!(result.toc.len(), 1);
    assert_eq!(result.toc[0].slug, "hello");
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
    let doc = Parser::new(&allocator, "## Setup!\n## Setup?\n##").parse().unwrap();

    let toc = extract_toc(&doc, 3);

    assert_eq!(toc[0].slug, "setup");
    assert_eq!(toc[1].slug, "setup-1");
    assert_eq!(toc[2].slug, "section");
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
