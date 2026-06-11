use serde::{Deserialize, Serialize};

use super::member::NormalizedMember;

/// Normalized parameter documentation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type, or `unknown` if it cannot be inferred.
    pub type_annotation: String,
    /// Parameter description.
    pub description: String,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value if specified.
    pub default_value: Option<String>,
}

/// Normalized return documentation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedReturnDoc {
    /// Return type, or `unknown` if it cannot be inferred.
    pub type_annotation: String,
    /// Return value description.
    pub description: String,
    /// Members of an inline object literal return type.
    #[serde(default)]
    pub members: Vec<NormalizedMember>,
}

/// Normalized exception/error documentation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedThrowsDoc {
    /// Error type, when documented.
    pub type_annotation: Option<String>,
    /// Error condition description.
    pub description: String,
}

/// Normalized type parameter documentation (`<T extends C = D>`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTypeParam {
    /// Type parameter name (e.g. `T`).
    pub name: String,
    /// Constraint after `extends`, when present.
    pub constraint: Option<String>,
    /// Default type after `=`, when present.
    pub default: Option<String>,
    /// Description merged from a `@typeParam` / `@template` tag.
    pub description: String,
}
