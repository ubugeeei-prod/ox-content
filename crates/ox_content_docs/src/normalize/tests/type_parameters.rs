use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn type_parameters_opt_in_merges_typeparam_and_excludes_tag() {
    let source = r"
/**
 * A combinator.
 * @typeParam T - The parsed value type.
 * @experimental
 */
export type Combinator<T> = { parse: (value: string) => T };
";
    let items = DocExtractor::new().extract_source(source, "src/c.ts", SourceType::ts()).unwrap();

    // Opted out (default): `@typeParam` stays a generic tag, no type parameters.
    let off = normalize_doc_items(items.clone(), false);
    let off = off.iter().find(|entry| entry.name == "Combinator").unwrap();
    assert!(off.type_parameters.is_empty());
    assert_eq!(off.tags.get("typeParam").map(String::as_str), Some("T - The parsed value type."));

    // Opted in: structured type parameter with merged description; tag removed.
    let on = normalize_doc_items(items, true);
    let on = on.iter().find(|entry| entry.name == "Combinator").unwrap();
    assert_eq!(on.type_parameters.len(), 1);
    assert_eq!(on.type_parameters[0].name, "T");
    assert_eq!(on.type_parameters[0].description, "The parsed value type.");
    assert!(!on.tags.contains_key("typeParam"));
    assert!(on.tags.contains_key("experimental"));
}

#[test]
fn member_type_parameters_opt_in_merges_typeparam_and_excludes_tag() {
    let source = r"
/** Plugin context. */
export interface PluginContext<G> {
  /**
   * Decorate the command.
   * @typeParam L - Extension context.
   * @experimental
   */
  decorateCommand<L extends Record<string, unknown> = DefaultExtensions>(
    decorator: (value: L) => void
  ): void;
}
";
    let items =
        DocExtractor::new().extract_source(source, "src/context.ts", SourceType::ts()).unwrap();

    let off = normalize_doc_items(items.clone(), false);
    let off_member = off[0].members.iter().find(|member| member.name == "decorateCommand").unwrap();
    assert!(off_member.type_parameters.is_empty());
    assert_eq!(
        off_member.tags.get("typeParam").map(String::as_str),
        Some("L - Extension context.")
    );

    let on = normalize_doc_items(items, true);
    let on_member = on[0].members.iter().find(|member| member.name == "decorateCommand").unwrap();
    assert_eq!(on_member.type_parameters.len(), 1);
    assert_eq!(on_member.type_parameters[0].name, "L");
    assert_eq!(on_member.type_parameters[0].constraint.as_deref(), Some("Record<string, unknown>"));
    assert_eq!(on_member.type_parameters[0].default.as_deref(), Some("DefaultExtensions"));
    assert_eq!(on_member.type_parameters[0].description, "Extension context.");
    assert!(!on_member.tags.contains_key("typeParam"));
    assert!(on_member.tags.contains_key("experimental"));
}
