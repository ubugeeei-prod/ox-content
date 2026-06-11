use std::fs;
use std::path::PathBuf;

use super::super::*;
use super::support::{temp_root, write_external_package};

#[test]
fn extracts_docs_from_resolved_external_package_types() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/index.ts"), "export { ExternalThing } from 'external-pkg';\n")
        .unwrap();
    write_external_package(
        &root,
        "external-pkg",
        r"
/** External thing. */
export interface ExternalThing {
  value: string;
}
",
    );

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let graph_options = GraphOptions {
        root: Some(root.clone()),
        external_docs: ExternalDocsOptions { enabled: true, package_sources: vec![] },
        ..GraphOptions::default()
    };

    let graph = build_export_graph(&entrypoints, &graph_options).unwrap();
    assert!(graph.entrypoints[0].exports.iter().any(|export| matches!(
        &export.source,
        ExportSource::External { package, module: Some(module), original_name, .. }
            if package == "external-pkg"
                && original_name == "ExternalThing"
                && module.ends_with("index.d.ts")
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

    assert_eq!(docs[0].entries[0].name, "ExternalThing");
    assert_eq!(docs[0].entries[0].description, "External thing.");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn follows_import_aliases_in_declaration_barrels() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/index.ts"), "export { parseArgs } from 'external-pkg';\n").unwrap();
    let package_root = root.join("node_modules/external-pkg");
    fs::create_dir_all(package_root.join("lib")).unwrap();
    fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "external-pkg",
  "type": "module",
  "exports": {
    ".": {
      "types": "./lib/index.d.ts",
      "default": "./lib/index.js"
    }
  }
}"#,
    )
    .unwrap();
    fs::write(
        package_root.join("lib/index.d.ts"),
        r#"
import { a as parseArgs } from "./parser-hash.js";
export { parseArgs };
"#,
    )
    .unwrap();
    fs::write(
        package_root.join("lib/parser-hash.d.ts"),
        r"
/** Parse args. */
declare function a(): void;
export { a };
",
    )
    .unwrap();

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let graph_options = GraphOptions {
        root: Some(root.clone()),
        external_docs: ExternalDocsOptions { enabled: true, package_sources: vec![] },
        ..GraphOptions::default()
    };
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

    assert_eq!(docs[0].entries[0].name, "parseArgs");
    assert_eq!(docs[0].entries[0].description, "Parse args.");
    assert!(docs[0].entries[0].file.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_package_source_mapping_prefers_workspace_source() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("packages/plugin/src")).unwrap();
    fs::write(root.join("src/index.ts"), "export { helper } from '@scope/plugin';\n").unwrap();
    fs::write(
        root.join("packages/plugin/src/index.ts"),
        r"
/** Workspace helper. */
export function helper(): void {}
",
    )
    .unwrap();

    let entrypoints =
        [EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("default".to_string()) }];
    let graph_options = GraphOptions {
        root: Some(root.clone()),
        external_docs: ExternalDocsOptions {
            enabled: true,
            package_sources: vec![ExternalPackageSource {
                package: "@scope/plugin".to_string(),
                entry: PathBuf::from("packages/plugin/src/index.ts"),
            }],
        },
        ..GraphOptions::default()
    };

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

    assert_eq!(docs[0].entries[0].name, "helper");
    assert_eq!(docs[0].entries[0].description, "Workspace helper.");
    assert!(docs[0].entries[0].file.ends_with("packages/plugin/src/index.ts"));

    fs::remove_dir_all(root).unwrap();
}
