//! Shared code-block annotation state.
//!
//! These types are the compact hand-off format between metadata parsers and the HTML
//! writer. Parsers attach semantic annotation kinds to line states; the writer later
//! turns those semantic kinds into class names and `data-*` attributes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::html) enum CodeAnnotationKind {
    Highlight,
    Warning,
    Error,
    Add,
    Remove,
    Focus,
}

impl CodeAnnotationKind {
    pub(in crate::html) fn from_str(value: &str) -> Option<Self> {
        match value {
            "highlight" => Some(Self::Highlight),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            "add" => Some(Self::Add),
            "remove" => Some(Self::Remove),
            "focus" => Some(Self::Focus),
            _ => None,
        }
    }

    pub(in crate::html) fn class_name(self) -> &'static str {
        match self {
            Self::Highlight => "ox-code-line--highlight",
            Self::Warning => "ox-code-line--warning",
            Self::Error => "ox-code-line--error",
            Self::Add => "ox-code-line--add",
            Self::Remove => "ox-code-line--remove",
            Self::Focus => "ox-code-line--focus",
        }
    }

    pub(in crate::html) fn extra_class_names(self) -> &'static [&'static str] {
        match self {
            Self::Highlight => &["highlighted"],
            Self::Warning => &["highlighted", "warning"],
            Self::Error => &["highlighted", "error"],
            Self::Add => &["diff", "add"],
            Self::Remove => &["diff", "remove"],
            Self::Focus => &["focused"],
        }
    }

    pub(in crate::html) fn block_class_name(self) -> Option<&'static str> {
        match self {
            Self::Highlight | Self::Warning | Self::Error => Some("has-highlighted"),
            Self::Add | Self::Remove => Some("has-diff"),
            Self::Focus => Some("has-focused"),
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::html) struct CodeLineRenderState {
    pub(in crate::html) value: String,
    pub(in crate::html) annotations: Vec<CodeAnnotationKind>,
}

#[derive(Debug, Clone)]
pub(in crate::html) struct CodeBlockRenderState {
    pub(in crate::html) language: Option<String>,
    pub(in crate::html) title: Option<String>,
    pub(in crate::html) line_numbers_start: Option<usize>,
    pub(in crate::html) lines: Vec<CodeLineRenderState>,
}

impl CodeBlockRenderState {
    pub(in crate::html) fn has_annotations(&self) -> bool {
        self.lines.iter().any(|line| !line.annotations.is_empty())
    }

    pub(in crate::html) fn has_focus(&self) -> bool {
        self.lines.iter().any(|line| line.annotations.contains(&CodeAnnotationKind::Focus))
    }

    pub(in crate::html) fn block_classes(&self) -> Vec<&'static str> {
        let mut classes = Vec::new();
        if self.has_annotations() || self.line_numbers_start.is_some() || self.title.is_some() {
            classes.push("ox-code-block");
        }
        if self.has_annotations() {
            classes.push("ox-code-block--annotated");
        }
        if self.line_numbers_start.is_some() {
            classes.push("ox-code-block--line-numbers");
            classes.push("line-numbers-mode");
        }
        if self.title.is_some() {
            classes.push("ox-code-block--with-title");
        }

        for line in &self.lines {
            for annotation in &line.annotations {
                if let Some(class_name) = annotation.block_class_name() {
                    if !classes.contains(&class_name) {
                        classes.push(class_name);
                    }
                }
            }
        }

        classes
    }

    pub(in crate::html) fn needs_line_wrappers(&self) -> bool {
        self.has_annotations() || self.line_numbers_start.is_some()
    }
}

#[derive(Debug, Clone)]
pub(in crate::html) struct NormalizedCodeBlockInfo {
    pub(in crate::html) language: Option<String>,
    pub(in crate::html) meta: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::html) enum MetaTokenKind {
    Raw,
    Braces,
    Brackets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::html) struct MetaToken<'a> {
    pub(in crate::html) kind: MetaTokenKind,
    pub(in crate::html) value: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::html) struct PendingCodeAnnotation {
    pub(in crate::html) kind: CodeAnnotationKind,
    pub(in crate::html) remaining: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::html) enum InlineDirectiveAction {
    Annotate { kind: CodeAnnotationKind, count: usize },
    EscapeNextLine,
}

#[derive(Debug, Clone)]
pub(in crate::html) struct ParsedInlineDirective {
    pub(in crate::html) action: InlineDirectiveAction,
    pub(in crate::html) stripped_line: String,
    pub(in crate::html) standalone: bool,
}
