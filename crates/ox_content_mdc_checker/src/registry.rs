//! MDC component registry.
//!
//! Editors and CLIs load a JSON file that declares every MDC
//! component the project uses, together with each component's
//! attributes. The registry powers completion (component name after
//! `<`, attribute name inside `<Foo …>`) and hover documentation.
//!
//! The file format is intentionally minimal so authors can hand-write
//! it without a framework integration. Future PRs can layer
//! auto-discovery on top (Nuxt content config, Astro components,
//! etc.) without changing the consumer surface.
//!
//! ```json
//! {
//!   "components": [
//!     {
//!       "name": "Alert",
//!       "description": "Inline alert callout.",
//!       "attributes": [
//!         { "name": "tone", "description": "info | warn | error" },
//!         { "name": "icon" }
//!       ]
//!     }
//!   ]
//! }
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Registry {
    /// All known components, keyed by name for O(log n) lookup.
    /// `BTreeMap` is used over `HashMap` so iteration order is
    /// deterministic — completion lists, fixture snapshots, and
    /// JSON round-trips become stable across runs.
    pub components: BTreeMap<String, Component>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Component {
    pub description: Option<String>,
    pub attributes: BTreeMap<String, Attribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Attribute {
    pub description: Option<String>,
    /// Optional human-readable type hint (e.g. `"info | warn | error"`).
    #[serde(rename = "type")]
    pub type_hint: Option<String>,
    /// Marks the attribute as required so completion can flag it.
    /// Reserved for future use; not currently consumed by the LSP.
    #[serde(default)]
    pub required: bool,
}

/// Serde-friendly mirror of `Registry` that matches the on-disk
/// shape: components as an array of objects with a `name` field.
/// Keeping the disk format separate from the in-memory map lets us
/// optimize the lookup data structure without breaking the public
/// JSON schema.
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct RegistryFile {
    components: Vec<ComponentEntry>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct ComponentEntry {
    name: String,
    description: Option<String>,
    attributes: Vec<AttributeEntry>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct AttributeEntry {
    name: String,
    description: Option<String>,
    #[serde(rename = "type")]
    type_hint: Option<String>,
    #[serde(default)]
    required: bool,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a component by exact (case-sensitive) name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    /// Component names that start with `prefix` (case-sensitive). The
    /// list is alphabetical because `BTreeMap` iterates in key order.
    pub fn complete_components<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a Component)> + 'a {
        self.components
            .iter()
            .filter(move |(name, _)| name.starts_with(prefix))
            .map(|(name, component)| (name.as_str(), component))
    }

    /// Attribute names of `component` that start with `prefix`.
    /// Returns an empty iterator when the component is unknown so
    /// the caller can chain unconditionally.
    pub fn complete_attributes<'a>(
        &'a self,
        component_name: &str,
        prefix: &'a str,
    ) -> impl Iterator<Item = (&'a str, &'a Attribute)> + 'a {
        self.components
            .get(component_name)
            .into_iter()
            .flat_map(|component| component.attributes.iter())
            .filter(move |(name, _)| name.starts_with(prefix))
            .map(|(name, attribute)| (name.as_str(), attribute))
    }

    /// Parse a registry JSON string. Lenient: missing optional fields
    /// fall back to defaults; extra fields are tolerated so older
    /// editors keep working when the schema grows.
    pub fn from_json(source: &str) -> Result<Self, serde_json::Error> {
        let file: RegistryFile = serde_json::from_str(source)?;
        Ok(Self::from(file))
    }

    /// Load a registry from disk. Returns `Ok(None)` when the path
    /// doesn't exist so callers can treat "no registry configured" and
    /// "configured path missing" the same way at the completion site
    /// (silently fall through) while still failing loudly on a true
    /// parse error.
    pub fn from_path(path: &Path) -> Result<Option<Self>, RegistryError> {
        if !path.exists() {
            return Ok(None);
        }
        let source = fs::read_to_string(path).map_err(RegistryError::Io)?;
        let registry = Self::from_json(&source).map_err(RegistryError::Parse)?;
        Ok(Some(registry))
    }
}

impl From<RegistryFile> for Registry {
    fn from(file: RegistryFile) -> Self {
        let mut components = BTreeMap::new();
        for entry in file.components {
            let mut attributes = BTreeMap::new();
            for attribute in entry.attributes {
                attributes.insert(
                    attribute.name,
                    Attribute {
                        description: attribute.description,
                        type_hint: attribute.type_hint,
                        required: attribute.required,
                    },
                );
            }
            components.insert(entry.name, Component { description: entry.description, attributes });
        }
        Self { components }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("failed to read registry: {0}")]
    Io(#[source] std::io::Error),
    #[error("failed to parse registry: {0}")]
    Parse(#[source] serde_json::Error),
}

#[cfg(test)]
mod tests;
