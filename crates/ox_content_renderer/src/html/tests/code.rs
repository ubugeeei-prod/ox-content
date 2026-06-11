use crate::html::{CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

#[test]
fn test_render_code_block() {
    let allocator = Allocator::new();
    let doc = Parser::new(&allocator, "```rust\nfn main() {}\n```").parse().unwrap();
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&doc);
    assert!(html.contains("<pre><code class=\"language-rust\">"));
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

    assert!(html.contains("class=\"ox-code-block ox-code-block--annotated has-highlighted\""));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--highlight highlighted\" data-line=\"1\""
    ));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--warning highlighted warning\" data-line=\"2\""
    ));
    assert!(html.contains(
        "class=\"line ox-code-line ox-code-line--error highlighted error\" data-line=\"3\""
    ));
    assert!(!html.contains("file=main.ts"));
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

    assert!(html.contains("ox-code-block--annotated"));
    assert!(html.contains("ox-code-line--warning"));
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

    assert!(html.contains("ox-code-block--annotated"));
    assert!(html.contains("ox-code-block--line-numbers"));
    assert!(html.contains("ox-code-block--with-title"));
    assert!(html.contains("line-numbers-mode"));
    assert!(html.contains("has-highlighted"));
    assert!(html.contains("data-code-title=\"config.ts\""));
    assert!(html.contains("data-line-number-start=\"2\""));
    assert!(html.contains("class=\"language-ts\""));
    assert!(html.contains("data-line-number=\"2\""));
    assert!(html.contains("data-line-number=\"4\""));
    assert!(html.contains("ox-code-line--highlight"));
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

    assert!(!html.contains("[!code"));
    assert!(html.contains("has-focused"));
    assert!(html.contains("has-diff"));
    assert!(html.contains("ox-code-line--focus"));
    assert!(html.contains("ox-code-line--dimmed"));
    assert!(html.contains("ox-code-line--remove"));
    assert!(html.contains("ox-code-line--add"));
    assert!(html.contains("ox-code-line--warning"));
    assert!(html.contains("ox-code-line--error"));
    assert!(html.contains("console.log(&#39;old value&#39;)"));
    assert!(html.contains("console.log(&#39;new value&#39;)"));
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

    assert!(!html.contains("[!code escape]"));
    assert!(html.contains("console.warn(&#39;literal&#39;) // [!code warning]"));
    assert!(html.contains("console.warn(&#39;annotated&#39;)"));
    assert_eq!(html.matches("ox-code-line--warning").count(), 1);
}
