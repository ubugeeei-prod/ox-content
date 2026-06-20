use ox_content_parser::ParserOptions;
use ox_content_renderer::{CodeAnnotationSyntax, HtmlRendererOptions};

use crate::TransformOptions;

pub(super) fn transform_options_to_parser_options(opts: &TransformOptions) -> ParserOptions {
    let mut options =
        if opts.gfm.unwrap_or(false) { ParserOptions::gfm() } else { ParserOptions::default() };

    if let Some(v) = opts.footnotes {
        options.footnotes = v;
    }
    if let Some(v) = opts.task_lists {
        options.task_lists = v;
    }
    if let Some(v) = opts.tables {
        options.tables = v;
    }
    if let Some(v) = opts.strikethrough {
        options.strikethrough = v;
    }
    if let Some(v) = opts.autolinks {
        options.autolinks = v;
    }

    options
}

pub(super) fn transform_options_to_renderer_options(
    opts: &TransformOptions,
) -> HtmlRendererOptions {
    let mut options = HtmlRendererOptions::new();

    options.toc_max_depth = opts.toc_max_depth.unwrap_or(options.toc_max_depth);

    if let Some(v) = opts.convert_md_links {
        options.convert_md_links = v;
    }
    if let Some(v) = &opts.base_url {
        options.base_url.clone_from(v);
    }
    if let Some(v) = &opts.source_path {
        options.source_path.clone_from(v);
    }
    if let Some(v) = opts.code_annotations {
        options.code_annotations = v;
    }
    if let Some(v) = &opts.code_annotation_meta_key {
        options.code_annotation_meta_key.clone_from(v);
    }
    if let Some(v) = &opts.code_annotation_syntax {
        options.code_annotation_syntax = match v.as_str() {
            "vitepress" => CodeAnnotationSyntax::VitePress,
            "both" => CodeAnnotationSyntax::Both,
            _ => CodeAnnotationSyntax::Attribute,
        };
    }
    if let Some(v) = opts.code_annotation_default_line_numbers {
        options.code_annotation_default_line_numbers = v;
    }
    if let Some(v) = opts.autolink_urls {
        options.autolink_urls = v;
    }
    if let Some(v) = &opts.autolink_patterns {
        options.autolink_patterns.clone_from(v);
    }
    if let Some(v) = opts.autolink_target_blank {
        options.autolink_target_blank = v;
    }

    options
}
