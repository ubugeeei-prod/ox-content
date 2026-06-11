use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Comment;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use super::jsdoc::build_jsdoc_cache;
use super::{DocExtractor, DocItem, DocVisitor, ExtractError, ExtractResult};

impl DocExtractor {
    /// Creates a new documentation extractor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_private: false,
            include_internal: false,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Creates a new extractor that includes private items.
    #[must_use]
    pub fn with_private(include_private: bool) -> Self {
        Self {
            include_private,
            include_internal: false,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Creates a new extractor with explicit visibility options.
    #[must_use]
    pub fn with_visibility(include_private: bool, include_internal: bool) -> Self {
        Self {
            include_private,
            include_internal,
            include_undocumented_declarations: false,
            capture_jsdoc_raw: true,
        }
    }

    /// Drops the verbatim JSDoc text from extracted items.
    ///
    /// For callers that immediately normalize (which discards `DocItem::jsdoc`),
    /// this avoids allocating and cloning the raw comment text per declaration.
    #[must_use]
    pub fn without_raw_jsdoc(mut self) -> Self {
        self.capture_jsdoc_raw = false;
        self
    }

    /// Creates a new extractor for public entry point exports.
    ///
    /// This path always normalizes, so it never captures the raw JSDoc text.
    #[must_use]
    pub(crate) fn for_entrypoint_exports(include_private: bool, include_internal: bool) -> Self {
        Self {
            include_private,
            include_internal,
            include_undocumented_declarations: true,
            capture_jsdoc_raw: false,
        }
    }

    /// Extracts documentation from a source file.
    pub fn extract_file(&self, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let mut allocator = Allocator::default();
        self.extract_file_with(&mut allocator, path)
    }

    /// Like [`extract_file`](Self::extract_file), but reuses a caller-owned
    /// arena allocator.
    ///
    /// Batch callers (directory walks, multi-file extraction) create one
    /// [`Allocator`] and pass it to every file. The arena is rewound at the
    /// start of each parse, so a single allocation is reused instead of
    /// allocating and freeing a fresh multi-MB arena per file. The public
    /// [`extract_file`](Self::extract_file) wrapper just hands in a fresh one.
    pub(crate) fn extract_file_with(
        &self,
        allocator: &mut Allocator,
        path: &Path,
    ) -> ExtractResult<Vec<DocItem>> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            "ts" | "tsx" | "js" | "jsx" | "mts" | "mjs" | "cts" | "cjs" => {
                self.extract_js_ts(allocator, path)
            }
            _ => Err(ExtractError::UnsupportedFile(extension.to_string())),
        }
    }

    /// Extracts documentation from source code string.
    pub fn extract_source(
        &self,
        source: &str,
        file_path: &str,
        source_type: SourceType,
    ) -> ExtractResult<Vec<DocItem>> {
        let mut allocator = Allocator::default();
        self.extract_source_with(&mut allocator, source, file_path, source_type)
    }

    /// Like [`extract_source`](Self::extract_source), but reuses a caller-owned
    /// arena allocator. See [`extract_file_with`](Self::extract_file_with).
    pub(crate) fn extract_source_with(
        &self,
        allocator: &mut Allocator,
        source: &str,
        file_path: &str,
        source_type: SourceType,
    ) -> ExtractResult<Vec<DocItem>> {
        profile_span!("docs::extract_source");
        // Rewind the arena so a reused allocator starts clean for this file.
        // Extraction returns owned `DocItem`s, so by the time the next file
        // resets, nothing borrows the previous file's arena.
        allocator.reset();
        let ret = {
            profile_span!("docs::oxc_parse");
            Parser::new(&*allocator, source, source_type).parse()
        };

        if !ret.errors.is_empty() {
            let error_msg = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExtractError::Parse(error_msg));
        }

        Ok(self.extract_items_from_program(source, file_path, &ret.program))
    }

    /// Extract doc items from an already-parsed program (build the JSDoc cache,
    /// then walk the AST).
    ///
    /// Shared by [`extract_source_with`](Self::extract_source_with) and the
    /// export-graph walk: the graph parses every reachable module to collect its
    /// exports, so it extracts docs from that same AST instead of parsing the
    /// file a second time.
    pub(crate) fn extract_items_from_program(
        &self,
        source: &str,
        file_path: &str,
        program: &oxc_ast::ast::Program<'_>,
    ) -> Vec<DocItem> {
        let comments: Vec<Comment> = program.comments.iter().copied().collect();
        let jsdoc_cache = build_jsdoc_cache(source, &comments, self.capture_jsdoc_raw);

        let mut visitor = DocVisitor::new(
            source,
            file_path,
            self.include_private,
            self.include_internal,
            self.include_undocumented_declarations,
            jsdoc_cache,
        );
        let first_stmt_start = program.body.first().map(|statement| statement.span().start);
        {
            profile_span!("docs::visit_ast");
            if let Some(module_item) = visitor.extract_module_entry(&comments, first_stmt_start) {
                visitor.items.push(module_item);
            }
            visitor.visit_program(program);
        }

        visitor.items
    }

    /// Extracts documentation from a JavaScript/TypeScript file, reusing the
    /// caller-owned arena allocator.
    fn extract_js_ts(&self, allocator: &mut Allocator, path: &Path) -> ExtractResult<Vec<DocItem>> {
        let content = std::fs::read_to_string(path)?;
        let file_path = path.to_string_lossy().to_string();
        let source_type = SourceType::from_path(path).unwrap_or_default();

        self.extract_source_with(allocator, &content, &file_path, source_type)
    }
}

impl Default for DocExtractor {
    fn default() -> Self {
        Self::new()
    }
}
