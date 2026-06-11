use super::super::*;

#[test]
fn test_extract_file_level_module_jsdoc() {
    let source = r"
/**
 * @module default
 *
 * Main entry point for the framework.
 */
export { cli } from './core';

/** Runs the CLI. */
export function cli(): void {}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/index.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "default");
    assert_eq!(items[0].kind, DocItemKind::Module);
    assert_eq!(items[0].doc.as_deref(), Some("Main entry point for the framework."));
    assert!(items[0].tags.iter().any(|tag| tag.tag == "module"));
    assert_eq!(items[1].name, "cli");
}

#[test]
fn test_extract_function_type_parameters() {
    let source = r"
/** Make a thing. */
export function make<G extends Base = Default, V>(value: V): G {
  return value as unknown as G;
}
";
    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/make.ts", SourceType::ts()).unwrap();
    let func = items.iter().find(|item| item.name == "make").unwrap();

    assert_eq!(func.type_parameters.len(), 2);
    assert_eq!(func.type_parameters[0].name, "G");
    assert_eq!(func.type_parameters[0].constraint.as_deref(), Some("Base"));
    assert_eq!(func.type_parameters[0].default.as_deref(), Some("Default"));
    assert_eq!(func.type_parameters[1].name, "V");
    assert_eq!(func.type_parameters[1].constraint, None);
    assert_eq!(func.type_parameters[1].default, None);
}

#[test]
fn test_extract_member_type_parameters() {
    let source = r"
/** Plugin context. */
export interface PluginContext<G> {
  /**
   * Decorate the command.
   * @typeParam L - Extension context.
   */
  decorateCommand<L extends Record<string, unknown> = DefaultExtensions>(
    decorator: (value: L) => void
  ): void;

  /**
   * Setup hook.
   * @typeParam T - Hook value.
   */
  setup?: <T extends BaseHook = DefaultHook>(value: T) => Result;
}
";
    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/context.ts", SourceType::ts()).unwrap();
    let interface = items.iter().find(|item| item.name == "PluginContext").unwrap();

    let method = interface.children.iter().find(|item| item.name == "decorateCommand").unwrap();
    assert_eq!(method.type_parameters.len(), 1);
    assert_eq!(method.type_parameters[0].name, "L");
    assert_eq!(method.type_parameters[0].constraint.as_deref(), Some("Record<string, unknown>"));
    assert_eq!(method.type_parameters[0].default.as_deref(), Some("DefaultExtensions"));

    let property = interface.children.iter().find(|item| item.name == "setup").unwrap();
    assert_eq!(property.type_parameters.len(), 1);
    assert_eq!(property.type_parameters[0].name, "T");
    assert_eq!(property.type_parameters[0].constraint.as_deref(), Some("BaseHook"));
    assert_eq!(property.type_parameters[0].default.as_deref(), Some("DefaultHook"));
}

#[test]
fn test_module_description_survives_trailing_author_comment() {
    // Regression: `@module` block immediately followed by a second leading
    // block comment (`@author`/`@license`). Both comments attach to the same
    // first statement, so an `attached_to`-keyed lookup would surface the
    // second comment and drop the `@module` description.
    let source = r"
/**
 * Module summary.
 * @module
 */

/**
 * @author kazupon
 * @license MIT
 */
export const z = 1;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/context.ts", SourceType::ts()).unwrap();
    let module = items.iter().find(|item| item.kind == DocItemKind::Module).unwrap();

    assert_eq!(module.name, "context");
    assert_eq!(module.doc.as_deref(), Some("Module summary."));
    assert!(module.tags.iter().any(|tag| tag.tag == "module"));
}

#[test]
fn test_module_description_from_detached_comment_without_module_tag() {
    // Gap 1: a leading file comment without `@module`, separated from the code
    // by a blank line, should still be used as the module description
    // (matching TypeDoc). The file stem becomes the module name.
    let source = r"
/**
 * The entry point for AI agent detection utility.
 *
 * @author kazupon
 * @license MIT
 */

import { agentInfo } from 'std-env';

/** A profile. */
export function getAgentProfile(): void {}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/agent.ts", SourceType::ts()).unwrap();
    let module = items.iter().find(|item| item.kind == DocItemKind::Module).unwrap();

    assert_eq!(module.name, "agent");
    assert_eq!(module.doc.as_deref(), Some("The entry point for AI agent detection utility."));
    // The real declaration is still extracted with its own doc.
    let func = items.iter().find(|item| item.name == "getAgentProfile").unwrap();
    assert_eq!(func.doc.as_deref(), Some("A profile."));
}

#[test]
fn test_leading_comment_attached_to_declaration_is_not_a_module() {
    // A doc comment that directly precedes the first declaration (no blank
    // line, no module marker) documents that declaration, not the module.
    let source = r"
/** Documents foo. */
export function foo(): void {}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/foo.ts", SourceType::ts()).unwrap();

    assert!(items.iter().all(|item| item.kind != DocItemKind::Module));
    let func = items.iter().find(|item| item.name == "foo").unwrap();
    assert_eq!(func.doc.as_deref(), Some("Documents foo."));
}

#[test]
fn test_module_jsdoc_name_falls_back_to_file_stem() {
    let source = r"
/**
 * @module
 */
export { value } from './value';
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "src/runtime.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "runtime");
    assert_eq!(items[0].kind, DocItemKind::Module);
}

#[test]
fn test_internal_items_are_excluded_by_default() {
    let source = r"
/** Public command. */
export function publicCommand(): void {}

/**
 * Internal helper.
 * @internal
 */
export function internalHelper(): void {}
";

    let public_only =
        DocExtractor::new().extract_source(source, "visibility.ts", SourceType::ts()).unwrap();
    assert_eq!(public_only.len(), 1);
    assert_eq!(public_only[0].name, "publicCommand");

    let with_internal = DocExtractor::with_visibility(false, true)
        .extract_source(source, "visibility.ts", SourceType::ts())
        .unwrap();
    assert_eq!(with_internal.len(), 2);
    assert_eq!(with_internal[1].name, "internalHelper");
}
