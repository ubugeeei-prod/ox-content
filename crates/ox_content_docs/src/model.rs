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
    /// Members belonging to class/interface/type/enum entries.
    #[serde(default)]
    pub members: Vec<ApiDocMember>,
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
}

/// Custom JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocTag {
    /// Tag name without `@`.
    pub tag: String,
    /// Tag value.
    pub value: String,
}
