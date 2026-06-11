use serde::{Deserialize, Serialize};

use crate::extractor::DocItemKind;

/// Documentation item kind supported by the generated API reference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizedDocKind {
    /// Function declaration or function-valued variable.
    #[default]
    Function,
    /// Class declaration.
    Class,
    /// TypeScript interface declaration.
    Interface,
    /// Type alias.
    Type,
    /// Enum declaration.
    Enum,
    /// Variable declaration.
    Variable,
    /// Module or namespace.
    Module,
}

impl NormalizedDocKind {
    /// Converts extractor-level kinds into public documentation kinds.
    #[must_use]
    pub fn from_doc_item_kind(kind: DocItemKind) -> Option<Self> {
        match kind {
            DocItemKind::Function => Some(Self::Function),
            DocItemKind::Class => Some(Self::Class),
            DocItemKind::Interface => Some(Self::Interface),
            DocItemKind::Type => Some(Self::Type),
            DocItemKind::Enum => Some(Self::Enum),
            DocItemKind::Variable => Some(Self::Variable),
            DocItemKind::Module => Some(Self::Module),
            DocItemKind::Method
            | DocItemKind::Property
            | DocItemKind::Constructor
            | DocItemKind::Getter
            | DocItemKind::Setter
            | DocItemKind::EnumMember
            | DocItemKind::IndexSignature => None,
        }
    }

    /// Returns the JavaScript-facing string for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Type => "type",
            Self::Enum => "enum",
            Self::Variable => "variable",
            Self::Module => "module",
        }
    }
}

/// Documentation item kind supported for class/interface/type/enum members.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizedMemberKind {
    /// Object or class property.
    #[default]
    Property,
    /// Method signature.
    Method,
    /// Class constructor.
    Constructor,
    /// Getter accessor.
    Getter,
    /// Setter accessor.
    Setter,
    /// Enum member.
    #[serde(rename = "enumMember")]
    EnumMember,
    /// TypeScript index signature.
    #[serde(rename = "indexSignature")]
    IndexSignature,
}

impl NormalizedMemberKind {
    /// Converts extractor-level member kinds into public member kinds.
    #[must_use]
    pub fn from_doc_item_kind(kind: DocItemKind) -> Option<Self> {
        match kind {
            DocItemKind::Property => Some(Self::Property),
            DocItemKind::Method => Some(Self::Method),
            DocItemKind::Constructor => Some(Self::Constructor),
            DocItemKind::Getter => Some(Self::Getter),
            DocItemKind::Setter => Some(Self::Setter),
            DocItemKind::EnumMember => Some(Self::EnumMember),
            DocItemKind::IndexSignature => Some(Self::IndexSignature),
            DocItemKind::Module
            | DocItemKind::Function
            | DocItemKind::Class
            | DocItemKind::Interface
            | DocItemKind::Type
            | DocItemKind::Enum
            | DocItemKind::Variable => None,
        }
    }

    /// Returns the JavaScript-facing string for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Property => "property",
            Self::Method => "method",
            Self::Constructor => "constructor",
            Self::Getter => "getter",
            Self::Setter => "setter",
            Self::EnumMember => "enumMember",
            Self::IndexSignature => "indexSignature",
        }
    }
}
