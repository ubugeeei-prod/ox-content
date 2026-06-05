//! Shared intermediate representation (IR) for generated API documentation.
//!
//! These types are the renderer-agnostic data model produced from extracted
//! JSDoc and consumed by the Markdown (`markdown.rs`) and JSON (`data.rs`)
//! generators. Keeping them here (rather than inside a specific renderer module)
//! lets every renderer depend on the IR instead of on each other.

use serde::{Deserialize, Serialize};

/// Extracted docs for one source module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocModule {
    /// Source file path.
    pub file: String,
    /// Module-level description from the entry file's `@module` block or leading
    /// file comment. Empty when the source has no module-level JSDoc.
    #[serde(default)]
    pub description: String,
    /// Absolute source path of the entry point this module was generated from.
    ///
    /// Used by the TypeDoc path strategy to place a re-exported symbol's
    /// canonical page under its defining module (when that module is itself an
    /// entry point). Empty when the caller does not supply it; dedup then falls
    /// back to the first module that exports the symbol.
    #[serde(default)]
    pub source_path: String,
    /// Module-level example blocks from the entry file's `@module` block or
    /// leading file comment.
    #[serde(default)]
    pub examples: Vec<String>,
    /// Module-level custom JSDoc tags, kept in source insertion order where the
    /// caller provides ordered tags.
    #[serde(default)]
    pub tags: Vec<ApiDocTag>,
    /// Documented entries in the source file.
    pub entries: Vec<ApiDocEntry>,
}

/// A single normalized API documentation entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocEntry {
    /// Entry name.
    pub name: String,
    /// Entry kind.
    pub kind: String,
    /// Human-readable description.
    pub description: String,
    /// Parameters.
    pub params: Vec<ApiParamDoc>,
    /// Return documentation.
    pub returns: Option<ApiReturnDoc>,
    /// Example blocks.
    pub examples: Vec<String>,
    /// Custom JSDoc tags, kept in source insertion order.
    pub tags: Vec<ApiDocTag>,
    /// Whether the entry is private.
    pub private: bool,
    /// Source file path.
    pub file: String,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
    /// Full source signature.
    pub signature: Option<String>,
    /// Extended base class/interface names.
    #[serde(default)]
    pub extends: Vec<String>,
    /// Implemented interface names.
    #[serde(default)]
    pub implements: Vec<String>,
    /// Whether a function declaration carries an implementation body. `false` for
    /// overload signatures and ambient (`declare` / `.d.ts`) declarations. The
    /// TypeDoc path renderer uses this to hide the implementation signature when a
    /// symbol's overloads are grouped onto one page.
    #[serde(default)]
    pub has_body: bool,
    /// Members belonging to class/interface/type/enum entries.
    #[serde(default)]
    pub members: Vec<ApiDocMember>,
    /// Declaration type parameters (opt-in; empty unless enabled).
    #[serde(default)]
    pub type_parameters: Vec<ApiTypeParamDoc>,
}

/// Documentation for a member of a class/interface/type/enum entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocMember {
    /// Member name.
    pub name: String,
    /// Member kind.
    pub kind: String,
    /// Human-readable description.
    pub description: String,
    /// Full member signature.
    pub signature: Option<String>,
    /// Property or enum member type/value annotation.
    pub type_annotation: Option<String>,
    /// Parameters.
    #[serde(default)]
    pub params: Vec<ApiParamDoc>,
    /// Member type parameters (opt-in; empty unless enabled).
    #[serde(default)]
    pub type_parameters: Vec<ApiTypeParamDoc>,
    /// Return documentation.
    pub returns: Option<ApiReturnDoc>,
    /// Whether the member is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the member is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the member is static.
    #[serde(default)]
    pub r#static: bool,
    /// Whether the member is private.
    #[serde(default)]
    pub private: bool,
    /// Custom JSDoc tags, kept in source insertion order.
    #[serde(default)]
    pub tags: Vec<ApiDocTag>,
    /// Implemented interface members this member satisfies.
    #[serde(default)]
    pub implementation_of: Vec<String>,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
}

/// Parameter documentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub type_annotation: String,
    /// Parameter description.
    pub description: String,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value.
    pub default_value: Option<String>,
}

/// Return type documentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiReturnDoc {
    /// Return type.
    pub type_annotation: String,
    /// Return description.
    pub description: String,
    /// Members of an inline object literal return type.
    #[serde(default)]
    pub members: Vec<ApiDocMember>,
}

/// Type parameter documentation (`<T extends C = D>`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiTypeParamDoc {
    /// Type parameter name (e.g. `T`).
    pub name: String,
    /// Constraint after `extends`, when present.
    pub constraint: Option<String>,
    /// Default type after `=`, when present.
    pub default: Option<String>,
    /// Description merged from a `@typeParam` / `@template` tag.
    pub description: String,
}

/// Custom JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocTag {
    /// Tag name without `@`.
    pub tag: String,
    /// Tag value.
    pub value: String,
}
