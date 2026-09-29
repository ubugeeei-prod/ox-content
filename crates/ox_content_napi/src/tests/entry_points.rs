use super::*;

#[test]
fn extract_docs_from_entry_points_preserves_explicit_module_name() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-module-name-{}-{unique}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/index.ts"),
        r"
/**
 * gunshi cli entry point.
 *
 * @example
 * ```ts
 * cli()
 * ```
 *
 * @experimental This module is experimental.
 *
 * @module default
 */
/** Runs the CLI. */
export function cli(): void {}
",
    )
    .unwrap();

    let modules = extract_docs_from_entry_points_napi(
        vec![JsEntryPointSpec { path: "src/index.ts".to_string(), name: None }],
        Some(JsEntryPointDocsOptions {
            root: Some(root.to_string_lossy().into_owned()),
            ..Default::default()
        }),
    )
    .unwrap();

    assert_eq!(modules[0].name, "default");
    assert_eq!(modules[0].file, "default");
    assert_eq!(modules[0].description, "gunshi cli entry point.");
    assert_eq!(modules[0].examples, vec!["```ts\ncli()\n```".to_string()]);
    assert_eq!(modules[0].tags.len(), 1);
    assert_eq!(modules[0].tags[0].tag, "experimental");
    assert_eq!(modules[0].tags[0].value, "This module is experimental.");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn extract_docs_from_entry_points_accepts_external_docs_options() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-external-docs-{}-{unique}", std::process::id()));
    let package_root = root.join("node_modules/external-pkg");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(package_root.join("lib")).unwrap();
    fs::write(root.join("src/index.ts"), "export { ExternalThing } from 'external-pkg';\n")
        .unwrap();
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
        r"
/** External thing. */
export interface ExternalThing {
  value: string;
}
",
    )
    .unwrap();

    let modules = extract_docs_from_entry_points_napi(
        vec![JsEntryPointSpec {
            path: "src/index.ts".to_string(),
            name: Some("default".to_string()),
        }],
        Some(JsEntryPointDocsOptions {
            root: Some(root.to_string_lossy().into_owned()),
            external_docs: Some(true),
            ..Default::default()
        }),
    )
    .unwrap();

    assert_eq!(modules[0].entries[0].name, "ExternalThing");
    assert_eq!(modules[0].entries[0].description, "External thing.");
    assert_eq!(modules[0].exports[0].source.kind, "external");
    assert!(
        modules[0].exports[0]
            .source
            .module
            .as_deref()
            .is_some_and(|module| { module.ends_with("external-pkg/lib/index.d.ts") })
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn extract_docs_from_entry_points_emits_undocumented_public_const_and_diagnostics() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-local-export-docs-{}-{unique}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/index.ts"),
        r"
export { ANONYMOUS_COMMAND_NAME } from './constants';
export type { ExtractArgs } from './types';
",
    )
    .unwrap();
    fs::write(
        root.join("src/constants.ts"),
        r#"
export const ANONYMOUS_COMMAND_NAME = "(anonymous)";
"#,
    )
    .unwrap();
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

    let modules = extract_docs_from_entry_points_napi(
        vec![JsEntryPointSpec {
            path: "src/index.ts".to_string(),
            name: Some("default".to_string()),
        }],
        Some(JsEntryPointDocsOptions {
            root: Some(root.to_string_lossy().into_owned()),
            ..Default::default()
        }),
    )
    .unwrap();

    assert_eq!(modules[0].entries.len(), 1);
    assert_eq!(modules[0].entries[0].name, "ANONYMOUS_COMMAND_NAME");
    assert_eq!(modules[0].entries[0].kind, "variable");
    assert!(modules[0].entries[0].description.is_empty());
    assert_eq!(modules[0].diagnostics.len(), 1);
    assert_eq!(modules[0].diagnostics[0].code, "filteredByVisibility");
    assert_eq!(modules[0].diagnostics[0].export_name, "ExtractArgs");
    assert_eq!(modules[0].diagnostics[0].source.original_name, "ExtractArgs");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn extract_docs_from_entry_points_keeps_every_value_of_a_repeated_tag() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir()
        .join(format!("ox-content-napi-repeated-tags-{}-{unique}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/index.ts"),
        r"
/**
 * The parsers.
 *
 * @see https://example.com/module-first
 * @see https://example.com/module-second
 * @module
 */
/**
 * A parser.
 *
 * @see https://example.com/first
 * @see https://example.com/second
 */
export interface Parser {
  /**
   * Parse a value.
   *
   * @see https://example.com/parse-first
   * @remarks Strict.
   * @see https://example.com/parse-second
   */
  parse(value: string): number
}
",
    )
    .unwrap();

    let modules = extract_docs_from_entry_points_napi(
        vec![JsEntryPointSpec { path: "src/index.ts".to_string(), name: None }],
        Some(JsEntryPointDocsOptions {
            root: Some(root.to_string_lossy().into_owned()),
            ..Default::default()
        }),
    )
    .unwrap();
    let tags = |tags: &[JsDocsMarkdownTag]| {
        tags.iter().map(|tag| format!("@{} {}", tag.tag, tag.value)).collect::<Vec<_>>()
    };

    assert_eq!(
        tags(&modules[0].tags),
        ["@see https://example.com/module-first", "@see https://example.com/module-second"]
    );
    let parser = modules[0].entries.iter().find(|entry| entry.name == "Parser").unwrap();
    assert_eq!(
        tags(parser.tag_list.as_deref().unwrap()),
        ["@see https://example.com/first", "@see https://example.com/second"]
    );
    assert_eq!(
        parser.tags.as_ref().unwrap().get("see").map(String::as_str),
        Some("https://example.com/first")
    );

    // `generateDocsMarkdown` takes the members as extracted, with every tag.
    let parse = parser.members.as_ref().unwrap()[0].clone();
    let markdown_entry = |member| JsDocsMarkdownEntry {
        name: "Parser".to_string(),
        kind: "interface".to_string(),
        members: Some(vec![member]),
        ..Default::default()
    };
    let converted = convert_markdown_entry(markdown_entry(parse.clone()));
    let member_tags: Vec<_> =
        converted.members[0].tags.iter().map(|tag| format!("@{} {}", tag.tag, tag.value)).collect();
    assert_eq!(
        member_tags,
        [
            "@see https://example.com/parse-first",
            "@remarks Strict.",
            "@see https://example.com/parse-second"
        ]
    );

    // A member without `tagList` keeps its tag map, in tag name order.
    let converted = convert_markdown_entry(markdown_entry(JsDocMember { tag_list: None, ..parse }));
    let member_tags: Vec<_> =
        converted.members[0].tags.iter().map(|tag| tag.tag.as_str()).collect();
    assert_eq!(member_tags, ["remarks", "see"]);

    let _ = fs::remove_dir_all(root);
}
