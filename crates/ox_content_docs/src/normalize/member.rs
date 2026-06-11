use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::kind::NormalizedMemberKind;
use super::model::{
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};

/// Normalized documentation for a member of a class/interface/type/enum entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedMember {
    /// Member name.
    pub name: String,
    /// Member kind.
    pub kind: NormalizedMemberKind,
    /// Human-readable description.
    pub description: String,
    /// Full member signature, for callable members.
    pub signature: Option<String>,
    /// Property or enum member type/value annotation.
    pub type_annotation: Option<String>,
    /// Default value extracted from `@default` / `@defaultValue`.
    #[serde(default)]
    pub default_value: Option<String>,
    /// Parameters, if any.
    #[serde(default)]
    pub params: Vec<NormalizedParamDoc>,
    /// Member type parameters. Populated only when type-parameter docs are
    /// enabled (opt-in); empty otherwise.
    #[serde(default)]
    pub type_parameters: Vec<NormalizedTypeParam>,
    /// Return documentation, if any.
    pub returns: Option<NormalizedReturnDoc>,
    /// Exceptions/errors documented with `@throws` / `@exception`.
    #[serde(default)]
    pub throws: Vec<NormalizedThrowsDoc>,
    /// Nested members owned by this member (for property-owned object literals).
    #[serde(default)]
    pub members: Vec<NormalizedMember>,
    /// Whether the member is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the member is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the member is static.
    #[serde(default)]
    pub r#static: bool,
    /// Whether the member is marked private.
    #[serde(default)]
    pub private: bool,
    /// Custom JSDoc tags.
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
}

impl Default for NormalizedMember {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: NormalizedMemberKind::default(),
            description: String::new(),
            signature: None,
            type_annotation: None,
            default_value: None,
            params: Vec::new(),
            type_parameters: Vec::new(),
            returns: None,
            throws: Vec::new(),
            members: Vec::new(),
            optional: false,
            readonly: false,
            r#static: false,
            private: false,
            tags: BTreeMap::new(),
            line: 1,
            end_line: 1,
        }
    }
}
