use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for extraction operations.
pub type ExtractResult<T> = Result<T, ExtractError>;

/// Errors during documentation extraction.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Unsupported file type.
    #[error("Unsupported file type: {0}")]
    UnsupportedFile(String),
}

/// Documentation item extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    /// Item name.
    pub name: String,
    /// Item kind (function, class, interface, etc.).
    pub kind: DocItemKind,
    /// Documentation comment (JSDoc).
    pub doc: Option<String>,
    /// Source file path.
    pub source_path: String,
    /// Line number in source.
    pub line: u32,
    /// End line number in source.
    pub end_line: u32,
    /// Column number in source.
    pub column: u32,
    /// Raw JSDoc comment content without the outer delimiters.
    pub jsdoc: Option<String>,
    /// Whether the item is exported.
    pub exported: bool,
    /// Type signature (if applicable).
    pub signature: Option<String>,
    /// Extended base class/interface names.
    #[serde(default)]
    pub extends: Vec<String>,
    /// Implemented interface names.
    #[serde(default)]
    pub implements: Vec<String>,
    /// Whether a function/method declaration carries an implementation body.
    #[serde(default)]
    pub has_body: bool,
    /// Whether the item is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the item is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the item is static.
    #[serde(default)]
    pub r#static: bool,
    /// Parameters (for functions/methods).
    pub params: Vec<ParamDoc>,
    /// Return type (for functions/methods).
    pub return_type: Option<String>,
    /// Members of an inline object literal return type.
    #[serde(default)]
    pub return_members: Vec<DocItem>,
    /// Child items (for classes, modules, etc.).
    pub children: Vec<DocItem>,
    /// JSDoc tags.
    pub tags: Vec<DocTag>,
    /// Declaration type parameters (`<T extends C = D>`), in declaration order.
    #[serde(default)]
    pub type_parameters: Vec<TypeParamDoc>,
}

/// Parameter documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub type_annotation: Option<String>,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value (if any).
    pub default_value: Option<String>,
    /// Description from JSDoc @param tag.
    pub description: Option<String>,
}

/// Type parameter documentation (`<T extends C = D>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParamDoc {
    /// Type parameter name (e.g. `T`).
    pub name: String,
    /// Constraint after `extends`, when present.
    pub constraint: Option<String>,
    /// Default type after `=`, when present.
    pub default: Option<String>,
    /// Description merged from a `@typeParam` / `@template` tag (TSDoc).
    pub description: String,
}

#[derive(Debug, Clone)]
pub(super) struct FunctionTypeMetadata {
    pub(super) params: Vec<ParamDoc>,
    pub(super) return_type: Option<String>,
    pub(super) return_members: Vec<DocItem>,
    pub(super) type_parameters: Vec<TypeParamDoc>,
}

impl FunctionTypeMetadata {
    pub(super) fn as_reference_metadata(&self) -> Self {
        Self {
            params: self
                .params
                .iter()
                .map(|param| ParamDoc {
                    name: param.name.clone(),
                    type_annotation: param.type_annotation.clone(),
                    optional: param.optional,
                    default_value: param.default_value.clone(),
                    description: None,
                })
                .collect(),
            return_type: self.return_type.clone(),
            return_members: self.return_members.clone(),
            type_parameters: Vec::new(),
        }
    }
}

/// JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTag {
    /// Tag name (e.g., "param", "returns", "example").
    pub tag: String,
    /// Tag value.
    pub value: String,
    /// JSDoc type annotation, when the tag has a `{type}` part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<String>,
    /// JSDoc name, when the tag has a name part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the named part was marked optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    /// Default value from `[name=value]` syntax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Structured tag description parsed by `ox_jsdoc`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl DocTag {
    pub(super) fn new(tag: String, value: String) -> Self {
        Self {
            tag,
            value,
            type_annotation: None,
            name: None,
            optional: None,
            default_value: None,
            description: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ParsedParamTag {
    pub(super) name: String,
    pub(super) type_annotation: Option<String>,
    pub(super) optional: bool,
    pub(super) default_value: Option<String>,
    pub(super) description: Option<String>,
}

/// Kind of documentation item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocItemKind {
    /// Module or namespace.
    Module,
    /// Function.
    Function,
    /// Class.
    Class,
    /// Interface (TypeScript).
    Interface,
    /// Type alias.
    Type,
    /// Enum.
    Enum,
    /// Variable or constant.
    Variable,
    /// Class method.
    Method,
    /// Class property.
    Property,
    /// Constructor.
    Constructor,
    /// Getter.
    Getter,
    /// Setter.
    Setter,
    /// Enum member.
    #[serde(rename = "enumMember")]
    EnumMember,
    /// TypeScript index signature.
    #[serde(rename = "indexSignature")]
    IndexSignature,
}
