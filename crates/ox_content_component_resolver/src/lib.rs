//! Resolves component prop metadata for Ox Content Markdown/MDC
//! documents via `typescript-go` (through the `corsa_client` crate).
//!
//! This crate is the foundation for the "Vue / React props completion +
//! jump + typecheck" roadmap item. The first PR establishes the public
//! API surface and proves the dependency wires through the workspace.
//! Subsequent PRs land the actual resolver implementation, the LSP
//! integration, and the `ox-content-component-check` CLI.
//!
//! ```rust,no_run
//! # async fn demo() -> Result<(), ox_content_component_resolver::Error> {
//! use ox_content_component_resolver::{Resolver, ResolverConfig};
//!
//! let resolver = Resolver::spawn(ResolverConfig::with_tsgo_path("/usr/local/bin/tsgo")).await?;
//! let component = resolver.resolve_component_props(
//!     std::path::Path::new("/repo/src/Alert.tsx"),
//! ).await?;
//! for prop in &component.props {
//!     println!("{}: {}", prop.name, prop.type_text);
//! }
//! # Ok(()) }
//! ```
//!
//! # Why a Rust-side resolver
//!
//! The LSP already runs in Rust, and pushing through a Node sidecar to
//! reach `tsgo` would (a) double the process count per workspace and
//! (b) add an extra IPC hop on every prop hover. `corsa_client` talks
//! to `tsgo` directly over stdio, so the LSP can share a single
//! long-lived worker per workspace.
//!
//! # Opt-in
//!
//! The resolver is **disabled by default**. Users opt in by setting
//! `oxContent.components.tsgoPath` (VS Code), the
//! `OX_CONTENT_TSGO_PATH` environment variable, or the
//! `componentsTsgoPath` LSP initialization option. When the path is
//! not configured, the LSP / CLI consumers must surface this gracefully
//! (no completion, no diagnostics — the rest of the feature set keeps
//! working).

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Configuration for a [`Resolver`]. The minimal contract is a path
/// to a `tsgo` binary; everything else has sensible defaults.
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Absolute path to the `tsgo` (typescript-go) executable.
    pub tsgo_path: PathBuf,
    /// Path to the workspace `tsconfig.json` that the resolver
    /// should load when opening the first project. Defaults to the
    /// closest `tsconfig.json` walking up from the component file
    /// when `None`.
    pub tsconfig_path: Option<PathBuf>,
}

impl ResolverConfig {
    pub fn with_tsgo_path(path: impl Into<PathBuf>) -> Self {
        Self { tsgo_path: path.into(), tsconfig_path: None }
    }

    #[must_use]
    pub fn with_tsconfig(mut self, path: impl Into<PathBuf>) -> Self {
        self.tsconfig_path = Some(path.into());
        self
    }
}

/// Source location of a definition. Lines and columns are 1-indexed
/// because callers (LSP, CLI) translate them into editor coordinates
/// directly and the off-by-one is a frequent footgun otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

/// A single prop on a component's props type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedProp {
    pub name: String,
    /// Human-readable rendering of the prop type, e.g. `"string"`,
    /// `"'info' | 'warn' | 'error'"`, or `"() => void"`.
    pub type_text: String,
    pub optional: bool,
    pub docs: Option<String>,
    pub location: Option<Location>,
}

/// Result of resolving a component reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedComponent {
    /// Component identifier as written in the MDC source (e.g. `"Alert"`).
    pub name: String,
    /// Absolute path of the file declaring the component.
    pub source_file: PathBuf,
    /// Resolved props in declaration order. Empty when the component
    /// has no props type or when extraction was best-effort and the
    /// type couldn't be flattened (a follow-up will surface a
    /// diagnostic for the latter case).
    pub props: Vec<ResolvedProp>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "ox-content-component-resolver is not yet implemented (PR #195 + #196 land in stages)"
    )]
    NotImplemented,

    #[error("tsgo binary not found at {}", .0.display())]
    TsgoMissing(PathBuf),

    #[error("tsgo I/O error: {0}")]
    Tsgo(#[from] corsa_client::TsgoError),

    #[error("invalid component path {path}: {reason}", path = .path.display())]
    InvalidComponentPath { path: PathBuf, reason: String },
}

/// Wraps a long-lived `corsa_client::ApiClient` that talks to a
/// single `tsgo` worker per workspace.
#[derive(Debug)]
pub struct Resolver {
    #[allow(dead_code)] // Used by the real implementation in the follow-up PR.
    config: ResolverConfig,
}

impl Resolver {
    /// Spawn a tsgo worker and prepare the client for incoming
    /// `resolve_component_props` calls.
    ///
    /// The scaffold returns `Ok(Self)` without actually spawning the
    /// process so the rest of the workspace can integrate the public
    /// types ahead of the real implementation landing.
    #[allow(clippy::unused_async)] // The follow-up PR replaces the body with real async work.
    pub async fn spawn(config: ResolverConfig) -> Result<Self, Error> {
        if !config.tsgo_path.exists() {
            return Err(Error::TsgoMissing(config.tsgo_path));
        }
        Ok(Self { config })
    }

    /// Resolve the props of the component defined by `component_file`.
    ///
    /// The scaffold returns [`Error::NotImplemented`]. The follow-up PR
    /// fills in the real logic:
    ///
    /// 1. open `component_file` as a tsgo virtual document
    /// 2. locate the default export
    /// 3. extract the props type (TS `Props`, `defineProps<…>()`, or
    ///    React.FC `<Props>`)
    /// 4. enumerate prop members, looking up their type strings,
    ///    optionality, JSDoc, and declaration locations
    #[allow(clippy::unused_async)] // The follow-up PR replaces the body with real async work.
    pub async fn resolve_component_props(
        &self,
        component_file: &Path,
    ) -> Result<ResolvedComponent, Error> {
        if !component_file.is_absolute() {
            return Err(Error::InvalidComponentPath {
                path: component_file.to_path_buf(),
                reason: "expected an absolute path".into(),
            });
        }
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod tests;
