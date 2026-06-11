use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::kind::NormalizedDocKind;
use super::member::NormalizedMember;
use super::model::{
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};

/// Normalized documentation entry consumed by generated API docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedDocEntry {
    /// Entry name.
    pub name: String,
    /// Entry kind.
    pub kind: NormalizedDocKind,
    /// Human-readable description.
    pub description: String,
    /// Parameters, if any.
    pub params: Vec<NormalizedParamDoc>,
    /// Return documentation, if any.
    pub returns: Option<NormalizedReturnDoc>,
    /// Exceptions/errors documented with `@throws` / `@exception`.
    #[serde(default)]
    pub throws: Vec<NormalizedThrowsDoc>,
    /// Example blocks.
    pub examples: Vec<String>,
    /// Custom JSDoc tags.
    pub tags: BTreeMap<String, String>,
    /// Whether the entry is marked private.
    pub private: bool,
    /// Source file path.
    pub file: String,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
    /// Signature text.
    pub signature: Option<String>,
    /// Extended base class/interface names.
    #[serde(default)]
    pub extends: Vec<String>,
    /// Implemented interface names.
    #[serde(default)]
    pub implements: Vec<String>,
    /// Whether a function declaration carries an implementation body (`false` for
    /// overload signatures and ambient declarations). Used to hide the
    /// implementation signature when grouping overloads on TypeDoc symbol pages.
    #[serde(default)]
    pub has_body: bool,
    /// Members belonging to class/interface/type/enum entries.
    #[serde(default)]
    pub members: Vec<NormalizedMember>,
    /// Declaration type parameters. Populated only when type-parameter docs are
    /// enabled (opt-in); empty otherwise.
    #[serde(default)]
    pub type_parameters: Vec<NormalizedTypeParam>,
}

impl Default for NormalizedDocEntry {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: NormalizedDocKind::default(),
            description: String::new(),
            params: Vec::new(),
            returns: None,
            throws: Vec::new(),
            examples: Vec::new(),
            tags: BTreeMap::new(),
            private: false,
            file: String::new(),
            line: 1,
            end_line: 1,
            signature: None,
            extends: Vec::new(),
            implements: Vec::new(),
            has_body: false,
            members: Vec::new(),
            type_parameters: Vec::new(),
        }
    }
}
