use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_docs::{
    build_export_graph, extract_docs_from_directories, extract_docs_from_entry_points,
    generate_docs_data_json, generate_markdown, generate_nav_code, generate_nav_metadata,
    generate_nav_metadata_from_docs_with_options, normalize_doc_items, write_docs_output,
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiTypeParamDoc,
    DocExtractor, DocItem, DocItemKind, DocTag, DocsDiagnostic, DocsDiagnosticCode, DocsNavItem,
    DocsNavMetadataOptions, DocsOutputOptions, EntryPointDocsOptions, EntryPointSpec, ExportGraph,
    ExportKind, ExportSource, ExternalDocsOptions, ExternalPackageSource, ExtractedDocModule,
    GraphOptions, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle,
    MarkdownPathStrategy, MarkdownRenderStyle, MarkdownSingleEntryRoot, NormalizedDocEntry,
    NormalizedMember, NormalizedParamDoc, NormalizedReturnDoc, NormalizedTypeParam, ParamDoc,
    PublicExport,
};

use crate::{
    JsDocEntry, JsDocMember, JsDocParam, JsDocReturn, JsDocsDiagnostic, JsDocsMarkdownEntry,
    JsDocsMarkdownModule, JsDocsMarkdownOptions, JsDocsMarkdownTag, JsDocsNavItem,
    JsDocsNavOptions, JsDocsOutputOptions, JsEntryPointDocsOptions, JsEntryPointSpec,
    JsEntrypointDocsModule, JsEntrypointModule, JsExportGraph, JsExportSource,
    JsExternalPackageSource, JsExtractedDocsModule, JsGraphOptions, JsPublicExport,
    JsResolvedModule, JsSourceDocItem, JsSourceDocParam, JsSourceDocTag, JsTypeParam,
};

fn doc_item_kind_to_string(kind: DocItemKind) -> String {
    match kind {
        DocItemKind::Module => "module",
        DocItemKind::Function => "function",
        DocItemKind::Class => "class",
        DocItemKind::Interface => "interface",
        DocItemKind::Type => "type",
        DocItemKind::Enum => "enum",
        DocItemKind::Variable => "variable",
        DocItemKind::Method => "method",
        DocItemKind::Property => "property",
        DocItemKind::Constructor => "constructor",
        DocItemKind::Getter => "getter",
        DocItemKind::Setter => "setter",
        DocItemKind::EnumMember => "enumMember",
        DocItemKind::IndexSignature => "indexSignature",
    }
    .to_string()
}

fn map_doc_tag(tag: DocTag) -> JsSourceDocTag {
    JsSourceDocTag { tag: tag.tag, value: tag.value }
}

fn map_param_doc(param: ParamDoc) -> JsSourceDocParam {
    JsSourceDocParam {
        name: param.name,
        type_annotation: param.type_annotation,
        optional: param.optional,
        default_value: param.default_value,
        description: param.description,
    }
}

fn map_doc_item(item: DocItem) -> JsSourceDocItem {
    let return_members = (!item.return_members.is_empty())
        .then(|| item.return_members.into_iter().map(map_doc_item).collect());
    let members =
        (!item.children.is_empty()).then(|| item.children.into_iter().map(map_doc_item).collect());

    JsSourceDocItem {
        name: item.name,
        kind: doc_item_kind_to_string(item.kind),
        doc: item.doc,
        jsdoc: item.jsdoc,
        source_path: item.source_path,
        line: item.line,
        end_line: item.end_line,
        exported: item.exported,
        signature: item.signature,
        extends: (!item.extends.is_empty()).then_some(item.extends),
        implements: (!item.implements.is_empty()).then_some(item.implements),
        params: item.params.into_iter().map(map_param_doc).collect(),
        return_type: item.return_type,
        return_members,
        members,
        tags: item.tags.into_iter().map(map_doc_tag).collect(),
    }
}

fn map_normalized_param_doc(param: NormalizedParamDoc) -> JsDocParam {
    JsDocParam {
        name: param.name,
        r#type: param.type_annotation,
        description: param.description,
        optional: param.optional.then_some(true),
        r#default: param.default_value,
    }
}

fn map_normalized_return_doc(return_doc: NormalizedReturnDoc) -> JsDocReturn {
    JsDocReturn {
        r#type: return_doc.type_annotation,
        description: return_doc.description,
        members: (!return_doc.members.is_empty())
            .then(|| return_doc.members.into_iter().map(map_normalized_member).collect()),
    }
}

fn map_normalized_member(member: NormalizedMember) -> JsDocMember {
    JsDocMember {
        name: member.name,
        kind: member.kind.as_str().to_string(),
        description: member.description,
        signature: member.signature,
        r#type: member.type_annotation,
        r#default: member.default_value,
        params: (!member.params.is_empty())
            .then(|| member.params.into_iter().map(map_normalized_param_doc).collect()),
        type_parameters: (!member.type_parameters.is_empty())
            .then(|| member.type_parameters.into_iter().map(map_normalized_type_param).collect()),
        returns: member.returns.map(map_normalized_return_doc),
        members: (!member.members.is_empty())
            .then(|| member.members.into_iter().map(map_normalized_member).collect()),
        optional: member.optional.then_some(true),
        readonly: member.readonly.then_some(true),
        r#static: member.r#static.then_some(true),
        private: member.private.then_some(true),
        tags: (!member.tags.is_empty()).then(|| member.tags.into_iter().collect()),
        implementation_of: None,
        line: member.line,
        end_line: member.end_line,
    }
}

pub fn map_normalized_doc_entry(entry: NormalizedDocEntry) -> JsDocEntry {
    JsDocEntry {
        name: entry.name,
        kind: entry.kind.as_str().to_string(),
        description: entry.description,
        params: (!entry.params.is_empty())
            .then(|| entry.params.into_iter().map(map_normalized_param_doc).collect()),
        returns: entry.returns.map(map_normalized_return_doc),
        examples: (!entry.examples.is_empty()).then_some(entry.examples),
        tags: (!entry.tags.is_empty()).then(|| entry.tags.into_iter().collect()),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: (!entry.extends.is_empty()).then_some(entry.extends),
        implements: (!entry.implements.is_empty()).then_some(entry.implements),
        has_body: entry.has_body,
        members: (!entry.members.is_empty())
            .then(|| entry.members.into_iter().map(map_normalized_member).collect()),
        type_parameters: (!entry.type_parameters.is_empty())
            .then(|| entry.type_parameters.into_iter().map(map_normalized_type_param).collect()),
    }
}

fn map_normalized_type_param(type_param: NormalizedTypeParam) -> JsTypeParam {
    JsTypeParam {
        name: type_param.name,
        constraint: type_param.constraint,
        r#default: type_param.default,
        description: type_param.description,
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn convert_entrypoint_spec(spec: JsEntryPointSpec) -> EntryPointSpec {
    EntryPointSpec { path: PathBuf::from(spec.path), name: spec.name }
}

fn convert_graph_options(options: Option<JsGraphOptions>) -> GraphOptions {
    let options = options.unwrap_or_default();
    GraphOptions {
        root: options.root.map(PathBuf::from),
        tsconfig: options.tsconfig.map(PathBuf::from),
        external_docs: ExternalDocsOptions {
            enabled: options.external_docs.unwrap_or(false),
            package_sources: convert_external_package_sources(options.external_package_sources),
        },
    }
}

fn convert_entrypoint_docs_options(
    options: Option<JsEntryPointDocsOptions>,
) -> EntryPointDocsOptions {
    let options = options.unwrap_or_default();
    EntryPointDocsOptions {
        graph: GraphOptions {
            root: options.root.map(PathBuf::from),
            tsconfig: options.tsconfig.map(PathBuf::from),
            external_docs: ExternalDocsOptions {
                enabled: options.external_docs.unwrap_or(false),
                package_sources: convert_external_package_sources(options.external_package_sources),
            },
        },
        include_private: options.private.unwrap_or(false),
        include_internal: options.internal.unwrap_or(false),
        type_parameters: options.type_parameters.unwrap_or(false),
    }
}

fn convert_external_package_sources(
    sources: Option<Vec<JsExternalPackageSource>>,
) -> Vec<ExternalPackageSource> {
    sources
        .unwrap_or_default()
        .into_iter()
        .map(|source| ExternalPackageSource {
            package: source.package,
            entry: PathBuf::from(source.entry),
        })
        .collect()
}

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

fn map_public_export(export: PublicExport) -> JsPublicExport {
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

fn map_docs_diagnostic(diagnostic: DocsDiagnostic) -> JsDocsDiagnostic {
    JsDocsDiagnostic {
        code: map_docs_diagnostic_code(diagnostic.code),
        entrypoint: diagnostic.entrypoint,
        export_name: diagnostic.export_name,
        export_kind: map_export_kind(diagnostic.export_kind),
        source: map_export_source(diagnostic.source),
        message: diagnostic.message,
    }
}

fn map_export_graph(graph: ExportGraph) -> JsExportGraph {
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

fn map_docs_nav_item(item: DocsNavItem) -> JsDocsNavItem {
    JsDocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(map_docs_nav_item).collect::<Vec<JsDocsNavItem>>()
        }),
    }
}

fn convert_docs_nav_item(item: JsDocsNavItem) -> DocsNavItem {
    DocsNavItem {
        title: item.title,
        path: item.path,
        children: item.children.map(|children| {
            children.into_iter().map(convert_docs_nav_item).collect::<Vec<DocsNavItem>>()
        }),
    }
}

fn convert_markdown_param(param: JsDocParam) -> ApiParamDoc {
    ApiParamDoc {
        name: param.name,
        type_annotation: param.r#type,
        description: param.description,
        optional: param.optional.unwrap_or(false),
        default_value: param.r#default,
    }
}

fn convert_markdown_return(return_doc: JsDocReturn) -> ApiReturnDoc {
    ApiReturnDoc {
        type_annotation: return_doc.r#type,
        description: return_doc.description,
        members: return_doc
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
    }
}

fn convert_markdown_tag(tag: JsDocsMarkdownTag) -> ApiDocTag {
    ApiDocTag { tag: tag.tag, value: tag.value }
}

fn map_api_doc_tag(tag: ApiDocTag) -> JsDocsMarkdownTag {
    JsDocsMarkdownTag { tag: tag.tag, value: tag.value }
}

fn convert_markdown_member(member: JsDocMember) -> ApiDocMember {
    ApiDocMember {
        name: member.name,
        kind: member.kind,
        description: member.description,
        signature: member.signature,
        type_annotation: member.r#type,
        default_value: member.r#default,
        params: member.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        type_parameters: member
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
        returns: member.returns.map(convert_markdown_return),
        members: member
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        optional: member.optional.unwrap_or(false),
        readonly: member.readonly.unwrap_or(false),
        r#static: member.r#static.unwrap_or(false),
        private: member.private.unwrap_or(false),
        tags: member
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|(tag, value)| ApiDocTag { tag, value })
            .collect(),
        implementation_of: member.implementation_of.unwrap_or_default(),
        line: member.line,
        end_line: member.end_line,
    }
}

pub fn convert_markdown_entry(entry: JsDocsMarkdownEntry) -> ApiDocEntry {
    ApiDocEntry {
        name: entry.name,
        kind: entry.kind,
        description: entry.description,
        params: entry.params.unwrap_or_default().into_iter().map(convert_markdown_param).collect(),
        returns: entry.returns.map(convert_markdown_return),
        examples: entry.examples.unwrap_or_default(),
        tags: entry.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        private: entry.private,
        file: entry.file,
        line: entry.line,
        end_line: entry.end_line,
        signature: entry.signature,
        extends: entry.extends.unwrap_or_default(),
        implements: entry.implements.unwrap_or_default(),
        has_body: entry.has_body.unwrap_or(false),
        members: entry
            .members
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_member)
            .collect(),
        type_parameters: entry
            .type_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_markdown_type_param)
            .collect(),
    }
}

fn convert_markdown_type_param(type_param: JsTypeParam) -> ApiTypeParamDoc {
    ApiTypeParamDoc {
        name: type_param.name,
        constraint: type_param.constraint,
        default: type_param.r#default,
        description: type_param.description,
    }
}

fn convert_markdown_module(module: JsDocsMarkdownModule) -> ApiDocModule {
    ApiDocModule {
        file: module.file,
        description: module.description.unwrap_or_default(),
        source_path: module.source_path.unwrap_or_default(),
        examples: module.examples.unwrap_or_default(),
        tags: module.tags.unwrap_or_default().into_iter().map(convert_markdown_tag).collect(),
        entries: module.entries.into_iter().map(convert_markdown_entry).collect(),
    }
}

fn map_extracted_doc_module(module: ExtractedDocModule) -> JsExtractedDocsModule {
    JsExtractedDocsModule {
        file: module.file,
        entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
    }
}

fn convert_docs_output_options(options: Option<JsDocsOutputOptions>) -> DocsOutputOptions {
    let options = options.unwrap_or_default();
    DocsOutputOptions {
        generate_nav: options.generate_nav.unwrap_or(false),
        group_by: options.group_by.unwrap_or_else(|| "file".to_string()),
        generated_at: options.generated_at.unwrap_or_default(),
        base_path: options.base_path,
        path_strategy: parse_markdown_path_strategy(options.path_strategy.as_deref()),
        group_order: options.group_order,
        sort: options.sort,
        sort_entry_points: options.sort_entry_points.unwrap_or(true),
        kind_sort_order: options.kind_sort_order,
        single_entry_root: parse_single_entry_root(options.single_entry_root.as_deref()),
    }
}

/// Extracts documented declarations from a JavaScript/TypeScript file using Oxc.
#[napi]
pub fn extract_file_docs(
    file_path: String,
    include_private: Option<bool>,
    include_internal: Option<bool>,
) -> Result<Vec<JsSourceDocItem>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(items.into_iter().map(map_doc_item).collect())
}

/// Extracts normalized documentation entries from a JavaScript/TypeScript file using Oxc.
#[napi(js_name = "extractFileDocEntries")]
pub fn extract_file_doc_entries(
    file_path: String,
    include_private: Option<bool>,
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsDocEntry>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(normalize_doc_items(items, type_parameters.unwrap_or(false))
        .into_iter()
        .map(map_normalized_doc_entry)
        .collect())
}

/// Generates sidebar navigation metadata from documentation file paths.
#[napi(js_name = "generateDocsNavMetadata")]
pub fn generate_docs_nav_metadata(
    files: Vec<String>,
    base_path: Option<String>,
) -> Vec<JsDocsNavItem> {
    generate_nav_metadata(&files, base_path.as_deref()).into_iter().map(map_docs_nav_item).collect()
}

/// Generates sidebar navigation metadata from extracted documentation modules.
///
/// Use this when the output `pathStrategy` is `"typedoc"` so that the navigation
/// tree mirrors the nested module/category/symbol pages.
#[napi(js_name = "generateDocsNavMetadataFromDocs")]
pub fn generate_docs_nav_metadata_from_docs_napi(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsNavOptions>,
) -> Vec<JsDocsNavItem> {
    let options = options.unwrap_or_default();
    let strategy = parse_markdown_path_strategy(options.path_strategy.as_deref());
    let modules = docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>();
    generate_nav_metadata_from_docs_with_options(
        &modules,
        &DocsNavMetadataOptions {
            base_path: options.base_path.as_deref(),
            path_strategy: strategy,
            group_order: options.group_order.as_deref(),
            sort: options.sort.as_deref(),
            sort_entry_points: options.sort_entry_points.unwrap_or(true),
            kind_sort_order: options.kind_sort_order.as_deref(),
            single_entry_root: parse_single_entry_root(options.single_entry_root.as_deref()),
        },
    )
    .into_iter()
    .map(map_docs_nav_item)
    .collect()
}

/// Generates TypeScript source code for documentation navigation metadata.
#[napi(js_name = "generateDocsNavCode")]
pub fn generate_docs_nav_code(
    nav_items: Vec<JsDocsNavItem>,
    export_name: Option<String>,
) -> String {
    let nav_items = nav_items.into_iter().map(convert_docs_nav_item).collect::<Vec<_>>();
    generate_nav_code(&nav_items, export_name.as_deref())
}

/// Collects source files for generated API documentation.
#[napi(js_name = "collectDocsSourceFiles")]
pub fn collect_docs_source_files(
    src_dir: String,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Vec<String> {
    ox_content_docs::collect_source_files(&src_dir, &include, &exclude)
}

/// Extracts normalized documentation entries from source directories using Oxc.
#[napi(js_name = "extractDocsFromDirectories")]
pub fn extract_docs_from_directories_napi(
    src_dirs: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    include_private: Option<bool>,
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsExtractedDocsModule>> {
    let modules = extract_docs_from_directories(
        &src_dirs,
        &include,
        &exclude,
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
        type_parameters.unwrap_or(false),
    )
    .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(modules.into_iter().map(map_extracted_doc_module).collect())
}

/// Builds the public API export graph from entry points.
#[napi(js_name = "buildExportGraph")]
pub fn build_export_graph_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsGraphOptions>,
) -> Result<JsExportGraph> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let graph = build_export_graph(&entry_points, &convert_graph_options(options))
        .map_err(|error| Error::from_reason(error.to_string()))?;
    Ok(map_export_graph(graph))
}

/// Extracts generated API docs grouped by public entry points.
#[napi(js_name = "extractDocsFromEntryPoints")]
pub fn extract_docs_from_entry_points_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsEntryPointDocsOptions>,
) -> Result<Vec<JsEntrypointDocsModule>> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let modules =
        extract_docs_from_entry_points(&entry_points, &convert_entrypoint_docs_options(options))
            .map_err(|error| Error::from_reason(error.to_string()))?;

    Ok(modules
        .into_iter()
        .map(|module| JsEntrypointDocsModule {
            name: module.name,
            file: module.file,
            source_path: path_to_string(&module.source_path),
            description: module.description,
            examples: module.examples,
            tags: module.tags.into_iter().map(map_api_doc_tag).collect(),
            entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
            exports: module.exports.into_iter().map(map_public_export).collect(),
            diagnostics: module.diagnostics.into_iter().map(map_docs_diagnostic).collect(),
        })
        .collect())
}

/// Generates Markdown API reference pages from extracted documentation entries.
#[napi(js_name = "generateDocsMarkdown")]
pub fn generate_docs_markdown(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsMarkdownOptions>,
) -> HashMap<String, String> {
    let options =
        options.map_or_else(MarkdownDocsOptions::default, |options| MarkdownDocsOptions {
            group_by: options.group_by.unwrap_or_else(|| "file".to_string()),
            github_url: options.github_url,
            link_style: parse_markdown_link_style(options.link_style.as_deref()),
            base_path: options.base_path,
            path_strategy: parse_markdown_path_strategy(options.path_strategy.as_deref()),
            render_style: parse_markdown_render_style(options.render_style.as_deref()),
            index_format: parse_markdown_display_format(options.index_format.as_deref()),
            parameters_format: parse_markdown_display_format(options.parameters_format.as_deref()),
            interface_properties_format: parse_markdown_display_format(
                options.interface_properties_format.as_deref(),
            ),
            class_properties_format: parse_markdown_display_format(
                options.class_properties_format.as_deref(),
            ),
            type_alias_properties_format: parse_markdown_display_format(
                options.type_alias_properties_format.as_deref(),
            ),
            enum_members_format: parse_markdown_display_format(
                options.enum_members_format.as_deref(),
            ),
            property_members_format: parse_markdown_display_format(
                options.property_members_format.as_deref(),
            ),
            type_declaration_format: parse_markdown_display_format(
                options.type_declaration_format.as_deref(),
            ),
            render_stats: options.render_stats.unwrap_or(true),
            render_generated_by: options.render_generated_by.unwrap_or(true),
            group_order: options.group_order,
            sort: options.sort,
            sort_entry_points: options.sort_entry_points.unwrap_or(true),
            kind_sort_order: options.kind_sort_order,
            single_entry_root: parse_single_entry_root(options.single_entry_root.as_deref()),
        });
    generate_markdown(&docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(), &options)
        .into_iter()
        .collect()
}

fn parse_markdown_link_style(link_style: Option<&str>) -> MarkdownLinkStyle {
    match link_style {
        Some("clean") => MarkdownLinkStyle::Clean,
        _ => MarkdownLinkStyle::Markdown,
    }
}

fn parse_markdown_path_strategy(path_strategy: Option<&str>) -> MarkdownPathStrategy {
    match path_strategy {
        Some("typedoc") => MarkdownPathStrategy::TypeDoc,
        _ => MarkdownPathStrategy::Flat,
    }
}

fn parse_single_entry_root(value: Option<&str>) -> MarkdownSingleEntryRoot {
    match value {
        Some("flatten") => MarkdownSingleEntryRoot::Flatten,
        _ => MarkdownSingleEntryRoot::Preserve,
    }
}

fn parse_markdown_render_style(render_style: Option<&str>) -> MarkdownRenderStyle {
    match render_style {
        Some("markdown") => MarkdownRenderStyle::Markdown,
        _ => MarkdownRenderStyle::Html,
    }
}

fn parse_markdown_display_format(format: Option<&str>) -> MarkdownDisplayFormat {
    match format {
        Some("list") => MarkdownDisplayFormat::List,
        Some("table") => MarkdownDisplayFormat::Table,
        _ => MarkdownDisplayFormat::None,
    }
}

/// Generates the machine-readable docs data JSON payload.
#[napi(js_name = "generateDocsDataJson")]
pub fn generate_docs_data_json_napi(
    docs: Vec<JsDocsMarkdownModule>,
    generated_at: String,
) -> Result<String> {
    generate_docs_data_json(
        &docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(),
        &generated_at,
    )
    .map_err(|error| Error::from_reason(error.to_string()))
}

/// Writes generated API documentation files and native sidecars.
#[napi(js_name = "writeGeneratedDocs")]
#[allow(clippy::implicit_hasher)]
pub fn write_generated_docs(
    docs: HashMap<String, String>,
    out_dir: String,
    extracted_docs: Option<Vec<JsDocsMarkdownModule>>,
    options: Option<JsDocsOutputOptions>,
) -> Result<()> {
    let docs = docs.into_iter().collect::<BTreeMap<_, _>>();
    let extracted_docs = extracted_docs
        .map(|docs| docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>());
    let options = convert_docs_output_options(options);

    write_docs_output(&docs, Path::new(&out_dir), extracted_docs.as_deref(), &options)
        .map_err(|error| Error::from_reason(error.to_string()))
}
