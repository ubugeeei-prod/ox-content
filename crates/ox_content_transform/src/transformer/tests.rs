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
fn semantic_footnotes_option_reaches_the_renderer() {
    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        footnotes: Some(true),
        semantic_footnotes: Some(true),
        ..Default::default()
    });
    let result = transformer.transform("A[^note].\n\n[^note]: Body.\n");

    assert!(result.errors.is_empty(), "unexpected transform errors: {:?}", result.errors);
    assert!(
        result.html.contains("<section class=\"footnotes\" aria-label=\"Footnotes\">"),
        "{}",
        result.html
    );
    assert!(
        result.html.contains("<sup><a href=\"#fn-note\" id=\"fnref-note\">1</a></sup>"),
        "{}",
        result.html
    );
    assert!(!result.html.contains("class=\"footnote\""), "{}", result.html);
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
    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].source, "./Alert");
    assert_eq!(result.components, vec!["Alert"]);
    assert!(result.exports.is_empty());
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
fn heading_permalinks_default_off_keeps_html() {
    let off = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        ..Default::default()
    })
    .transform("# Hello");
    let on = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        heading_permalinks: Some(true),
        ..Default::default()
    })
    .transform("# Hello");

    assert_eq!(off.html, "<h1 id=\"hello\">Hello</h1>\n");
    assert!(on.html.contains("class=\"header-anchor\""), "{}", on.html);
    assert!(on.html.contains("href=\"#hello\""), "{}", on.html);
    assert_eq!(on.toc[0].slug, "hello");
}

#[test]
fn heading_permalinks_compose_with_explicit_ids() {
    let html = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        heading_permalinks: Some(true),
        attributes: Some(crate::AttrsOptions { enabled: Some(true) }),
        ..Default::default()
    })
    .transform("## Title {#custom-id}")
    .html;

    assert!(html.contains("id=\"custom-id\""), "{html}");
    assert!(!html.contains("id=\"title-custom-id\""), "{html}");
    assert!(html.contains("class=\"header-anchor\""), "{html}");
    assert!(html.contains("href=\"#custom-id\""), "{html}");
    assert!(html.contains("Permalink to &quot;Title&quot;"), "{html}");
    assert!(!html.contains("{#custom-id}"), "{html}");
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

#[test]
fn hostile_user_content_returns_errors_or_html_without_aborting() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let transformer = MarkdownTransformer::from_options(&TransformOptions {
        gfm: Some(true),
        frontmatter: Some(true),
        ..Default::default()
    });
    let cases = [
        "---\ntitle: [broken\n---\nBody",
        "---\n{not: yaml\n---\n# Hi",
        &"> ".repeat(120),
        "*[unterminated",
        "```\nunclosed fence",
        "<!-- @include: ../../etc/passwd -->\n",
    ];

    for source in cases {
        let outcome = catch_unwind(AssertUnwindSafe(|| transformer.transform(source)));
        let result = outcome.unwrap_or_else(|_| panic!("transform aborted on {source:?}"));
        assert!(
            result.errors.iter().all(|error| !error.contains("panic")),
            "unexpected panic diagnostic for {source:?}: {:?}",
            result.errors
        );
    }
}
