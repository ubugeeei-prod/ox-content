use std::fs;
use std::path::PathBuf;

use super::super::*;
use super::support::temp_root;

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

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let graph_options = GraphOptions { root: Some(root.clone()), ..GraphOptions::default() };

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
            type_parameters: false,
        },
    )
    .unwrap();
    let names = docs[0].entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>();
    assert_eq!(names, ["sum", "Options", "label"]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn entrypoint_docs_emit_public_consts_without_jsdoc() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/index.ts"),
        r"
export { ANONYMOUS_COMMAND_NAME, CLI_OPTIONS_DEFAULT } from './constants';
",
    )
    .unwrap();
    fs::write(
        root.join("src/constants.ts"),
        r#"
export const ANONYMOUS_COMMAND_NAME = "(anonymous)";

export const CLI_OPTIONS_DEFAULT: CliOptions<DefaultGunshiParams> = {
  usageSilent: false,
};
"#,
    )
    .unwrap();

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let docs = extract_docs_from_entry_points(
        &entrypoints,
        &EntryPointDocsOptions {
            graph: GraphOptions { root: Some(root.clone()), ..GraphOptions::default() },
            include_private: false,
            include_internal: false,
            type_parameters: false,
        },
    )
    .unwrap();

    assert!(docs[0].diagnostics.is_empty());
    let anonymous =
        docs[0].entries.iter().find(|entry| entry.name == "ANONYMOUS_COMMAND_NAME").unwrap();
    assert_eq!(anonymous.kind.as_str(), "variable");
    assert!(anonymous.description.is_empty());
    assert_eq!(
        anonymous.signature.as_deref(),
        Some(r#"export const ANONYMOUS_COMMAND_NAME = "(anonymous)""#)
    );

    let cli_options =
        docs[0].entries.iter().find(|entry| entry.name == "CLI_OPTIONS_DEFAULT").unwrap();
    assert_eq!(cli_options.kind.as_str(), "variable");
    assert_eq!(
        cli_options.signature.as_deref(),
        Some("export const CLI_OPTIONS_DEFAULT: CliOptions<DefaultGunshiParams>")
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn entrypoint_docs_diagnose_internal_type_filtered_by_visibility() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/index.ts"), "export type { ExtractArgs } from './types';\n").unwrap();
    fs::write(
        root.join("src/types.ts"),
        r"
/**
 * Type helper to extract args.
 *
 * @internal
 */
export type ExtractArgs<G> = G extends { args: infer A } ? A : never;
",
    )
    .unwrap();

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let docs = extract_docs_from_entry_points(
        &entrypoints,
        &EntryPointDocsOptions {
            graph: GraphOptions { root: Some(root.clone()), ..GraphOptions::default() },
            include_private: false,
            include_internal: false,
            type_parameters: false,
        },
    )
    .unwrap();

    assert!(docs[0].entries.is_empty());
    assert_eq!(docs[0].diagnostics.len(), 1);
    assert_eq!(docs[0].diagnostics[0].code, DocsDiagnosticCode::FilteredByVisibility);
    assert_eq!(docs[0].diagnostics[0].export_name, "ExtractArgs");
    assert!(docs[0].diagnostics[0].message.contains("@internal"));

    let with_internal = extract_docs_from_entry_points(
        &entrypoints,
        &EntryPointDocsOptions {
            graph: GraphOptions { root: Some(root.clone()), ..GraphOptions::default() },
            include_private: false,
            include_internal: true,
            type_parameters: false,
        },
    )
    .unwrap();
    assert_eq!(with_internal[0].entries[0].name, "ExtractArgs");
    assert!(with_internal[0].diagnostics.is_empty());

    fs::remove_dir_all(root).unwrap();
}
