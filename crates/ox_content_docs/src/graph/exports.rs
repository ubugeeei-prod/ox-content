use std::path::{Path, PathBuf};

use oxc_ast::ast::{
    BindingPattern, Declaration, ImportDeclarationSpecifier, ImportOrExportKind, Statement,
};
use rustc_hash::FxHashMap;

use super::export_sources::{module_export_name, specifier_kind};
use super::{external_package_name, ExportKind, ExportSource, ImportBinding, PublicExport};

pub(super) fn append_declaration_exports(
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

pub(super) fn append_external_reexports(
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

pub(super) fn collect_import_bindings(
    statements: &[Statement<'_>],
) -> FxHashMap<String, ImportBinding> {
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
