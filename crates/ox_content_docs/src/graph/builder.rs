use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::{FxHashMap, FxHashSet};

use super::error::GraphError;
use super::export_sources::{
    dedupe_exports, export_kind, export_source_from_resolved, module_export_name,
    reexport_module_export,
};
use super::exports::{
    append_declaration_exports, append_external_reexports, collect_import_bindings,
};
use super::model::{ExportKind, ExportSource, PublicExport, ResolvedModule};
use super::resolver::{ModuleResolver, ResolvedModuleRef};
use super::util::{absolutize, external_package_name, normalize_existing_path};
#[allow(unused_imports)]
use crate::profile_span;
use crate::{DocExtractor, DocItem};

#[derive(Debug, Clone)]
pub(super) struct ImportBinding {
    pub(super) specifier: String,
    pub(super) imported_name: String,
    pub(super) type_only: bool,
}

pub(super) struct GraphBuilder {
    pub(super) root: PathBuf,
    pub(super) resolver: ModuleResolver,
    pub(super) modules: FxHashMap<PathBuf, ResolvedModule>,
    pub(super) active: FxHashSet<PathBuf>,
    /// When set, doc items are extracted from each module's already-parsed AST
    /// during the walk and stashed in `docs`, so the doc-extraction phase reuses
    /// them instead of parsing every module a second time. `None` for the
    /// standalone `build_export_graph` (exports only).
    pub(super) doc_extractor: Option<DocExtractor>,
    pub(super) docs: FxHashMap<PathBuf, Vec<DocItem>>,
}

impl GraphBuilder {
    pub(super) fn entrypoint_path(&self, path: &Path) -> Result<PathBuf, GraphError> {
        let path = absolutize(&self.root, path);
        std::fs::canonicalize(&path).map_err(|source| GraphError::Read { path, source })
    }

    pub(super) fn collect_module_exports(
        &mut self,
        path: &Path,
    ) -> Result<Vec<PublicExport>, GraphError> {
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
                            self.collect_export_all_from_resolved(&mut exports, &resolved, kind)?;
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

    fn collect_export_all_from_resolved(
        &mut self,
        exports: &mut Vec<PublicExport>,
        resolved: &ResolvedModuleRef,
        kind: ExportKind,
    ) -> Result<(), GraphError> {
        let module_exports = self.collect_module_exports(&resolved.path)?;
        exports.extend(
            module_exports
                .into_iter()
                .filter(|export| export.name != "default")
                .map(|export| reexport_module_export(export, resolved, kind)),
        );
        Ok(())
    }
}
