use super::head::HeadDiagnostic;
use super::page::{NavGroup, PageData, SsgConfig};
use super::render_inner::generate_html_inner;

/// Themed HTML plus page-head diagnostics.
pub struct GeneratedHtml {
    pub html: String,
    pub diagnostics: Vec<HeadDiagnostic>,
}

/// Generates a complete HTML page for SSG.
///
/// This function creates a full HTML document with navigation sidebar,
/// content area, table of contents, search functionality, and theme toggle.
pub fn generate_html(page_data: &PageData, nav_groups: &[NavGroup], config: &SsgConfig) -> String {
    generate_html_result(page_data, nav_groups, config).html
}

/// Same as [`generate_html`], with head-validation findings.
pub fn generate_html_result(
    page_data: &PageData,
    nav_groups: &[NavGroup],
    config: &SsgConfig,
) -> GeneratedHtml {
    generate_html_result_with_auto_navigation(page_data, nav_groups, config, false)
}

/// Generates themed HTML and scopes large inferred sidebars when requested.
pub fn generate_html_result_with_auto_navigation(
    page_data: &PageData,
    nav_groups: &[NavGroup],
    config: &SsgConfig,
    auto_navigation: bool,
) -> GeneratedHtml {
    let generated = generate_html_inner(page_data, nav_groups, config, auto_navigation);
    GeneratedHtml { html: generated.html, diagnostics: generated.head.diagnostics }
}
