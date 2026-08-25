//! Structured MDX metadata collected from the AST.
//!
//! Import / export syntax is scanned from `MdxjsEsm` source text. Nothing is
//! evaluated. Component names come from JSX element nodes only.

mod cursor;
mod parse;
#[cfg(test)]
mod tests;

use rustc_hash::FxHashSet;

use ox_content_ast::{Document, MdxJsxFlowElement, MdxJsxTextElement, MdxjsEsm, Visit};

use parse::parse_mdxjs_esm;

/// How a specifier was imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdxImportSpecifierKind {
    Default,
    Named,
    Namespace,
}

impl MdxImportSpecifierKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Named => "named",
            Self::Namespace => "namespace",
        }
    }
}

/// One binding created by an `import` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdxImportSpecifier {
    pub imported: String,
    pub local: String,
    pub kind: MdxImportSpecifierKind,
}

/// One `import` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdxImport {
    pub source: String,
    pub specifiers: Vec<MdxImportSpecifier>,
}

/// Imports, export names, and JSX component names from an MDX document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MdxMetadata {
    pub imports: Vec<MdxImport>,
    pub exports: Vec<String>,
    pub components: Vec<String>,
}

/// Walk `document` and collect MDX module metadata without evaluating JS.
pub fn extract_mdx_metadata(document: &Document<'_>) -> MdxMetadata {
    let mut collector = Collector::default();
    collector.visit_document(document);
    collector.finish()
}

pub fn specifier(
    imported: impl Into<String>,
    local: impl Into<String>,
    kind: MdxImportSpecifierKind,
) -> MdxImportSpecifier {
    MdxImportSpecifier { imported: imported.into(), local: local.into(), kind }
}

#[derive(Default)]
struct Collector {
    imports: Vec<MdxImport>,
    exports: Vec<String>,
    components: Vec<String>,
    seen_exports: FxHashSet<String>,
    seen_components: FxHashSet<String>,
}

impl Collector {
    fn finish(self) -> MdxMetadata {
        MdxMetadata { imports: self.imports, exports: self.exports, components: self.components }
    }

    fn push_export(&mut self, name: String) {
        if self.seen_exports.insert(name.clone()) {
            self.exports.push(name);
        }
    }

    fn push_component(&mut self, name: &str) {
        if !is_component_name(name) {
            return;
        }
        if self.seen_components.insert(name.to_string()) {
            self.components.push(name.to_string());
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_mdxjs_esm(&mut self, node: &MdxjsEsm<'a>) {
        let parsed = parse_mdxjs_esm(node.value);
        self.imports.extend(parsed.imports);
        for name in parsed.exports {
            self.push_export(name);
        }
    }

    fn visit_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'a>) {
        if let Some(name) = node.name {
            self.push_component(name);
        }
        ox_content_ast::walk_mdx_jsx_flow_element(self, node);
    }

    fn visit_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'a>) {
        if let Some(name) = node.name {
            self.push_component(name);
        }
        ox_content_ast::walk_mdx_jsx_text_element(self, node);
    }
}

fn is_component_name(name: &str) -> bool {
    let Some(first) = name.chars().next() else {
        return false;
    };
    first.is_uppercase() || name.contains('.')
}
