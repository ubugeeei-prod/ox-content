use std::path::Path;

use rustc_hash::FxHashMap;

use super::builder::{GraphBuilder, ImportBinding};
use super::error::GraphError;
use super::export_sources::{
    export_source_from_resolved, externalize_source, module_export_name, specifier_kind,
};
use super::model::{ExportKind, ExportSource, PublicExport};
use super::resolver::ResolvedModuleRef;
use super::util::{external_package_name, is_local_specifier};

impl GraphBuilder {
    pub(super) fn append_reexports_from_resolved_module(
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

    pub(super) fn append_local_specifier_exports(
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
