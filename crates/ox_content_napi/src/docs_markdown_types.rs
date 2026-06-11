use std::collections::HashMap;

use napi_derive::napi;

/// Normalized parameter documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocParam {
    pub name: String,
    pub r#type: String,
    pub description: String,
    pub optional: Option<bool>,
    pub r#default: Option<String>,
}

/// Normalized return documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocReturn {
    pub r#type: String,
    pub description: String,
    pub members: Option<Vec<JsDocMember>>,
}

/// Exception/error documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocThrows {
    pub r#type: Option<String>,
    pub description: String,
}

/// Type parameter documentation (`<T extends C = D>`) used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsTypeParam {
    pub name: String,
    pub constraint: Option<String>,
    pub r#default: Option<String>,
    pub description: String,
}

/// Normalized member documentation used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocMember {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub signature: Option<String>,
    pub r#type: Option<String>,
    pub r#default: Option<String>,
    pub params: Option<Vec<JsDocParam>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
    pub returns: Option<JsDocReturn>,
    pub throws: Option<Vec<JsDocThrows>>,
    pub members: Option<Vec<JsDocMember>>,
    pub optional: Option<bool>,
    pub readonly: Option<bool>,
    pub r#static: Option<bool>,
    pub private: Option<bool>,
    pub tags: Option<HashMap<String, String>>,
    pub implementation_of: Option<Vec<String>>,
    pub line: u32,
    pub end_line: u32,
}

/// Normalized documentation entry used by generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocEntry {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub params: Option<Vec<JsDocParam>>,
    pub returns: Option<JsDocReturn>,
    pub throws: Option<Vec<JsDocThrows>>,
    pub examples: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
    pub private: bool,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub signature: Option<String>,
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    /// Whether a function declaration carries an implementation body. `false` for
    /// overload signatures and ambient declarations.
    pub has_body: bool,
    pub members: Option<Vec<JsDocMember>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
}

/// Navigation item emitted for generated documentation.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsNavItem {
    pub title: String,
    pub path: String,
    pub children: Option<Vec<JsDocsNavItem>>,
}

/// Options for generating sidebar navigation metadata from extracted docs.
#[napi(object)]
#[derive(Default)]
pub struct JsDocsNavOptions {
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    /// TypeDoc-style group order for nav groups (matches `generateDocsMarkdown`'s
    /// `groupOrder` so the sidebar and page order stay in sync).
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style `sort`: ordered sort strategies for nav leaf entries (matches
    /// `generateDocsMarkdown`'s `sort`). Unsupported strategies are ignored.
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve the caller-provided
    /// module order instead of sorting alphabetically. Defaults to `true`.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style `kindSortOrder`: kind ranking used for nav group order (before
    /// `groupOrder`) and the `kind` sort strategy.
    pub kind_sort_order: Option<Vec<String>>,
    /// Single-entry root handling for TypeDoc-style nav.
    #[napi(ts_type = "'preserve' | 'flatten'")]
    pub single_entry_root: Option<String>,
}

/// Ordered JSDoc tag used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownTag {
    pub tag: String,
    pub value: String,
}

/// Documentation entry used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownEntry {
    pub name: String,
    pub kind: String,
    pub description: String,
    pub params: Option<Vec<JsDocParam>>,
    pub returns: Option<JsDocReturn>,
    pub throws: Option<Vec<JsDocThrows>>,
    pub examples: Option<Vec<String>>,
    pub tags: Option<Vec<JsDocsMarkdownTag>>,
    pub private: bool,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub signature: Option<String>,
    pub extends: Option<Vec<String>>,
    pub implements: Option<Vec<String>>,
    /// Whether a function declaration carries an implementation body. Optional so
    /// callers that build entries by hand need not set it; defaults to `false`.
    /// Round-trips from `extractDocsFromEntryPoints` output unchanged.
    pub has_body: Option<bool>,
    pub members: Option<Vec<JsDocMember>>,
    pub type_parameters: Option<Vec<JsTypeParam>>,
}

/// Extracted docs for one source file used by generated API Markdown.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsMarkdownModule {
    pub file: String,
    /// Module-level description from the entry file's `@module` / leading JSDoc.
    pub description: Option<String>,
    /// Absolute source path of the entry point (from `extractDocsFromEntryPoints`'
    /// `sourcePath`). Optional; when provided, the TypeDoc path strategy places a
    /// re-exported symbol's canonical page under its defining module.
    pub source_path: Option<String>,
    /// Module-level example blocks from the entry file's `@module` / leading JSDoc.
    pub examples: Option<Vec<String>>,
    /// Module-level custom JSDoc tags.
    pub tags: Option<Vec<JsDocsMarkdownTag>>,
    pub entries: Vec<JsDocsMarkdownEntry>,
}

/// Extracted docs for one source file returned to JavaScript callers.
#[napi(object)]
#[derive(Clone)]
pub struct JsExtractedDocsModule {
    pub file: String,
    pub entries: Vec<JsDocEntry>,
}

/// Options for generated API Markdown.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsDocsMarkdownOptions {
    pub group_by: Option<String>,
    pub github_url: Option<String>,
    #[napi(ts_type = "'markdown' | 'clean'")]
    pub link_style: Option<String>,
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    #[napi(ts_type = "'html' | 'markdown'")]
    pub render_style: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub index_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub parameters_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub interface_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub class_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub type_alias_properties_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub enum_members_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub property_members_format: Option<String>,
    #[napi(ts_type = "'none' | 'list' | 'table'")]
    pub type_declaration_format: Option<String>,
    /// Emit the stats summary line on index pages (default: true).
    pub render_stats: Option<bool>,
    /// Emit the generated-by attribution on root index pages (default: true).
    pub render_generated_by: Option<bool>,
    /// TypeDoc-style group order for module index sections and nav groups. Unlisted
    /// groups are sorted alphabetically at `*` (or at the end when `*` is absent).
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style `sort`: ordered sort strategies applied to entries and members.
    /// Later strategies break ties left by earlier ones. Omit to keep the default
    /// (alphabetical, with enum members in declaration order). Unsupported
    /// strategies (e.g. `enum-value-*`, `documents-*`) are ignored.
    #[napi(
        ts_type = "Array<'source-order' | 'alphabetical' | 'alphabetical-ignoring-documents' | 'enum-value-ascending' | 'enum-value-descending' | 'static-first' | 'instance-first' | 'visibility' | 'required-first' | 'kind' | 'external-last' | 'documents-first' | 'documents-last'>"
    )]
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve the caller-provided
    /// module order instead of sorting alphabetically. Defaults to `true`.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style `kindSortOrder`: declaration kind ranking used as the base order
    /// for module index sections / nav groups (before `groupOrder`) and the `kind`
    /// sort strategy.
    pub kind_sort_order: Option<Vec<String>>,
    /// Single-entry root handling for TypeDoc-style Markdown output.
    #[napi(ts_type = "'preserve' | 'flatten'")]
    pub single_entry_root: Option<String>,
}

/// Options for writing generated API documentation files.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsDocsOutputOptions {
    pub generate_nav: Option<bool>,
    pub group_by: Option<String>,
    pub generated_at: Option<String>,
    pub base_path: Option<String>,
    #[napi(ts_type = "'flat' | 'typedoc'")]
    pub path_strategy: Option<String>,
    /// TypeDoc-style group order for generated nav groups.
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style sort strategies for generated nav leaf entries.
    pub sort: Option<Vec<String>>,
    /// TypeDoc-style `sortEntryPoints`: when `false`, preserve module order.
    pub sort_entry_points: Option<bool>,
    /// TypeDoc-style kind ranking for generated nav groups.
    pub kind_sort_order: Option<Vec<String>>,
    /// Single-entry root handling for generated nav metadata.
    #[napi(ts_type = "'preserve' | 'flatten'")]
    pub single_entry_root: Option<String>,
}
