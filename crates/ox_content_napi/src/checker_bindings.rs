use std::fs;
use std::path::PathBuf;

use napi_derive::napi;

#[napi(object)]
#[derive(Default)]
pub struct JsLinkCheckOptions {
    pub src_dir: Option<String>,
    pub public_dir: Option<String>,
    pub site_dir: Option<String>,
    pub base: Option<String>,
    pub ignore: Option<Vec<String>>,
}

#[napi(object)]
pub struct JsLinkDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub kind: String,
    pub target: String,
}

#[napi(object)]
pub struct JsLinkFileReport {
    pub file: String,
    pub diagnostics: Vec<JsLinkDiagnostic>,
}

#[napi(object)]
pub struct JsLinkCheckResult {
    pub reports: Vec<JsLinkFileReport>,
    pub error_count: u32,
    pub warning_count: u32,
}

#[napi(object)]
pub struct JsMdcDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub component: Option<String>,
}

#[napi(object)]
pub struct JsMdcFileReport {
    pub file: String,
    pub diagnostics: Vec<JsMdcDiagnostic>,
}

#[napi(object)]
pub struct JsMdcCheckResult {
    pub reports: Vec<JsMdcFileReport>,
    pub error_count: u32,
}

/// Checks Markdown/MDC local links from Node.js.
#[napi(js_name = "checkLinks")]
pub fn check_links(files: Vec<String>, options: Option<JsLinkCheckOptions>) -> JsLinkCheckResult {
    let options = options.unwrap_or_default();
    let mut reports = Vec::new();
    let mut error_count = 0u32;
    let mut warning_count = 0u32;

    for file in files {
        let file_path = PathBuf::from(&file);
        let diagnostics = match fs::read_to_string(&file_path) {
            Ok(source) => ox_content_link_checker::check_source(
                &source,
                &ox_content_link_checker::CheckOptions {
                    file_path,
                    src_dir: options.src_dir.clone().map(PathBuf::from),
                    public_dir: options.public_dir.clone().map(PathBuf::from),
                    ignore_patterns: options.ignore.clone().unwrap_or_default(),
                },
            ),
            Err(error) => vec![ox_content_link_checker::Diagnostic {
                severity: ox_content_link_checker::Severity::Error,
                code: ox_content_link_checker::CODE_IO_READ.to_string(),
                message: format!("Failed to read file: {error}"),
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
                kind: ox_content_link_checker::LinkKind::Unknown,
                target: String::new(),
            }],
        };

        collect_link_counts(&diagnostics, &mut error_count, &mut warning_count);
        reports.push(JsLinkFileReport {
            file,
            diagnostics: diagnostics.into_iter().map(map_link_diagnostic).collect(),
        });
    }

    if let Some(site_dir) = options.site_dir {
        let site_options = ox_content_link_checker::SiteCheckOptions {
            site_dir: PathBuf::from(&site_dir),
            base: options.base.unwrap_or_else(|| "/".to_string()),
        };
        match ox_content_link_checker::check_site(&site_options) {
            Ok(site_reports) => {
                for report in site_reports {
                    collect_link_counts(&report.diagnostics, &mut error_count, &mut warning_count);
                    reports.push(JsLinkFileReport {
                        file: report.file_path.to_string_lossy().into_owned(),
                        diagnostics: report
                            .diagnostics
                            .into_iter()
                            .map(map_link_diagnostic)
                            .collect(),
                    });
                }
            }
            Err(error) => {
                error_count += 1;
                reports.push(JsLinkFileReport {
                    file: site_dir,
                    diagnostics: vec![JsLinkDiagnostic {
                        severity: "error".to_string(),
                        code: ox_content_link_checker::CODE_IO_READ.to_string(),
                        message: format!("Failed to inspect generated site: {error}"),
                        line: 1,
                        column: 1,
                        end_line: 1,
                        end_column: 1,
                        kind: "unknown".to_string(),
                        target: String::new(),
                    }],
                });
            }
        }
    }

    JsLinkCheckResult { reports, error_count, warning_count }
}

/// Checks MDC component syntax from Node.js.
#[napi(js_name = "checkMdc")]
pub fn check_mdc(files: Vec<String>) -> JsMdcCheckResult {
    let mut reports = Vec::new();
    let mut error_count = 0u32;

    for file in files {
        let diagnostics = match fs::read_to_string(&file) {
            Ok(source) => ox_content_mdc_checker::check_document(&source),
            Err(error) => vec![ox_content_mdc_checker::Diagnostic {
                severity: ox_content_mdc_checker::Severity::Error,
                code: ox_content_mdc_checker::CODE_IO_READ.to_string(),
                message: error.to_string(),
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
                component: None,
            }],
        };

        error_count += diagnostics.len() as u32;
        reports.push(JsMdcFileReport {
            file,
            diagnostics: diagnostics.into_iter().map(map_mdc_diagnostic).collect(),
        });
    }

    JsMdcCheckResult { reports, error_count }
}

fn collect_link_counts(
    diagnostics: &[ox_content_link_checker::Diagnostic],
    error_count: &mut u32,
    warning_count: &mut u32,
) {
    for diagnostic in diagnostics {
        match diagnostic.severity {
            ox_content_link_checker::Severity::Error => *error_count += 1,
            ox_content_link_checker::Severity::Warning => *warning_count += 1,
        }
    }
}

fn map_link_diagnostic(diagnostic: ox_content_link_checker::Diagnostic) -> JsLinkDiagnostic {
    JsLinkDiagnostic {
        severity: match diagnostic.severity {
            ox_content_link_checker::Severity::Error => "error",
            ox_content_link_checker::Severity::Warning => "warning",
        }
        .to_string(),
        code: diagnostic.code,
        message: diagnostic.message,
        line: diagnostic.line,
        column: diagnostic.column,
        end_line: diagnostic.end_line,
        end_column: diagnostic.end_column,
        kind: map_link_kind(diagnostic.kind).to_string(),
        target: diagnostic.target,
    }
}

fn map_link_kind(kind: ox_content_link_checker::LinkKind) -> &'static str {
    match kind {
        ox_content_link_checker::LinkKind::File => "file",
        ox_content_link_checker::LinkKind::Anchor => "anchor",
        ox_content_link_checker::LinkKind::FileAnchor => "file-anchor",
        ox_content_link_checker::LinkKind::External => "external",
        ox_content_link_checker::LinkKind::Scheme => "scheme",
        ox_content_link_checker::LinkKind::Unknown => "unknown",
    }
}

fn map_mdc_diagnostic(diagnostic: ox_content_mdc_checker::Diagnostic) -> JsMdcDiagnostic {
    JsMdcDiagnostic {
        severity: "error".to_string(),
        code: diagnostic.code,
        message: diagnostic.message,
        line: diagnostic.line,
        column: diagnostic.column,
        end_line: diagnostic.end_line,
        end_column: diagnostic.end_column,
        component: diagnostic.component,
    }
}
