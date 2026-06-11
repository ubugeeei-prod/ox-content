use std::path::Path;

use oxc_ast::ast::Comment;
use rustc_hash::FxHashMap;

use super::jsdoc::{parse_jsdoc_payload, ParsedJsdoc, MODULE_MARKER_TAGS};
use super::{DocItem, DocItemKind, DocTag, DocVisitor};

impl<'a> DocVisitor<'a> {
    pub(super) fn new(
        source: &'a str,
        file_path: &'a str,
        include_private: bool,
        include_internal: bool,
        include_undocumented_declarations: bool,
        jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    ) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0);
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter_map(|(index, byte)| (byte == b'\n').then_some(index + 1)),
        );

        Self {
            source,
            file_path,
            include_private,
            include_internal,
            include_undocumented_declarations,
            jsdoc_cache,
            line_starts,
            items: Vec::new(),
            type_alias_function_metadata: FxHashMap::default(),
            has_default_export: false,
        }
    }

    pub(super) fn slice(&self, start: u32, end: u32) -> String {
        self.source[start as usize..end as usize].to_string()
    }

    pub(super) fn line_number(&self, position: u32) -> u32 {
        let position = position as usize;
        self.line_starts.partition_point(|&start| start <= position) as u32
    }

    pub(super) fn column_number(&self, position: u32) -> u32 {
        let position = position as usize;
        let line_index = self.line_starts.partition_point(|&start| start <= position);
        let line_start = self.line_starts[line_index.saturating_sub(1)];
        (position.saturating_sub(line_start)) as u32
    }

    pub(super) fn span_lines(&self, start: u32, end: u32) -> (u32, u32) {
        let start_line = self.line_number(start);
        let end_position = end.saturating_sub(1).max(start);
        let end_line = self.line_number(end_position);
        (start_line, end_line)
    }

    pub(super) fn extract_jsdoc(&self, attached_to: u32) -> Option<(String, String, Vec<DocTag>)> {
        self.jsdoc_cache.get(&attached_to).cloned()
    }

    pub(super) fn extract_declaration_docs(
        &self,
        attached_to: u32,
    ) -> Option<(Option<String>, Option<String>, Vec<DocTag>)> {
        if let Some((jsdoc, doc, tags)) = self.jsdoc_cache.get(&attached_to) {
            // Borrow from the cache and apply the visibility filter first, so a
            // private/internal declaration is dropped without cloning the whole
            // (raw, description, tags) payload only to throw it away.
            if self.should_skip_by_visibility(tags) {
                return None;
            }
            return Some((
                Some(jsdoc.clone()),
                (!doc.is_empty()).then(|| doc.clone()),
                tags.clone(),
            ));
        }

        self.include_undocumented_declarations.then(|| (None, None, Vec::new()))
    }

    pub(super) fn extract_module_entry(
        &self,
        comments: &[Comment],
        first_stmt_start: Option<u32>,
    ) -> Option<DocItem> {
        let comment = comments.iter().find(|comment| comment.is_jsdoc())?;

        // Only the leading file comment (before the first statement) can be the
        // module comment; a JSDoc that follows code documents that declaration.
        if let Some(stmt_start) = first_stmt_start {
            if comment.span.start > stmt_start {
                return None;
            }
        }

        // Parse the candidate from its own span rather than the `attached_to`
        // cache, which collides when two leading comments share a target.
        let (raw, doc, tags) = parse_jsdoc_payload(self.source, comment);

        // Treat the leading comment as the module description when it either
        // carries a module marker tag (`@module` / `@packageDocumentation` /
        // `@fileoverview`, matching TypeDoc) or is detached from the following
        // code by a blank line. Otherwise it belongs to the first declaration.
        let has_module_marker =
            tags.iter().any(|tag| MODULE_MARKER_TAGS.contains(&tag.tag.as_str()));
        if !has_module_marker
            && !self.is_detached_leading_comment(comment, comments, first_stmt_start)
        {
            return None;
        }

        let (module_name, module_description) =
            Self::parse_module_tag(&tags).unwrap_or((None, None));
        let name = module_name.unwrap_or_else(|| self.file_stem_module_name());
        let (line, end_line) = self.span_lines(comment.span.start, comment.span.end);

        Some(DocItem {
            name,
            kind: DocItemKind::Module,
            doc: if doc.is_empty() { module_description } else { Some(doc) },
            source_path: self.file_path.to_string(),
            line,
            end_line,
            column: self.column_number(comment.span.start),
            jsdoc: Some(raw),
            exported: true,
            signature: None,
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            optional: false,
            readonly: false,
            r#static: false,
            params: Vec::new(),
            return_type: None,
            return_members: Vec::new(),
            children: Vec::new(),
            tags,
            type_parameters: Vec::new(),
        })
    }

    /// Returns true when the first leading JSDoc comment is detached from the
    /// following code by a blank line (or is the file's only content). Such a
    /// comment is a file-level/module comment rather than the doc of the first
    /// declaration, matching TypeDoc's leading-comment handling.
    fn is_detached_leading_comment(
        &self,
        comment: &Comment,
        comments: &[Comment],
        first_stmt_start: Option<u32>,
    ) -> bool {
        let after = comment.span.end;
        // The next syntactic element after this comment: the nearest of the
        // first statement and any later comment.
        let next_comment =
            comments.iter().map(|other| other.span.start).filter(|&start| start > after).min();
        let next_pos = match (next_comment, first_stmt_start) {
            (Some(comment_start), Some(stmt_start)) => Some(comment_start.min(stmt_start)),
            (Some(position), None) | (None, Some(position)) => Some(position),
            (None, None) => None,
        };
        // Nothing follows the comment: a comment-only file is a module comment.
        let Some(next_pos) = next_pos else {
            return true;
        };
        if next_pos <= after {
            return false;
        }
        // A blank line means two or more newlines between the comment and the
        // next element.
        self.source[after as usize..next_pos as usize].bytes().filter(|&byte| byte == b'\n').count()
            >= 2
    }

    fn parse_module_tag(tags: &[DocTag]) -> Option<(Option<String>, Option<String>)> {
        let tag = tags.iter().find(|tag| tag.tag == "module")?;
        let value = tag.value.trim();
        if value.is_empty() {
            return Some((None, tag.description.clone()));
        }

        let split_at = value
            .char_indices()
            .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
            .unwrap_or(value.len());
        let name = value[..split_at].trim();
        let rest = value[split_at..].trim();
        let description = Self::clean_tag_description(rest).or_else(|| tag.description.clone());

        Some(((!name.is_empty()).then(|| name.to_string()), description))
    }

    fn file_stem_module_name(&self) -> String {
        Path::new(self.file_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("module")
            .to_string()
    }
}
