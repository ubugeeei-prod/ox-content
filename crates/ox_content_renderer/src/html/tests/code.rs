use crate::html::{CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_code_block() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "```rust\nfn main() {}\n```").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    insta::assert_snapshot!(html);
}

#[test]
fn test_render_code_block_with_annotations() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts file=main.ts annotate=\"highlight:1;warning:2;error:3\"\nconst ok = true;\nconst maybe = false;\nthrow new Error('boom');\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_code_block_with_custom_annotation_meta_key() {
    let allocator = Allocator::new();
    let doc = Parser::new(
        &allocator,
        "```ts markers=\"warning:2\"\nconst ok = true;\nconst maybe = false;\n```",
    )
    .parse()
    .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_meta_key: "markers".to_string(),
        ..Default::default()
    });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_code_block_with_vitepress_meta() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts:line-numbers=2 {1,3} [config.ts]\nconst first = true;\nconst second = false;\nconst third = true;\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_code_block_with_vitepress_inline_directives() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts\n// [!code focus:2]\nconst first = true;\nconst second = false;\nconsole.log('old value') // [!code --]\nconsole.log('new value') // [!code ++]\nconsole.warn('careful') // [!code warning]\nthrow new Error('boom') // [!code error]\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    insta::assert_snapshot!(html);
}

#[test]
fn test_render_code_block_with_vitepress_escape_next_line() {
    let allocator = Allocator::new();
    let doc = Parser::new(
            &allocator,
            "```ts\n// [!code escape]\nconsole.warn('literal') // [!code warning]\nconsole.warn('annotated') // [!code warning]\n```",
        )
        .parse()
        .unwrap();
    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
        code_annotations: true,
        code_annotation_syntax: CodeAnnotationSyntax::VitePress,
        ..Default::default()
    });
    let html = renderer.render(&doc);

    assert_eq!(html.matches("ox-code-line--warning").count(), 1);
    insta::assert_snapshot!(html);
}
