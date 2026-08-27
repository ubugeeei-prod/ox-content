use napi_derive::napi;
use ox_content_transform::cross_references::{
    CrossReferenceKind, CrossReferenceLabels, CrossReferencesOptions, FailureMode,
    transform_cross_references,
};

/// The word placed before each number, per kind.
#[napi(object)]
pub struct JsCrossReferenceLabels {
    pub figure: Option<String>,
    pub table: Option<String>,
    pub section: Option<String>,
}

/// Switches and labels for the cross-reference pass.
#[napi(object)]
pub struct JsCrossReferencesOptions {
    pub enabled: bool,
    /// `"warn"` reports and carries on; anything else fails the build.
    pub missing: Option<String>,
    pub duplicates: Option<String>,
    pub mismatches: Option<String>,
    pub labels: Option<JsCrossReferenceLabels>,
}

/// One numbered target the document defines.
#[napi(object)]
pub struct JsCrossReferenceEntry {
    pub id: String,
    /// `"figure"`, `"table"`, or `"section"`.
    pub kind: String,
    pub number: String,
    pub label: String,
    pub text: String,
    pub href: String,
    pub title: Option<String>,
}

/// One rule the document broke.
#[napi(object)]
pub struct JsCrossReferenceDiagnostic {
    /// `"error"` or `"warn"`, as the options asked.
    pub policy: String,
    pub message: String,
}

/// The annotated HTML, what it defined, and what it got wrong.
#[napi(object)]
pub struct JsCrossReferenceOutput {
    pub html: String,
    pub references: Vec<JsCrossReferenceEntry>,
    pub diagnostics: Vec<JsCrossReferenceDiagnostic>,
}

/// Numbers headings, figures, and tables, and links `@id` references to them.
#[napi(js_name = "transformCrossReferences")]
pub fn transform_cross_references_binding(
    html: String,
    options: JsCrossReferencesOptions,
) -> JsCrossReferenceOutput {
    let labels = options.labels.unwrap_or(JsCrossReferenceLabels {
        figure: None,
        table: None,
        section: None,
    });
    let defaults = CrossReferenceLabels::default();
    let resolved = CrossReferencesOptions {
        enabled: options.enabled,
        missing: failure_mode(options.missing.as_deref()),
        duplicates: failure_mode(options.duplicates.as_deref()),
        mismatches: failure_mode(options.mismatches.as_deref()),
        labels: CrossReferenceLabels {
            figure: labels.figure.unwrap_or(defaults.figure),
            table: labels.table.unwrap_or(defaults.table),
            section: labels.section.unwrap_or(defaults.section),
        },
    };

    let output = transform_cross_references(&html, &resolved);
    JsCrossReferenceOutput {
        html: output.html,
        references: output
            .references
            .into_iter()
            .map(|entry| JsCrossReferenceEntry {
                id: entry.id,
                kind: kind_name(entry.kind),
                number: entry.number,
                label: entry.label,
                text: entry.text,
                href: entry.href,
                title: entry.title,
            })
            .collect(),
        diagnostics: output
            .diagnostics
            .into_iter()
            .map(|diagnostic| JsCrossReferenceDiagnostic {
                policy: match diagnostic.policy {
                    FailureMode::Warn => "warn".to_string(),
                    FailureMode::Error => "error".to_string(),
                },
                message: diagnostic.message,
            })
            .collect(),
    }
}

/// Only `"warn"` softens a rule; every other value keeps it fatal.
fn failure_mode(value: Option<&str>) -> FailureMode {
    if value == Some("warn") { FailureMode::Warn } else { FailureMode::Error }
}

fn kind_name(kind: CrossReferenceKind) -> String {
    kind.as_str().to_string()
}
