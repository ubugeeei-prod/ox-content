use std::path::PathBuf;

use ox_content_allocator::Allocator;
use ox_content_link_checker::{CheckOptions, Severity, check_source as link_check_source};
use ox_content_parser::{ParseError, Parser, ParserOptions};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Url};

use crate::config::ResolvedConfig;
use crate::document::TextDocumentState;
use crate::frontmatter::{self, FrontmatterBlock};
use crate::session::{DiagnosticCache, DiagnosticJob};

use super::Backend;

pub const CODE_MARKDOWN_PARSE: &str = "markdown-parse";
pub const CODE_FRONTMATTER_SCHEMA: &str = "frontmatter-schema-missing";

pub(super) fn markdown_parse_diagnostics(
    document: &TextDocumentState,
    block: Option<&FrontmatterBlock>,
    mdx: bool,
) -> Vec<Diagnostic> {
    let (source, offset) = block.map_or_else(
        || (document.text(), 0),
        |block| (&document.text()[block.block_end_offset..], block.block_end_offset),
    );

    let allocator = Allocator::for_source_len(source.len());
    let mut options = ParserOptions::gfm();
    options.mdx = mdx;
    let parser = Parser::with_options(&allocator, source, options);

    match parser.parse() {
        Ok(_) => Vec::new(),
        Err(error) => vec![parse_error_to_diagnostic(document, offset, error)],
    }
}

pub(super) fn mdc_diagnostics(document: &TextDocumentState) -> Vec<Diagnostic> {
    ox_content_mdc_checker::check_document(document.text())
        .into_iter()
        .map(|diagnostic| Diagnostic {
            range: checker_range(
                diagnostic.line,
                diagnostic.column,
                diagnostic.end_line,
                diagnostic.end_column,
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(diagnostic.code)),
            source: Some("ox-content-mdc".to_string()),
            message: diagnostic.message,
            ..Default::default()
        })
        .collect()
}

pub(super) fn link_check_diagnostics(
    document: &TextDocumentState,
    uri: &Url,
    src_dir: Option<PathBuf>,
) -> Vec<Diagnostic> {
    let Some(file_path) = uri.to_file_path().ok() else {
        return Vec::new();
    };

    let options =
        CheckOptions { file_path, src_dir, public_dir: None, ignore_patterns: Vec::new() };
    link_check_source(document.text(), &options)
        .into_iter()
        .map(|diagnostic| Diagnostic {
            range: checker_range(
                diagnostic.line,
                diagnostic.column,
                diagnostic.end_line,
                diagnostic.end_column,
            ),
            severity: Some(match diagnostic.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
            }),
            code: Some(NumberOrString::String(diagnostic.code)),
            source: Some("ox-content-link".to_string()),
            message: diagnostic.message,
            ..Default::default()
        })
        .collect()
}

fn checker_range(
    line: u32,
    column: u32,
    end_line: u32,
    end_column: u32,
) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range {
        start: tower_lsp::lsp_types::Position {
            line: line.saturating_sub(1),
            character: column.saturating_sub(1),
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line.saturating_sub(1),
            character: end_column.saturating_sub(1),
        },
    }
}

fn parse_error_to_diagnostic(
    document: &TextDocumentState,
    base_offset: usize,
    error: ParseError,
) -> Diagnostic {
    let span = error.span();
    let start = base_offset + span.start as usize;
    let end = (base_offset + span.end as usize).max(start + 1).min(document.text().len());

    Diagnostic {
        range: document.range_from_offsets(start, end),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(CODE_MARKDOWN_PARSE.to_string())),
        source: Some("ox-content".to_string()),
        message: error.to_string(),
        ..Default::default()
    }
}

impl Backend {
    pub(super) async fn publish_diagnostics_for(&self, uri: &Url) {
        let Some((document, job, cache)) = self.state.begin_diagnostics(uri).await else {
            return;
        };
        let Some((diagnostics, next_cache)) =
            self.collect_diagnostics(uri, &document, &cache, &job).await
        else {
            return;
        };
        if !self.state.finish_diagnostics(uri, &job, next_cache).await {
            return;
        }
        self.client.publish_diagnostics(uri.clone(), diagnostics, Some(job.version)).await;
    }

    pub(super) async fn run_textlint_for(&self, uri: &Url) {
        let config = self.resolved_config().await;
        if !config.textlint.enabled {
            return;
        }
        let Some((document, job, cache)) = self.state.begin_diagnostics(uri).await else {
            return;
        };
        let Ok(path) = uri.to_file_path() else {
            return;
        };
        if !crate::document::is_markdown_path(&path) {
            return;
        }
        let textlint_diagnostics =
            crate::textlint::run(document.text(), &path, &config.textlint).await;
        if job.is_cancelled() {
            return;
        }
        let Some((mut current, next_cache)) =
            self.collect_diagnostics(uri, &document, &cache, &job).await
        else {
            return;
        };
        current.extend(textlint_diagnostics);
        if !self.state.finish_diagnostics(uri, &job, next_cache).await {
            return;
        }
        self.client.publish_diagnostics(uri.clone(), current, Some(job.version)).await;
    }

    async fn collect_diagnostics(
        &self,
        uri: &Url,
        document: &TextDocumentState,
        cache: &DiagnosticCache,
        job: &DiagnosticJob,
    ) -> Option<(Vec<Diagnostic>, DiagnosticCache)> {
        if job.is_cancelled() {
            return None;
        }
        let config = self.resolved_config().await;
        let root = self.state.root().await;
        if job.is_cancelled() {
            return None;
        }
        collect_markdown_diagnostics(uri, document, &config, root, cache, job)
    }
}

pub(super) fn collect_markdown_diagnostics(
    uri: &Url,
    document: &TextDocumentState,
    config: &ResolvedConfig,
    src_dir: Option<PathBuf>,
    cache: &DiagnosticCache,
    job: &DiagnosticJob,
) -> Option<(Vec<Diagnostic>, DiagnosticCache)> {
    let frontmatter = frontmatter::parse_frontmatter(document);
    let frontmatter_src = frontmatter
        .block
        .as_ref()
        .map_or(String::new(), |block| document.text()[..block.block_end_offset].to_string());
    let body_src = document.text()[frontmatter_src.len()..].to_string();
    let (reuse_frontmatter, reuse_body) = cache.reuse(&frontmatter_src, &body_src);

    let mut next = DiagnosticCache { frontmatter_src, body_src, ..DiagnosticCache::default() };
    if reuse_frontmatter {
        next.frontmatter.clone_from(&cache.frontmatter);
    } else {
        if job.is_cancelled() {
            return None;
        }
        next.frontmatter = frontmatter
            .block
            .as_ref()
            .map_or_else(Vec::new, |block| frontmatter_diagnostics(block, config));
    }

    if reuse_body {
        next.parse.clone_from(&cache.parse);
        next.mdc.clone_from(&cache.mdc);
        next.links.clone_from(&cache.links);
        next.spacing.clone_from(&cache.spacing);
    } else {
        if job.is_cancelled() {
            return None;
        }
        let is_mdc = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.extension().and_then(|ext| ext.to_str()).map(str::to_string))
            .is_some_and(|ext| ext == "mdc");
        let mdx = uri.to_file_path().is_ok_and(|path| crate::document::is_mdx_path(&path));
        next.parse = markdown_parse_diagnostics(document, frontmatter.block.as_ref(), mdx);
        if is_mdc {
            next.mdc = mdc_diagnostics(document);
        }
        next.spacing =
            crate::spacing::diagnostics(document, frontmatter.block.as_ref(), config.spacing);
        next.links = link_check_diagnostics(document, uri, src_dir);
    }

    if job.is_cancelled() {
        return None;
    }

    let mut diagnostics = next.frontmatter.clone();
    diagnostics.extend(next.parse.iter().cloned());
    diagnostics.extend(next.mdc.iter().cloned());
    diagnostics.extend(next.spacing.iter().cloned());
    diagnostics.extend(next.links.iter().cloned());
    Some((diagnostics, next))
}

pub(super) fn frontmatter_diagnostics(
    block: &frontmatter::FrontmatterBlock,
    config: &ResolvedConfig,
) -> Vec<Diagnostic> {
    let mut diagnostics = block.diagnostics.clone();
    match Backend::load_schema(config) {
        Ok(Some(schema)) => {
            diagnostics.extend(frontmatter::validate_frontmatter(block, &schema));
        }
        Ok(None) => {}
        Err(message) => diagnostics.push(Diagnostic {
            range: block.block_range,
            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(CODE_FRONTMATTER_SCHEMA.to_string())),
            source: Some("ox-content".to_string()),
            message,
            ..Default::default()
        }),
    }
    diagnostics
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
