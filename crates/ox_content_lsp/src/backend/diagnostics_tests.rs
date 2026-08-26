use super::*;
use crate::frontmatter::parse_frontmatter;
use crate::session::DiagnosticJob;
use ox_content_link_checker::{CODE_MISSING_FILE, CheckOptions, check_source};
use ox_content_mdc_checker::{CODE_UNQUOTED_PROP, check_document};

#[test]
fn markdown_parse_diagnostics_skip_valid_body_after_frontmatter() {
    let source = "---\ntitle: Doc\n---\n\n# Valid heading\n\nA paragraph.\n";
    let document = TextDocumentState::new(source.to_string());
    let frontmatter = parse_frontmatter(&document);
    let diagnostics = markdown_parse_diagnostics(&document, frontmatter.block.as_ref(), false);
    assert!(diagnostics.is_empty(), "expected clean parse for valid body, got {diagnostics:?}");
}

#[test]
fn markdown_parse_diagnostics_skip_yaml_inside_frontmatter() {
    let source = "---\ntitle: A title with : colon\n---\n\nbody\n";
    let document = TextDocumentState::new(source.to_string());
    let frontmatter = parse_frontmatter(&document);
    let diagnostics = markdown_parse_diagnostics(&document, frontmatter.block.as_ref(), false);
    assert!(diagnostics.is_empty(), "yaml leaked into markdown diagnostics: {diagnostics:?}");
}

#[test]
fn markdown_parse_diagnostics_accept_mdx_when_enabled() {
    let source = "import Card from './Card'\n\n<Card count={1}>Visible</Card>\n";
    let document = TextDocumentState::new(source.to_string());
    let diagnostics = markdown_parse_diagnostics(&document, None, true);
    assert!(diagnostics.is_empty(), "expected clean MDX parse, got {diagnostics:?}");
}

#[test]
fn cancelled_collect_returns_none() {
    let document = TextDocumentState::with_version("# hi\n".into(), 1);
    let uri = Url::parse("file:///tmp/cancel.md").expect("uri");
    let job = DiagnosticJob::new(1);
    job.cancel();
    let result = collect_markdown_diagnostics(
        &uri,
        &document,
        &ResolvedConfig::default(),
        None,
        &DiagnosticCache::default(),
        &job,
    );
    assert!(result.is_none());
}

#[test]
fn cli_and_lsp_agree_on_link_fixture() {
    let dir = std::env::temp_dir().join("ox-content-shared-link-854");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("fixture dir");
    let path = dir.join("page.md");
    let source = "---\ntitle: Shared\n---\n\nSee [missing](./does-not-exist.md).\n";
    std::fs::write(&path, source).expect("write fixture");
    let cli = check_source(source, &CheckOptions::for_file(path.clone()));
    let uri = Url::from_file_path(&path).expect("file url");
    let document = TextDocumentState::new(source.to_string());
    let lsp = link_check_diagnostics(&document, &uri, None);
    assert_eq!(cli.len(), 1, "{cli:?}");
    assert_eq!(lsp.len(), 1, "{lsp:?}");
    assert_eq!(cli[0].code, CODE_MISSING_FILE);
    assert_eq!(lsp[0].code, Some(NumberOrString::String(CODE_MISSING_FILE.to_string())));
    assert_eq!(cli[0].message, lsp[0].message);
    assert_eq!(cli[0].line.saturating_sub(1), lsp[0].range.start.line);
    assert_eq!(cli[0].column.saturating_sub(1), lsp[0].range.start.character);
}

#[test]
fn cli_and_lsp_agree_on_mdc_frontmatter_fixture() {
    let source = "---\ntitle: Shared\n---\n\n<Alert tone=info></Alert>\n";
    let cli = check_document(source);
    let document = TextDocumentState::new(source.to_string());
    let lsp = mdc_diagnostics(&document);
    assert_eq!(cli.len(), 1, "{cli:?}");
    assert_eq!(lsp.len(), 1, "{lsp:?}");
    assert_eq!(cli[0].code, CODE_UNQUOTED_PROP);
    assert_eq!(lsp[0].code, Some(NumberOrString::String(CODE_UNQUOTED_PROP.to_string())));
    assert_eq!(cli[0].message, lsp[0].message);
    assert_eq!(cli[0].line.saturating_sub(1), lsp[0].range.start.line);
    assert_eq!(cli[0].column.saturating_sub(1), lsp[0].range.start.character);
}
