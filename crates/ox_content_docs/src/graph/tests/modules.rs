use std::fs;
use std::path::PathBuf;

use super::super::*;
use super::support::temp_root;

#[test]
fn entrypoint_docs_capture_module_level_description() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/context.ts"),
        r"
/**
 * The entry for gunshi context.
 *
 * @example
 * ```ts
 * createCommandContext()
 * ```
 *
 * @experimental This entry point is experimental.
 *
 * @module
 */
export { createCommandContext } from './context-impl';
",
    )
    .unwrap();
    fs::write(
        root.join("src/context-impl.ts"),
        r"
/** Creates a command context. */
export function createCommandContext(): void {}
",
    )
    .unwrap();
    fs::write(
        root.join("src/plugin.ts"),
        r"
export { plugin } from './plugin-impl';
",
    )
    .unwrap();
    fs::write(
        root.join("src/plugin-impl.ts"),
        r"
/** Defines a plugin. */
export function plugin(): void {}
",
    )
    .unwrap();

    let entrypoints = [
        EntryPointSpec { path: PathBuf::from("src/context.ts"), name: Some("context".to_string()) },
        EntryPointSpec { path: PathBuf::from("src/plugin.ts"), name: Some("plugin".to_string()) },
    ];
    let graph_options = GraphOptions { root: Some(root.clone()), ..GraphOptions::default() };

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

    let context = docs.iter().find(|module| module.name == "context").unwrap();
    assert_eq!(context.file, "context");
    assert_eq!(context.description, "The entry for gunshi context.");
    assert_eq!(context.examples, vec!["```ts\ncreateCommandContext()\n```".to_string()]);
    assert_eq!(context.tags.len(), 1);
    assert_eq!(context.tags[0].tag, "experimental");
    assert_eq!(context.tags[0].value, "This entry point is experimental.");
    assert!(context.entries.iter().all(|entry| entry.kind != NormalizedDocKind::Module));
    assert!(context.entries.iter().any(|entry| entry.name == "createCommandContext"));

    let plugin = docs.iter().find(|module| module.name == "plugin").unwrap();
    assert_eq!(plugin.file, "plugin");
    assert!(plugin.description.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn entrypoint_docs_prefers_explicit_module_name() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/index.ts"),
        r"
/**
 * Default entry point.
 *
 * @module default
 */
/** Runs the CLI. */
export function cli(): void {}
",
    )
    .unwrap();

    let docs = extract_docs_from_entry_points(
        &[EntryPointSpec { path: PathBuf::from("src/index.ts"), name: Some("entry".to_string()) }],
        &EntryPointDocsOptions {
            graph: GraphOptions { root: Some(root.clone()), ..GraphOptions::default() },
            include_private: false,
            include_internal: false,
            type_parameters: false,
        },
    )
    .unwrap();

    assert_eq!(docs[0].name, "default");
    assert_eq!(docs[0].file, "default");
    assert_eq!(docs[0].description, "Default entry point.");

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn entrypoint_docs_uses_entrypoint_name_without_module_tag() {
    let root = temp_root();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/agent.ts"),
        r"
export function isAiAgent(): boolean {
  return false;
}
",
    )
    .unwrap();

    let docs = extract_docs_from_entry_points(
        &[EntryPointSpec { path: PathBuf::from("src/agent.ts"), name: Some("agent".to_string()) }],
        &EntryPointDocsOptions {
            graph: GraphOptions { root: Some(root.clone()), ..GraphOptions::default() },
            include_private: false,
            include_internal: false,
            type_parameters: false,
        },
    )
    .unwrap();

    assert_eq!(docs[0].name, "agent");
    assert_eq!(docs[0].file, "agent");
    assert!(docs[0].description.is_empty());

    fs::remove_dir_all(root).unwrap();
}
