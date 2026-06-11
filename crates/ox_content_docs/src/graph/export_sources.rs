use oxc_ast::ast::{ImportOrExportKind, ModuleExportName};
use rustc_hash::FxHashSet;

use super::resolver::{ExternalModuleRef, ResolvedModuleRef};
use super::{ExportKind, ExportSource, GraphError, PublicExport};
use crate::string_builder::{join4, StringBuilder};

pub(super) fn reexport_module_export(
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

pub(super) fn export_source_from_resolved(
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

pub(super) fn externalize_source(
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

pub(super) fn specifier_kind(
    statement_kind: ExportKind,
    specifier_kind: ImportOrExportKind,
) -> ExportKind {
    if statement_kind == ExportKind::Type || specifier_kind == ImportOrExportKind::Type {
        ExportKind::Type
    } else {
        ExportKind::Value
    }
}

pub(super) fn export_kind(kind: ImportOrExportKind) -> ExportKind {
    match kind {
        ImportOrExportKind::Value => ExportKind::Value,
        ImportOrExportKind::Type => ExportKind::Type,
    }
}

pub(super) fn module_export_name(name: &ModuleExportName<'_>) -> String {
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

pub(super) fn dedupe_exports(exports: Vec<PublicExport>) -> Result<Vec<PublicExport>, GraphError> {
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
