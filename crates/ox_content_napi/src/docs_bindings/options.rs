use std::path::PathBuf;

use ox_content_docs::{
    DocsOutputOptions, EntryPointDocsOptions, EntryPointSpec, ExternalDocsOptions,
    ExternalPackageSource, GraphOptions, MarkdownDisplayFormat, MarkdownDocsOptions,
    MarkdownLinkStyle, MarkdownPathStrategy, MarkdownRenderStyle, MarkdownSingleEntryRoot,
};

use crate::{
    JsDocsMarkdownOptions, JsDocsOutputOptions, JsEntryPointDocsOptions, JsEntryPointSpec,
    JsExternalPackageSource, JsGraphOptions,
};

pub(super) fn convert_entrypoint_spec(spec: JsEntryPointSpec) -> EntryPointSpec {
    EntryPointSpec { path: PathBuf::from(spec.path), name: spec.name }
}

pub(super) fn convert_graph_options(options: Option<JsGraphOptions>) -> GraphOptions {
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

pub(super) fn convert_entrypoint_docs_options(
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

pub(super) fn convert_docs_output_options(
    options: Option<JsDocsOutputOptions>,
) -> DocsOutputOptions {
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

pub(super) fn convert_markdown_docs_options(
    options: Option<JsDocsMarkdownOptions>,
) -> MarkdownDocsOptions {
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
        enum_members_format: parse_markdown_display_format(options.enum_members_format.as_deref()),
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
    })
}

fn parse_markdown_link_style(link_style: Option<&str>) -> MarkdownLinkStyle {
    match link_style {
        Some("clean") => MarkdownLinkStyle::Clean,
        _ => MarkdownLinkStyle::Markdown,
    }
}

pub(super) fn parse_markdown_path_strategy(path_strategy: Option<&str>) -> MarkdownPathStrategy {
    match path_strategy {
        Some("typedoc") => MarkdownPathStrategy::TypeDoc,
        _ => MarkdownPathStrategy::Flat,
    }
}

pub(super) fn parse_single_entry_root(value: Option<&str>) -> MarkdownSingleEntryRoot {
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
