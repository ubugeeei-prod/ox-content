//! Public API export graph extraction for generated documentation.

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier,
    ImportOrExportKind, ModuleExportName, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

mod docs;
mod error;
mod model;
mod options;
mod resolver;

use docs::{
    docs_diagnostic, export_entrypoint_message, filtered_visibility_reason, is_dependency_source,
    normalized_entries_for_module, resolve_entrypoint_module_metadata,
};
pub use error::GraphError;
pub use model::{
    DocsDiagnostic, DocsDiagnosticCode, EntrypointDocsModule, EntrypointModule, ExportGraph,
    ExportKind, ExportSource, PublicExport, ResolvedModule,
};
pub use options::{
    EntryPointDocsOptions, EntryPointSpec, ExternalDocsOptions, ExternalPackageSource, GraphOptions,
};
use resolver::{ExternalModuleRef, ModuleResolver, ResolvedModuleRef};

#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, join4, StringBuilder};
#[cfg(test)]
use crate::NormalizedDocKind;
use crate::{DocExtractor, DocItem, NormalizedDocEntry};

/// Builds an export graph from entry points.
///
/// Local re-exports are followed recursively. External package re-exports are
/// preserved as metadata and are not resolved into declaration docs.
pub fn build_export_graph(
    entrypoints: &[EntryPointSpec],
    options: &GraphOptions,
) -> Result<ExportGraph, GraphError> {
    Ok(build_export_graph_inner(entrypoints, options, None)?.0)
}

/// Shared implementation behind [`build_export_graph`] and
/// [`extract_docs_from_entry_points`].
///
/// When `doc_extractor` is `Some`, doc items are extracted from each module's
/// already-parsed AST during the walk and returned in the second tuple element,
/// keyed by normalized path, so the extraction phase can reuse them instead of
/// re-parsing every module.
fn build_export_graph_inner(
    entrypoints: &[EntryPointSpec],
    options: &GraphOptions,
    doc_extractor: Option<DocExtractor>,
) -> Result<(ExportGraph, FxHashMap<PathBuf, Vec<DocItem>>), GraphError> {
    profile_span!("docs::build_export_graph");
    let root = graph_root(options);
    let resolver = ModuleResolver::new(&root, options);
    let mut builder = GraphBuilder {
        root,
        resolver,
        modules: FxHashMap::with_hasher(FxBuildHasher),
        active: FxHashSet::default(),
        doc_extractor,
        docs: FxHashMap::with_hasher(FxBuildHasher),
    };

    let mut graph_entrypoints = Vec::with_capacity(entrypoints.len());
    for entrypoint in entrypoints {
        let source_path = builder.entrypoint_path(&entrypoint.path)?;
        let name = entrypoint.name.clone().unwrap_or_else(|| module_name_from_path(&source_path));
        let exports = builder.collect_module_exports(&source_path)?;
        graph_entrypoints.push(EntrypointModule { name, source_path, exports });
    }

    Ok((ExportGraph { entrypoints: graph_entrypoints, modules: builder.modules }, builder.docs))
}

/// Extracts normalized docs grouped by public entry points.
pub fn extract_docs_from_entry_points(
    entrypoints: &[EntryPointSpec],
    options: &EntryPointDocsOptions,
) -> Result<Vec<EntrypointDocsModule>, GraphError> {
    profile_span!("docs::extract_entry_points");
    // Build the export graph and extract docs from the same parse in one walk,
    // so each reachable module is parsed once here instead of again below.
    let (graph, mut walk_docs) = build_export_graph_inner(
        entrypoints,
        &options.graph,
        Some(DocExtractor::for_entrypoint_exports(
            options.include_private,
            options.include_internal,
        )),
    )?;
    let extractor =
        DocExtractor::for_entrypoint_exports(options.include_private, options.include_internal);
    let all_visibility_extractor = DocExtractor::for_entrypoint_exports(true, true);
    let mut docs_cache: FxHashMap<PathBuf, Vec<NormalizedDocEntry>> =
        FxHashMap::with_hasher(FxBuildHasher);
    let mut all_docs_cache: FxHashMap<PathBuf, Vec<NormalizedDocEntry>> =
        FxHashMap::with_hasher(FxBuildHasher);
    let mut modules = Vec::with_capacity(graph.entrypoints.len());

    for entrypoint in graph.entrypoints {
        let mut entries = Vec::new();
        let mut diagnostics = Vec::new();
        let mut seen = FxHashSet::default();

        for export in &entrypoint.exports {
            let (module, original_name) = match &export.source {
                ExportSource::Local { module, original_name }
                | ExportSource::External { module: Some(module), original_name, .. } => {
                    (module, original_name)
                }
                ExportSource::External { .. } => {
                    diagnostics.push(docs_diagnostic(
                        DocsDiagnosticCode::UnresolvedExternal,
                        &entrypoint.name,
                        export,
                        export_entrypoint_message(
                            &export.name,
                            &entrypoint.name,
                            " was not documented because its external source could not be resolved",
                        ),
                    ));
                    continue;
                }
            };
            if original_name == "*" {
                diagnostics.push(docs_diagnostic(
                    DocsDiagnosticCode::UnsupportedExport,
                    &entrypoint.name,
                    export,
                    export_entrypoint_message(
                        &export.name,
                        &entrypoint.name,
                        " was not documented because namespace exports are not emitted as docs entries",
                    ),
                ));
                continue;
            }

            let matched = {
                let module_entries = normalized_entries_for_module(
                    &mut docs_cache,
                    Some(&mut walk_docs),
                    &extractor,
                    module,
                    options.type_parameters,
                )?;
                let mut matched = false;
                for entry in module_entries.iter().filter(|entry| entry.name == *original_name) {
                    matched = true;
                    let mut key =
                        StringBuilder::with_capacity(export.name.len() + entry.file.len() + 2 + 20);
                    key.push_str(&export.name);
                    key.push_char('\0');
                    key.push_str(&entry.file);
                    key.push_char('\0');
                    key.push_usize(entry.line as usize);
                    let key = key.into_string();
                    if !seen.insert(key) {
                        continue;
                    }
                    let mut entry = entry.clone();
                    entry.name.clone_from(&export.name);
                    if is_dependency_source(module) {
                        // Source lives in an installed dependency (under a
                        // node_modules directory): drop the absolute path so we emit
                        // no "View source" link and never leak a local absolute path
                        // (matches TypeDoc's inlined external symbols). Workspace
                        // sources resolved inside the repo keep their path.
                        entry.file = String::new();
                    }
                    entries.push(entry);
                }
                matched
            };

            if matched {
                continue;
            }

            let all_module_entries = normalized_entries_for_module(
                &mut all_docs_cache,
                None,
                &all_visibility_extractor,
                module,
                options.type_parameters,
            )?;
            if let Some(hidden_entry) =
                all_module_entries.iter().find(|entry| entry.name == *original_name)
            {
                if let Some(reason) = filtered_visibility_reason(
                    hidden_entry,
                    options.include_private,
                    options.include_internal,
                ) {
                    let suffix = join2(" was excluded from docs because it is marked ", reason);
                    diagnostics.push(docs_diagnostic(
                        DocsDiagnosticCode::FilteredByVisibility,
                        &entrypoint.name,
                        export,
                        export_entrypoint_message(&export.name, &entrypoint.name, &suffix),
                    ));
                    continue;
                }
            }

            diagnostics.push(docs_diagnostic(
                DocsDiagnosticCode::MissingDeclaration,
                &entrypoint.name,
                export,
                export_entrypoint_message(
                    &export.name,
                    &entrypoint.name,
                    " was not documented because no matching declaration was extracted",
                ),
            ));
        }

        // The entry file's own module-level `@module` / leading JSDoc is emitted
        // by the extractor as a `Module`-kind entry but is never an export, so it
        // is dropped from `entries` above. Pull it out of the entry file's
        // normalized items and carry it as the module description.
        let module_metadata = resolve_entrypoint_module_metadata(
            &entrypoint.name,
            normalized_entries_for_module(
                &mut docs_cache,
                Some(&mut walk_docs),
                &extractor,
                &entrypoint.source_path,
                options.type_parameters,
            )?,
        );

        modules.push(EntrypointDocsModule {
            file: module_metadata.name.clone(),
            name: module_metadata.name,
            source_path: entrypoint.source_path,
            description: module_metadata.description,
            examples: module_metadata.examples,
            tags: module_metadata.tags,
            entries,
            exports: entrypoint.exports,
            diagnostics,
        });
    }

    Ok(modules)
}

#[derive(Debug, Clone)]
struct ImportBinding {
    specifier: String,
    imported_name: String,
    type_only: bool,
}

struct GraphBuilder {
    root: PathBuf,
    resolver: ModuleResolver,
    modules: FxHashMap<PathBuf, ResolvedModule>,
    active: FxHashSet<PathBuf>,
    /// When set, doc items are extracted from each module's already-parsed AST
    /// during the walk and stashed in `docs`, so the doc-extraction phase reuses
    /// them instead of parsing every module a second time. `None` for the
    /// standalone `build_export_graph` (exports only).
    doc_extractor: Option<DocExtractor>,
    docs: FxHashMap<PathBuf, Vec<DocItem>>,
}

impl GraphBuilder {
    fn entrypoint_path(&self, path: &Path) -> Result<PathBuf, GraphError> {
        let path = absolutize(&self.root, path);
        std::fs::canonicalize(&path).map_err(|source| GraphError::Read { path, source })
    }

    fn collect_module_exports(&mut self, path: &Path) -> Result<Vec<PublicExport>, GraphError> {
        profile_span!("docs::collect_exports");
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
        let ret = {
            profile_span!("docs::graph_oxc_parse");
            Parser::new(&allocator, source, source_type).parse()
        };
        if !ret.errors.is_empty() {
            let message = ret
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(GraphError::Parse { path: path.to_path_buf(), message });
        }

        // Extract docs from this same AST when requested, so the doc-extraction
        // phase reads them from the cache rather than parsing the file again.
        // Disjoint field borrows: `&self.doc_extractor` then `&mut self.docs`.
        if let Some(items) = self.doc_extractor.as_ref().map(|extractor| {
            extractor.extract_items_from_program(source, &path.to_string_lossy(), &ret.program)
        }) {
            self.docs.insert(path.to_path_buf(), items);
        }

        let mut exports = Vec::new();
        let imports = collect_import_bindings(&ret.program.body);
        for statement in &ret.program.body {
            match statement {
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(declaration) = &export.declaration {
                        append_declaration_exports(&mut exports, path, declaration);
                    }

                    if let Some(source) = &export.source {
                        let specifier = source.value.to_string();
                        let kind = export_kind(export.export_kind);
                        if let Some(resolved) = self.resolver.resolve_specifier(path, &specifier)? {
                            self.append_reexports_from_resolved_module(
                                &mut exports,
                                &resolved,
                                &export.specifiers,
                                kind,
                            )?;
                        } else {
                            append_external_reexports(
                                &mut exports,
                                &specifier,
                                None,
                                &export.specifiers,
                                kind,
                            );
                        }
                    } else {
                        self.append_local_specifier_exports(
                            &mut exports,
                            path,
                            &imports,
                            &export.specifiers,
                            export_kind(export.export_kind),
                        )?;
                    }
                }
                Statement::ExportAllDeclaration(export) => {
                    let specifier = export.source.value.to_string();
                    let kind = export_kind(export.export_kind);
                    if let Some(resolved) = self.resolver.resolve_specifier(path, &specifier)? {
                        if let Some(exported) = &export.exported {
                            exports.push(PublicExport {
                                name: module_export_name(exported),
                                kind: ExportKind::Namespace,
                                source: export_source_from_resolved(
                                    &resolved,
                                    "*".to_string(),
                                    kind == ExportKind::Type,
                                ),
                            });
                        } else {
                            let module_exports = self.collect_module_exports(&resolved.path)?;
                            exports.extend(
                                module_exports
                                    .into_iter()
                                    .filter(|export| export.name != "default")
                                    .map(|export| reexport_module_export(export, &resolved, kind)),
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
                                specifier,
                                module: None,
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

    fn append_reexports_from_resolved_module(
        &mut self,
        exports: &mut Vec<PublicExport>,
        resolved: &ResolvedModuleRef,
        specifiers: &[oxc_ast::ast::ExportSpecifier<'_>],
        statement_kind: ExportKind,
    ) -> Result<(), GraphError> {
        for specifier in specifiers {
            let kind = specifier_kind(statement_kind, specifier.export_kind);
            exports.push(self.public_export_from_resolved_module(
                resolved,
                module_export_name(&specifier.exported),
                module_export_name(&specifier.local),
                kind,
            )?);
        }

        Ok(())
    }

    fn append_local_specifier_exports(
        &mut self,
        exports: &mut Vec<PublicExport>,
        path: &Path,
        imports: &FxHashMap<String, ImportBinding>,
        specifiers: &[oxc_ast::ast::ExportSpecifier<'_>],
        statement_kind: ExportKind,
    ) -> Result<(), GraphError> {
        for specifier in specifiers {
            let local_name = module_export_name(&specifier.local);
            let public_name = module_export_name(&specifier.exported);
            let kind = specifier_kind(statement_kind, specifier.export_kind);

            if let Some(binding) = imports.get(&local_name) {
                let kind = if binding.type_only { ExportKind::Type } else { kind };
                if let Some(resolved) = self.resolver.resolve_specifier(path, &binding.specifier)? {
                    exports.push(self.public_export_from_resolved_module(
                        &resolved,
                        public_name,
                        binding.imported_name.clone(),
                        kind,
                    )?);
                } else if !is_local_specifier(&binding.specifier) {
                    exports.push(PublicExport {
                        name: public_name,
                        kind,
                        source: ExportSource::External {
                            package: external_package_name(&binding.specifier),
                            specifier: binding.specifier.clone(),
                            module: None,
                            original_name: binding.imported_name.clone(),
                            type_only: kind == ExportKind::Type,
                        },
                    });
                }
                continue;
            }

            exports.push(PublicExport {
                name: public_name,
                kind,
                source: ExportSource::Local {
                    module: path.to_path_buf(),
                    original_name: local_name,
                },
            });
        }

        Ok(())
    }

    fn public_export_from_resolved_module(
        &mut self,
        resolved: &ResolvedModuleRef,
        public_name: String,
        original_name: String,
        kind: ExportKind,
    ) -> Result<PublicExport, GraphError> {
        let module_exports = self.collect_module_exports(&resolved.path)?;
        if let Some(export) = module_exports.iter().find(|export| export.name == original_name) {
            let mut export = export.clone();
            export.name = public_name;
            export.kind = kind;
            if let Some(external) = &resolved.external {
                export.source =
                    externalize_source(export.source, external, kind == ExportKind::Type);
            }
            return Ok(export);
        }

        Ok(PublicExport {
            name: public_name,
            kind,
            source: export_source_from_resolved(resolved, original_name, kind == ExportKind::Type),
        })
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

fn append_external_reexports(
    exports: &mut Vec<PublicExport>,
    specifier: &str,
    module: Option<PathBuf>,
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
                specifier: specifier.to_string(),
                module: module.clone(),
                original_name: module_export_name(&export_specifier.local),
                type_only: kind == ExportKind::Type,
            },
        });
    }
}

fn collect_import_bindings(statements: &[Statement<'_>]) -> FxHashMap<String, ImportBinding> {
    let mut imports = FxHashMap::default();

    for statement in statements {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let Some(specifiers) = &import.specifiers else {
            continue;
        };

        let statement_type_only = import.import_kind == ImportOrExportKind::Type;
        for specifier in specifiers {
            let specifier =
                import_binding(import.source.value.as_str(), statement_type_only, specifier);
            imports.insert(specifier.0, specifier.1);
        }
    }

    imports
}

fn import_binding(
    specifier: &str,
    statement_type_only: bool,
    import_specifier: &ImportDeclarationSpecifier<'_>,
) -> (String, ImportBinding) {
    match import_specifier {
        ImportDeclarationSpecifier::ImportSpecifier(import) => (
            import.local.name.to_string(),
            ImportBinding {
                specifier: specifier.to_string(),
                imported_name: module_export_name(&import.imported),
                type_only: statement_type_only || import.import_kind == ImportOrExportKind::Type,
            },
        ),
        ImportDeclarationSpecifier::ImportDefaultSpecifier(import) => (
            import.local.name.to_string(),
            ImportBinding {
                specifier: specifier.to_string(),
                imported_name: "default".to_string(),
                type_only: statement_type_only,
            },
        ),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(import) => (
            import.local.name.to_string(),
            ImportBinding {
                specifier: specifier.to_string(),
                imported_name: "*".to_string(),
                type_only: statement_type_only,
            },
        ),
    }
}

fn reexport_module_export(
    mut export: PublicExport,
    resolved: &ResolvedModuleRef,
    statement_kind: ExportKind,
) -> PublicExport {
    if statement_kind == ExportKind::Type {
        export.kind = ExportKind::Type;
    }

    if let Some(external) = &resolved.external {
        export.source =
            externalize_source(export.source, external, export.kind == ExportKind::Type);
    }

    export
}

fn export_source_from_resolved(
    resolved: &ResolvedModuleRef,
    original_name: String,
    type_only: bool,
) -> ExportSource {
    if let Some(external) = &resolved.external {
        ExportSource::External {
            package: external.package.clone(),
            specifier: external.specifier.clone(),
            module: Some(resolved.path.clone()),
            original_name,
            type_only,
        }
    } else {
        ExportSource::Local { module: resolved.path.clone(), original_name }
    }
}

fn externalize_source(
    source: ExportSource,
    external: &ExternalModuleRef,
    type_only: bool,
) -> ExportSource {
    match source {
        ExportSource::Local { module, original_name } => ExportSource::External {
            package: external.package.clone(),
            specifier: external.specifier.clone(),
            module: Some(module),
            original_name,
            type_only,
        },
        ExportSource::External {
            package,
            specifier,
            module,
            original_name,
            type_only: source_type_only,
        } => ExportSource::External {
            package,
            specifier,
            module,
            original_name,
            type_only: type_only || source_type_only,
        },
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

fn export_kind_key(kind: ExportKind) -> &'static str {
    match kind {
        ExportKind::Value => "value",
        ExportKind::Type => "type",
        ExportKind::ValueAndType => "valueAndType",
        ExportKind::Namespace => "namespace",
        ExportKind::Default => "default",
    }
}

fn dedupe_exports(exports: Vec<PublicExport>) -> Result<Vec<PublicExport>, GraphError> {
    let mut seen = FxHashSet::default();
    let mut deduped = Vec::with_capacity(exports.len());

    for export in exports {
        let source_key = match &export.source {
            ExportSource::Local { module, original_name } => {
                let module = module.to_string_lossy();
                join4("local:", module.as_ref(), ":", original_name)
            }
            ExportSource::External { package, specifier, module, original_name, type_only } => {
                let module = module.as_ref().map(|module| module.to_string_lossy());
                let module = module.as_deref().unwrap_or_default();
                let type_only = if *type_only { "true" } else { "false" };
                let mut key = StringBuilder::with_capacity(
                    "external:::::".len()
                        + package.len()
                        + specifier.len()
                        + module.len()
                        + original_name.len()
                        + type_only.len(),
                );
                key.push_str("external:");
                key.push_str(package);
                key.push_char(':');
                key.push_str(specifier);
                key.push_char(':');
                key.push_str(module);
                key.push_char(':');
                key.push_str(original_name);
                key.push_char(':');
                key.push_str(type_only);
                key.into_string()
            }
        };
        let mut key = StringBuilder::with_capacity(
            export.name.len() + export_kind_key(export.kind).len() + source_key.len() + 2,
        );
        key.push_str(&export.name);
        key.push_char(':');
        key.push_str(export_kind_key(export.kind));
        key.push_char(':');
        key.push_str(&source_key);
        let key = key.into_string();
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
            return join4("@", scope, "/", package);
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
mod tests;
