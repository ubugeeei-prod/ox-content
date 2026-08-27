/// What a cross-reference target is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossReferenceKind {
    Figure,
    Table,
    Section,
}

impl CrossReferenceKind {
    /// The slug used in `data-ox-xref-kind` and the `ox-xref-*` class.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Figure => "figure",
            Self::Table => "table",
            Self::Section => "section",
        }
    }
}

/// What to do when a document breaks one of the cross-reference rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    /// Fail the build.
    Error,
    /// Report and carry on.
    Warn,
}

/// The word placed before each number, per kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossReferenceLabels {
    pub figure: String,
    pub table: String,
    pub section: String,
}

impl Default for CrossReferenceLabels {
    fn default() -> Self {
        Self {
            figure: "Figure".to_string(),
            table: "Table".to_string(),
            section: "Section".to_string(),
        }
    }
}

/// Switches and labels for the cross-reference pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossReferencesOptions {
    pub enabled: bool,
    /// A `@ref` with no target.
    pub missing: FailureMode,
    /// Two targets claiming the same id.
    pub duplicates: FailureMode,
    /// A `@fig-x` that resolves to a table, and the like.
    pub mismatches: FailureMode,
    pub labels: CrossReferenceLabels,
}

impl Default for CrossReferencesOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            missing: FailureMode::Error,
            duplicates: FailureMode::Error,
            mismatches: FailureMode::Error,
            labels: CrossReferenceLabels::default(),
        }
    }
}

/// One numbered target the document defines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossReferenceEntry {
    pub id: String,
    pub kind: CrossReferenceKind,
    /// `1`, or `2.3` for a nested section.
    pub number: String,
    /// The label word on its own, for callers that re-compose the text.
    pub label: String,
    /// Label and number together, as written into the link.
    pub text: String,
    pub href: String,
    /// Heading text, figure caption, or image `alt`, when there is one.
    pub title: Option<String>,
}

/// One rule a document broke, and how the caller asked to be told.
///
/// Rust returns these rather than raising: the plugin decides which are fatal,
/// and the panic-construct gate keeps this crate from deciding for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossReferenceDiagnostic {
    pub policy: FailureMode,
    pub message: String,
}

/// The annotated HTML, what it defined, and what it got wrong.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrossReferenceOutput {
    pub html: String,
    pub references: Vec<CrossReferenceEntry>,
    pub diagnostics: Vec<CrossReferenceDiagnostic>,
}
