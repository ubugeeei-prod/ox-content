mod frontmatter;
mod options;
#[cfg(test)]
mod tests;
mod toc;

use ox_content_allocator::Allocator;
use ox_content_ast::Document;
use ox_content_mdast::{
    mdast_raw::{
        self, MDAST_SECTION_CONTENT, MDAST_SECTION_FRONTMATTER, MDAST_SECTION_SOURCE_ORIGIN,
    },
    transfer::{TransferBufferBuilder, TransferError, TransferPayloadKind},
};
use ox_content_parser::{ParseError, Parser, ParserOptions};
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

pub use frontmatter::{parse_frontmatter, PreparedMarkdownSource, SourceOrigin};

use frontmatter::{parse_frontmatter_with_origin, source_without_frontmatter};
use options::{transform_options_to_parser_options, transform_options_to_renderer_options};
use toc::extract_toc;

use crate::{
    features::{self, TransformFeatureOptions},
    SanitizeOptions, TransformOptions, TransformResult,
};

const PREPARED_SOURCE_PAYLOAD_VERSION: u32 = 1;
const PREPARED_SOURCE_SECTION_CONTENT: u32 = 1;
const PREPARED_SOURCE_SECTION_FRONTMATTER: u32 = 2;
const PREPARED_SOURCE_SECTION_SOURCE_ORIGIN: u32 = 3;

pub struct MarkdownTransformer {
    frontmatter: bool,
    toc_max_depth: u8,
    parser_options: ParserOptions,
    renderer_options: HtmlRendererOptions,
    feature_options: TransformFeatureOptions,
    sanitize_options: Option<SanitizeOptions>,
}

impl MarkdownTransformer {
    pub fn with_frontmatter(frontmatter: bool) -> Self {
        Self {
            frontmatter,
            toc_max_depth: 3,
            parser_options: ParserOptions::default(),
            renderer_options: HtmlRendererOptions::new(),
            feature_options: TransformFeatureOptions::default(),
            sanitize_options: None,
        }
    }

    pub fn from_options(options: &TransformOptions) -> Self {
        Self {
            frontmatter: options.frontmatter.unwrap_or(true),
            toc_max_depth: options.toc_max_depth.unwrap_or(3),
            parser_options: transform_options_to_parser_options(options),
            renderer_options: transform_options_to_renderer_options(options),
            feature_options: TransformFeatureOptions::from_options(options),
            sanitize_options: options.sanitize.clone(),
        }
    }

    pub fn transform(&self, source: &str) -> TransformResult {
        let prepared = self.prepare_source(source);
        let preprocessed = features::preprocess_markdown(&prepared.content, &self.feature_options);
        let content = preprocessed.source.as_ref();
        // Preprocessors may return borrowed or owned content. Size the arena
        // from the exact slice that will be parsed, not the original source, so
        // generated feature content and stripped frontmatter both get an arena
        // that matches their actual AST footprint.
        let allocator = Allocator::for_source_len(content.len());
        let parse_result = self.parse_document(&allocator, content);
        let mut errors = preprocessed.errors;

        match parse_result {
            Ok(document) => {
                let mut html = self.render_html(&document);
                if self.feature_options.has_postprocess() {
                    let postprocessed = features::postprocess_html(&html, &self.feature_options);
                    html = postprocessed.html;
                    errors.extend(postprocessed.errors);
                }
                if self.sanitize_options.is_some() {
                    html = crate::sanitize::sanitize_html(&html, self.sanitize_options.as_ref());
                }

                TransformResult {
                    html,
                    frontmatter: serde_json::to_string(&prepared.frontmatter)
                        .unwrap_or_else(|_| "{}".to_string()),
                    toc: extract_toc(&document, self.toc_max_depth),
                    errors,
                }
            }
            Err(error) => TransformResult {
                html: String::new(),
                frontmatter: "{}".to_string(),
                toc: vec![],
                errors: {
                    errors.push(error.to_string());
                    errors
                },
            },
        }
    }

    pub fn transform_mdast_raw(&self, source: &str) -> ox_content_mdast::transfer::Result<Vec<u8>> {
        let prepared = self.prepare_source(source);
        let preprocessed = features::preprocess_markdown(&prepared.content, &self.feature_options);
        let content = preprocessed.source.as_ref();
        let content_bytes = content.as_bytes().to_vec();
        let frontmatter_bytes = serde_json::to_vec(&prepared.frontmatter)
            .map_err(|error| TransferError::new(error.to_string()))?;
        let allocator = Allocator::for_source_len(content.len());
        let document = self
            .parse_document(&allocator, content)
            .map_err(|error| TransferError::new(error.to_string()))?;

        mdast_raw::to_mdast_raw_with_sections(
            &document,
            vec![
                (MDAST_SECTION_CONTENT, content_bytes),
                (MDAST_SECTION_FRONTMATTER, frontmatter_bytes),
                (MDAST_SECTION_SOURCE_ORIGIN, prepared.source_origin.to_bytes()),
            ],
        )
    }

    pub fn prepare_source_raw(&self, source: &str) -> ox_content_mdast::transfer::Result<Vec<u8>> {
        let prepared = self.prepare_source(source);
        let frontmatter_bytes = serde_json::to_vec(&prepared.frontmatter)
            .map_err(|error| TransferError::new(error.to_string()))?;
        let mut builder = TransferBufferBuilder::new(
            TransferPayloadKind::PreparedSource,
            PREPARED_SOURCE_PAYLOAD_VERSION,
            0,
        );
        builder.push_section(PREPARED_SOURCE_SECTION_CONTENT, prepared.content.into_bytes());
        builder.push_section(PREPARED_SOURCE_SECTION_FRONTMATTER, frontmatter_bytes);
        builder
            .push_section(PREPARED_SOURCE_SECTION_SOURCE_ORIGIN, prepared.source_origin.to_bytes());
        builder.finish()
    }

    pub fn parse_document<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<Document<'a>, ParseError> {
        Parser::with_options(allocator, source, self.parser_options.clone()).parse()
    }

    pub fn render_html(&self, document: &Document<'_>) -> String {
        let mut renderer = HtmlRenderer::with_options(self.renderer_options.clone());
        renderer.render(document)
    }

    pub fn prepare_source(&self, source: &str) -> PreparedMarkdownSource {
        if self.frontmatter {
            parse_frontmatter_with_origin(source)
        } else {
            // Frontmatter-disabled mode keeps a simple owned copy because this
            // value may cross NAPI as a prepared-source payload; the parse hot
            // path borrows from this string rather than reparsing YAML.
            source_without_frontmatter(source)
        }
    }
}
