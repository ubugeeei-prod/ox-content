//! Public API export graph extraction for generated documentation.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, ImportOrExportKind,
    ModuleExportName, Statement,
};
use oxc_parser::Parser;
use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
use oxc_span::SourceType;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{normalize_doc_items, DocExtractor, ExtractError, NormalizedDocEntry};

/// Entry point used to group generated API docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointSpec {
    /// Source file path.
    pub path: PathBuf,
    /// Public module name. Defaults to the file stem.
    pub name: Option<String>,
}

/// Export graph resolution options.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphOptions {
    /// Root used to resolve relative entry point paths.
    pub root: Option<PathBuf>,
    /// Optional TypeScript config for path alias resolution.
    pub tsconfig: Option<PathBuf>,
}

/// Options for extracting docs grouped by entry point.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointDocsOptions {
    /// Export graph options.
    pub graph: GraphOptions,
    /// Include `@private` docs.
    pub include_private: bool,
    /// Include `@internal` docs.
    pub include_internal: bool,
}

/// Resolved export graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportGraph {
    /// Public entry point modules.
    pub entrypoints: Vec<EntrypointModule>,
    /// Resolved modules keyed by absolute source path.
    pub modules: FxHashMap<PathBuf, ResolvedModule>,
}

/// Public entry point module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrypointModule {
    /// Public module name.
    pub name: String,
    /// Source file path.
    pub source_path: PathBuf,
    /// Public exports reachable from this entry point.
    pub exports: Vec<PublicExport>,
}

/// Resolved source module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedModule {
    /// Source file path.
    pub path: PathBuf,
    /// Exports declared or re-exported by this module.
    pub exports: Vec<PublicExport>,
}

/// Public export metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicExport {
    /// Public export name after aliasing.
    pub name: String,
    /// Export kind.
    pub kind: ExportKind,
    /// Export source.
    pub source: ExportSource,
}

/// Export kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportKind {
    /// Runtime value export.
    Value,
    /// Type-only export.
    Type,
    /// Export available as both value and type.
    ValueAndType,
    /// Namespace export.
    Namespace,
    /// Default export.
    Default,
}

/// Export source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportSource {
    /// Export resolved to a local source file.
    Local {
        /// Source module path.
        module: PathBuf,
        /// Original exported name in the source module.
        original_name: String,
    },
    /// Export from an external package.
    External {
        /// Package name.
        package: String,
        /// Original exported name.
        original_name: String,
        /// Whether the export is type-only.
        type_only: bool,
    },
}

/// Docs grouped by a public entry point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrypointDocsModule {
    /// Public module name.
    pub name: String,
    /// Virtual docs module file used by Markdown generation.
    pub file: String,
    /// Source file path.
    pub source_path: PathBuf,
    /// Normalized docs entries for reachable local exports.
    pub entries: Vec<NormalizedDocEntry>,
    /// Public export metadata, including external re-exports.
    pub exports: Vec<PublicExport>,
}

/// Export graph error.
#[derive(Debug, Error)]
pub enum GraphError {
    /// IO error while reading a module.
    #[error("failed to read {path}: {source}")]
    Read {
        /// Path that failed to read.
        path: PathBuf,
        /// Source error.
        #[source]
        source: std::io::Error,
    },
    /// Parser error.
    #[error("failed to parse {path}: {message}")]
    Parse {
        /// Path that failed to parse.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// Resolver error.
    #[error("failed to resolve {specifier} from {importer}: {message}")]
    Resolve {
        /// Importer path.
        importer: PathBuf,
        /// Module specifier.
        specifier: String,
        /// Resolver message.
        message: String,
    },
    /// Documentation extraction error.
    #[error("failed to extract docs from {path}: {source}")]
    Extract {
        /// Path that failed to extract.
        path: PathBuf,
        /// Source error.
        #[source]
        source: ExtractError,
    },
}

/// Builds an export graph from entry points.
///
/// Local re-exports are followed recursively. External package re-exports are
/// preserved as metadata and are not resolved into declaration docs.
pub fn build_export_graph(
    entrypoints: &[EntryPointSpec],
    options: &GraphOptions,
) -> Result<ExportGraph, GraphError> {
    let root = graph_root(options);
    let resolver = ModuleResolver::new(&root, options);
    let mut builder = GraphBuilder {
        root,
        resolver,
        modules: FxHashMap::with_hasher(FxBuildHasher),
        active: FxHashSet::default(),
    };

    let mut graph_entrypoints = Vec::with_capacity(entrypoints.len());
    for entrypoint in entrypoints {
        let source_path = builder.entrypoint_path(&entrypoint.path)?;
        let name = entrypoint.name.clone().unwrap_or_else(|| module_name_from_path(&source_path));
        let exports = builder.collect_module_exports(&source_path)?;
        graph_entrypoints.push(EntrypointModule { name, source_path, exports });
    }

    Ok(ExportGraph { entrypoints: graph_entrypoints, modules: builder.modules })
}

/// Extracts normalized docs grouped by public entry points.
pub fn extract_docs_from_entry_points(
    entrypoints: &[EntryPointSpec],
    options: &EntryPointDocsOptions,
) -> Result<Vec<EntrypointDocsModule>, GraphError> {
    let graph = build_export_graph(entrypoints, &options.graph)?;
    let extractor =
        DocExtractor::with_visibility(options.include_private, options.include_internal);
    let mut docs_cache: FxHashMap<PathBuf, Vec<NormalizedDocEntry>> =
        FxHashMap::with_hasher(FxBuildHasher);
    let mut modules = Vec::with_capacity(graph.entrypoints.len());

    for entrypoint in graph.entrypoints {
        let mut entries = Vec::new();
        let mut seen = FxHashSet::default();

        for export in &entrypoint.exports {
            let ExportSource::Local { module, original_name } = &export.source else {
                continue;
            };
            if original_name == "*" {
                continue;
            }

            if !docs_cache.contains_key(module) {
                let items = extractor
                    .extract_file(module)
                    .map_err(|source| GraphError::Extract { path: module.clone(), source })?;
                docs_cache.insert(module.clone(), normalize_doc_items(items));
            }

            let Some(module_entries) = docs_cache.get(module) else {
                continue;
            };
            for entry in module_entries.iter().filter(|entry| entry.name == *original_name) {
                let key = format!("{}\0{}\0{}", export.name, entry.file, entry.line);
                if !seen.insert(key) {
                    continue;
                }
                let mut entry = entry.clone();
                entry.name.clone_from(&export.name);
                entries.push(entry);
            }
        }

        modules.push(EntrypointDocsModule {
            file: entrypoint.name.clone(),
            name: entrypoint.name,
            source_path: entrypoint.source_path,
            entries,
            exports: entrypoint.exports,
        });
    }

    Ok(modules)
}

struct ModuleResolver {
    resolver: Resolver,
}

impl ModuleResolver {
    fn new(root: &Path, options: &GraphOptions) -> Self {
        let mut resolve_options = ResolveOptions {
            extensions: vec![
                ".ts".to_string(),
                ".tsx".to_string(),
                ".mts".to_string(),
                ".cts".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
                ".mjs".to_string(),
                ".cjs".to_string(),
                ".json".to_string(),
                ".node".to_string(),
            ],
            extension_alias: vec![
                (".js".to_string(), vec![".ts".to_string(), ".tsx".to_string(), ".js".to_string()]),
                (".mjs".to_string(), vec![".mts".to_string(), ".mjs".to_string()]),
                (".cjs".to_string(), vec![".cts".to_string(), ".cjs".to_string()]),
            ],
            condition_names: vec![
                "types".to_string(),
                "import".to_string(),
                "module".to_string(),
                "default".to_string(),
            ],
            main_fields: vec!["types".to_string(), "module".to_string(), "main".to_string()],
            ..ResolveOptions::default()
        };

        if let Some(tsconfig) = &options.tsconfig {
            resolve_options.tsconfig = Some(TsconfigOptions {
                config_file: absolutize(root, tsconfig),
                references: TsconfigReferences::Auto,
            });
        }

        Self { resolver: Resolver::new(resolve_options) }
    }

    fn resolve_local(
        &self,
        importer: &Path,
        specifier: &str,
    ) -> Result<Option<PathBuf>, GraphError> {
        if !is_local_specifier(specifier) {
            return Ok(None);
        }

        let directory = importer.parent().unwrap_or_else(|| Path::new("."));
        self.resolver
            .resolve(directory, specifier)
            .map(|resolution| Some(normalize_existing_path(resolution.path())))
            .map_err(|error| GraphError::Resolve {
                importer: importer.to_path_buf(),
                specifier: specifier.to_string(),
                message: error.to_string(),
            })
    }
}

struct GraphBuilder {
    root: PathBuf,
    resolver: ModuleResolver,
    modules: FxHashMap<PathBuf, ResolvedModule>,
    active: FxHashSet<PathBuf>,
}

impl GraphBuilder {
    fn entrypoint_path(&self, path: &Path) -> Result<PathBuf, GraphError> {
        let path = absolutize(&self.root, path);
        std::fs::canonicalize(&path).map_err(|source| GraphError::Read { path, source })
    }

    fn collect_module_exports(&mut self, path: &Path) -> Result<Vec<PublicExport>, GraphError> {
        let path = normalize_existing_path(path);
        if let Some(module) = self.modules.get(&path) {
            return Ok(module.exports.clone());
        }
        if !self.active.insert(path.clone()) {
            return Ok(Vec::new());
        }

        let source = std::fs::read_to_string(&path)
            .map_err(|source| GraphError::Read { path: path.clone(), source })?;
        let exports = self.collect_source_exports(&path, &source)?;

        self.active.remove(&path);
        self.modules
            .insert(path.clone(), ResolvedModule { path: path.clone(), exports: exports.clone() });
        Ok(exports)
    }

    fn collect_source_exports(
        &mut self,
        path: &Path,
        source: &str,
    ) -> Result<Vec<PublicExport>, GraphError> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap_or_default();
        let ret = Parser::new(&allocator, source, source_type).parse();
        if !ret.errors.is_empty() {
            let message = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(GraphError::Parse { path: path.to_path_buf(), message });
        }

        let mut exports = Vec::new();
        for statement in &ret.program.body {
            match statement {
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = &export.declaration {
                        append_declaration_exports(&mut exports, path, declaration);
                    }

                    if let Some(source) = &export.source {
                        let specifier = source.value.to_string();
                        let kind = export_kind(export.export_kind);
                        if let Some(module) = self.resolver.resolve_local(path, &specifier)? {
                            append_reexports_from_specifiers(
                                &mut exports,
                                &module,
                                &export.specifiers,
                                kind,
                            );
                        } else {
                            append_external_reexports(
                                &mut exports,
                                &specifier,
                                &export.specifiers,
                                kind,
                            );
                        }
                    } else {
                        append_local_specifier_exports(
                            &mut exports,
                            path,
                            &export.specifiers,
                            export_kind(export.export_kind),
                        );
                    }
                }
                Statement::ExportAllDeclaration(export) => {
                    let specifier = export.source.value.to_string();
                    let kind = export_kind(export.export_kind);
                    if let Some(module) = self.resolver.resolve_local(path, &specifier)? {
                        if let Some(exported) = &export.exported {
                            exports.push(PublicExport {
                                name: module_export_name(exported),
                                kind: ExportKind::Namespace,
                                source: ExportSource::Local {
                                    module,
                                    original_name: "*".to_string(),
                                },
                            });
                        } else {
                            exports.extend(
                                self.collect_module_exports(&module)?
                                    .into_iter()
                                    .filter(|export| export.name != "default"),
                            );
                        }
                    } else {
                        exports.push(PublicExport {
                            name: export
                                .exported
                                .as_ref()
                                .map_or_else(|| "*".to_string(), module_export_name),
                            kind,
                            source: ExportSource::External {
                                package: external_package_name(&specifier),
                                original_name: "*".to_string(),
                                type_only: kind == ExportKind::Type,
                            },
                        });
                    }
                }
                Statement::ExportDefaultDeclaration(export) => {
                    let original_name = match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
                            .id
                            .as_ref()
                            .map_or_else(|| "default".to_string(), |id| id.name.to_string()),
                        ExportDefaultDeclarationKind::ClassDeclaration(class) => class
                            .id
                            .as_ref()
                            .map_or_else(|| "default".to_string(), |id| id.name.to_string()),
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(interface) => {
                            interface.id.name.to_string()
                        }
                        _ => "default".to_string(),
                    };
                    exports.push(PublicExport {
                        name: "default".to_string(),
                        kind: ExportKind::Default,
                        source: ExportSource::Local { module: path.to_path_buf(), original_name },
                    });
                }
                _ => {}
            }
        }

        dedupe_exports(exports)
    }
}

fn append_declaration_exports(
    exports: &mut Vec<PublicExport>,
    path: &Path,
    declaration: &Declaration<'_>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                append_local_export(exports, path, id.name.as_str(), ExportKind::Value);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                append_local_export(exports, path, id.name.as_str(), ExportKind::Value);
            }
        }
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    append_local_export(exports, path, id.name.as_str(), ExportKind::Value);
                }
            }
        }
        Declaration::TSTypeAliasDeclaration(type_alias) => {
            append_local_export(exports, path, type_alias.id.name.as_str(), ExportKind::Type);
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            append_local_export(exports, path, interface.id.name.as_str(), ExportKind::Type);
        }
        Declaration::TSEnumDeclaration(enum_decl) => {
            append_local_export(
                exports,
                path,
                enum_decl.id.name.as_str(),
                ExportKind::ValueAndType,
            );
        }
        _ => {}
    }
}

fn append_local_export(exports: &mut Vec<PublicExport>, path: &Path, name: &str, kind: ExportKind) {
    exports.push(PublicExport {
        name: name.to_string(),
        kind,
        source: ExportSource::Local { module: path.to_path_buf(), original_name: name.to_string() },
    });
}

fn append_reexports_from_specifiers(
    exports: &mut Vec<PublicExport>,
    module: &Path,
    specifiers: &[oxc_ast::ast::ExportSpecifier<'_>],
    statement_kind: ExportKind,
) {
    for specifier in specifiers {
        let kind = specifier_kind(statement_kind, specifier.export_kind);
        exports.push(PublicExport {
            name: module_export_name(&specifier.exported),
            kind,
            source: ExportSource::Local {
                module: module.to_path_buf(),
                original_name: module_export_name(&specifier.local),
            },
        });
    }
}

fn append_local_specifier_exports(
    exports: &mut Vec<PublicExport>,
    path: &Path,
    specifiers: &[oxc_ast::ast::ExportSpecifier<'_>],
    statement_kind: ExportKind,
) {
    for specifier in specifiers {
        let kind = specifier_kind(statement_kind, specifier.export_kind);
        exports.push(PublicExport {
            name: module_export_name(&specifier.exported),
            kind,
            source: ExportSource::Local {
                module: path.to_path_buf(),
                original_name: module_export_name(&specifier.local),
            },
        });
    }
}

fn append_external_reexports(
    exports: &mut Vec<PublicExport>,
    specifier: &str,
    specifiers: &[oxc_ast::ast::ExportSpecifier<'_>],
    statement_kind: ExportKind,
) {
    let package = external_package_name(specifier);
    for export_specifier in specifiers {
        let kind = specifier_kind(statement_kind, export_specifier.export_kind);
        exports.push(PublicExport {
            name: module_export_name(&export_specifier.exported),
            kind,
            source: ExportSource::External {
                package: package.clone(),
                original_name: module_export_name(&export_specifier.local),
                type_only: kind == ExportKind::Type,
            },
        });
    }
}

fn specifier_kind(statement_kind: ExportKind, specifier_kind: ImportOrExportKind) -> ExportKind {
    if statement_kind == ExportKind::Type || specifier_kind == ImportOrExportKind::Type {
        ExportKind::Type
    } else {
        ExportKind::Value
    }
}

fn export_kind(kind: ImportOrExportKind) -> ExportKind {
    match kind {
        ImportOrExportKind::Value => ExportKind::Value,
        ImportOrExportKind::Type => ExportKind::Type,
    }
}

fn module_export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(identifier) => identifier.name.to_string(),
        ModuleExportName::IdentifierReference(identifier) => identifier.name.to_string(),
        ModuleExportName::StringLiteral(literal) => literal.value.to_string(),
    }
}

fn dedupe_exports(exports: Vec<PublicExport>) -> Result<Vec<PublicExport>, GraphError> {
    let mut seen = FxHashSet::default();
    let mut deduped = Vec::with_capacity(exports.len());

    for export in exports {
        let source_key = match &export.source {
            ExportSource::Local { module, original_name } => {
                format!("local:{}:{original_name}", module.display())
            }
            ExportSource::External { package, original_name, type_only } => {
                format!("external:{package}:{original_name}:{type_only}")
            }
        };
        let key = format!("{}:{:?}:{source_key}", export.name, export.kind);
        if seen.insert(key) {
            deduped.push(export);
        }
    }

    Ok(deduped)
}

fn graph_root(options: &GraphOptions) -> PathBuf {
    options.root.as_ref().map_or_else(
        || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        PathBuf::from,
    )
}

fn module_name_from_path(path: &Path) -> String {
    path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("module").to_string()
}

fn is_local_specifier(specifier: &str) -> bool {
    specifier.starts_with('.') || specifier.starts_with('/')
}

fn external_package_name(specifier: &str) -> String {
    if let Some(rest) = specifier.strip_prefix('@') {
        let mut segments = rest.split('/');
        let scope = segments.next().unwrap_or_default();
        let package = segments.next().unwrap_or_default();
        if !scope.is_empty() && !package.is_empty() {
            return format!("@{scope}/{package}");
        }
    }

    specifier.split('/').next().unwrap_or(specifier).to_string()
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn builds_export_graph_and_extracts_entrypoint_docs() {
        let root = temp_root();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/index.ts"),
            r"
export { add as sum } from './math';
export type { Options } from './types';
export * from './extra';
export { ExternalThing } from 'external-pkg/subpath';
",
        )
        .unwrap();
        fs::write(
            root.join("src/math.ts"),
            r"
/** Adds two numbers. */
export function add(a: number, b: number): number {
  return a + b;
}
",
        )
        .unwrap();
        fs::write(
            root.join("src/types.ts"),
            r"
/** Runtime options. */
export interface Options {
  value: string;
}
",
        )
        .unwrap();
        fs::write(
            root.join("src/extra.ts"),
            r"
/** Creates a label. */
export function label(value: string): string {
  return value;
}
",
        )
        .unwrap();

        let entrypoints = [EntryPointSpec {
            path: PathBuf::from("src/index.ts"),
            name: Some("default".to_string()),
        }];
        let graph_options = GraphOptions { root: Some(root.clone()), tsconfig: None };

        let graph = build_export_graph(&entrypoints, &graph_options).unwrap();
        assert_eq!(graph.entrypoints[0].name, "default");
        assert!(graph.entrypoints[0].exports.iter().any(|export| export.name == "sum"));
        assert!(graph.entrypoints[0].exports.iter().any(|export| export.name == "Options"));
        assert!(graph.entrypoints[0].exports.iter().any(|export| export.name == "label"));
        assert!(graph.entrypoints[0].exports.iter().any(|export| matches!(
            &export.source,
            ExportSource::External { package, original_name, .. }
                if package == "external-pkg" && original_name == "ExternalThing"
        )));

        let docs = extract_docs_from_entry_points(
            &entrypoints,
            &EntryPointDocsOptions {
                graph: graph_options,
                include_private: false,
                include_internal: false,
            },
        )
        .unwrap();
        let names = docs[0].entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["sum", "Options", "label"]);

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("ox-content-docs-graph-{nanos}"))
    }
}
