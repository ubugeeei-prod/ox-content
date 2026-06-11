use ox_content_docs::{
    DocsDiagnostic, DocsDiagnosticCode, ExportGraph, ExportKind, ExportSource, PublicExport,
};

use crate::{
    JsDocsDiagnostic, JsEntrypointModule, JsExportGraph, JsExportSource, JsPublicExport,
    JsResolvedModule,
};

use super::paths::path_to_string;

fn map_export_kind(kind: ExportKind) -> String {
    match kind {
        ExportKind::Value => "value",
        ExportKind::Type => "type",
        ExportKind::ValueAndType => "valueAndType",
        ExportKind::Namespace => "namespace",
        ExportKind::Default => "default",
    }
    .to_string()
}

fn map_export_source(source: ExportSource) -> JsExportSource {
    match source {
        ExportSource::Local { module, original_name } => JsExportSource {
            kind: "local".to_string(),
            module: Some(path_to_string(&module)),
            package: None,
            specifier: None,
            original_name,
            type_only: false,
        },
        ExportSource::External { package, specifier, module, original_name, type_only } => {
            JsExportSource {
                kind: "external".to_string(),
                module: module.as_ref().map(|module| path_to_string(module)),
                package: Some(package),
                specifier: (!specifier.is_empty()).then_some(specifier),
                original_name,
                type_only,
            }
        }
    }
}

pub(super) fn map_public_export(export: PublicExport) -> JsPublicExport {
    JsPublicExport {
        name: export.name,
        kind: map_export_kind(export.kind),
        source: map_export_source(export.source),
    }
}

fn map_docs_diagnostic_code(code: DocsDiagnosticCode) -> String {
    match code {
        DocsDiagnosticCode::FilteredByVisibility => "filteredByVisibility",
        DocsDiagnosticCode::MissingDeclaration => "missingDeclaration",
        DocsDiagnosticCode::UnsupportedExport => "unsupportedExport",
        DocsDiagnosticCode::UnresolvedExternal => "unresolvedExternal",
    }
    .to_string()
}

pub(super) fn map_docs_diagnostic(diagnostic: DocsDiagnostic) -> JsDocsDiagnostic {
    JsDocsDiagnostic {
        code: map_docs_diagnostic_code(diagnostic.code),
        entrypoint: diagnostic.entrypoint,
        export_name: diagnostic.export_name,
        export_kind: map_export_kind(diagnostic.export_kind),
        source: map_export_source(diagnostic.source),
        message: diagnostic.message,
    }
}

pub(super) fn map_export_graph(graph: ExportGraph) -> JsExportGraph {
    JsExportGraph {
        entrypoints: graph
            .entrypoints
            .into_iter()
            .map(|entrypoint| JsEntrypointModule {
                name: entrypoint.name,
                source_path: path_to_string(&entrypoint.source_path),
                exports: entrypoint.exports.into_iter().map(map_public_export).collect(),
            })
            .collect(),
        modules: graph
            .modules
            .into_values()
            .map(|module| JsResolvedModule {
                path: path_to_string(&module.path),
                exports: module.exports.into_iter().map(map_public_export).collect(),
            })
            .collect(),
    }
}
