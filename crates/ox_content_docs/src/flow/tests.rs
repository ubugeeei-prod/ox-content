use oxc_allocator::Allocator;
use oxc_span::SourceType;

use super::{is_flow_source, parse, to_typescript};
use crate::DocExtractor;

/// Rewrites `source` and asserts the result parses as TypeScript with every
/// byte offset preserved.
#[track_caller]
fn rewrite(source: &str) -> String {
    let rewritten = to_typescript(source);
    assert_eq!(rewritten.len(), source.len(), "length changed:\n{rewritten}");
    assert_eq!(rewritten.lines().count(), source.lines().count());
    let allocator = Allocator::default();
    let ret = parse(&allocator, source, "input.js.flow", SourceType::mjs());
    assert!(ret.diagnostics.is_empty(), "{:?}\n--- rewritten ---\n{rewritten}", ret.diagnostics);
    rewritten
}

#[test]
fn detects_flow_files() {
    assert!(is_flow_source("lib/index.js.flow", "export type A = string;"));
    assert!(is_flow_source("a.js", "// @flow\nexport const a = 1;"));
    assert!(is_flow_source("a.jsx", "/**\n * Button.\n * @flow strict\n */\n"));
    assert!(is_flow_source("a.js", "#!/usr/bin/env node\n/* @noflow */\n"));
    assert!(!is_flow_source("a.js", "export const a = 1; // @flow"));
    assert!(!is_flow_source("a.ts", "// @flow\n"));
    assert!(!is_flow_source("a.js", "// @flowtype is not a pragma\n"));
}

#[test]
fn rewrites_maybe_types_and_exact_objects() {
    assert_eq!(
        rewrite("type A = {| +a: ?string, -b?: Array<?number> |};"),
        "type A = {   a:  string,  b?: Array< number>  };",
    );
    assert_eq!(rewrite("function f(x: ?string): ?number {}"), "function f(x:  string):  number {}");
    assert_eq!(rewrite("type F = (?string) => ?void;"), "type F = (a      ) =>  void;");
}

#[test]
fn keeps_ternaries_optional_chaining_and_nullish() {
    let source = "const a = b ? c : d;\nconst e = f?.g ?? h;\nfunction i(j?: string) {}\n";
    assert_eq!(rewrite(source), source);
}

#[test]
fn rewrites_type_parameter_bounds_and_variance() {
    assert_eq!(
        rewrite("function f<T: Object = {}, +U>(x: T): U {}"),
        "function f<T         = {},  U>(x: T): U {}",
    );
    assert_eq!(rewrite("class A<+T> { +x: T; -y: T }"), "class A< T> {  x: T;  y: T }");
}

#[test]
fn rewrites_type_casts() {
    assert_eq!(rewrite("const a = (b: any);"), "const a = (b     );");
    assert_eq!(rewrite("((a: any): string);"), "((a     )        );");
    assert_eq!(rewrite("const f = (x: T): T => x;"), "const f = (x: T): T => x;");
    assert_eq!(rewrite("const g = a ? (b) : c;"), "const g = a ? (b) : c;");
}

#[test]
fn rewrites_unnamed_function_type_parameters() {
    assert_eq!(
        rewrite("type F = (string, Array<T>, ...Array<U>) => void;"),
        "type F = (string, b       , ...c       ) => void;",
    );
    assert_eq!(rewrite("declare function f(Foo.Bar): void;"), "declare function f(a      ): void;");
}

#[test]
fn rewrites_object_type_spreads_and_inexact_objects() {
    assert_eq!(rewrite("type A = {...B, c: T};"), "type A = {      c: T};");
    assert_eq!(rewrite("type A = {|...$Exact<B>|};"), "type A = {              };");
    assert_eq!(rewrite("type A = {a: T, ...};"), "type A = {a: T,    };");
    assert_eq!(rewrite("type A = {...};"), "type A = {   };");
}

#[test]
fn keeps_value_spreads_parseable() {
    rewrite("const a = {...b, c};\nf(...d);\nconst [e, ...g] = h;\n");
    let jsx = "// @flow\nexport const A = (p: Props) => <B {...p} c=\"d\" {...e} />;\n";
    let allocator = Allocator::default();
    let ret = parse(&allocator, jsx, "a.js", SourceType::jsx());
    assert!(ret.diagnostics.is_empty(), "{:?}", ret.diagnostics);
}

#[test]
fn rewrites_flow_only_declarations() {
    assert_eq!(
        rewrite("export opaque type Id: string = string;"),
        "export        type Id         = string;"
    );
    assert_eq!(rewrite("declare opaque type Id: string;"), "declare        type Id= string;");
    assert_eq!(rewrite("declare export function f(): void;"), "export declare function f(): void;",);
    assert_eq!(rewrite("import typeof A from 'a';"), "import type   A from 'a';");
    assert_eq!(
        rewrite("import {typeof B, type C} from 'b';"),
        "import {type   B, type C} from 'b';"
    );
    assert_eq!(
        rewrite("declare function isString(x: mixed): boolean %checks(typeof x === 'string');"),
        "declare function isString(x: mixed): boolean                               ;",
    );
    assert_eq!(rewrite("type A = Array<*>;"), "type A = Array<_>;");
}

#[test]
fn ignores_flow_syntax_in_strings_comments_templates_and_regexes() {
    let source =
        "const a = '{| ?x |}';\n// (b: c)\nconst d = `${e} {| ?f |}`;\nconst g = /{|?h|}/u;\n";
    assert_eq!(rewrite(source), source);
}

#[test]
fn import_typeof_stops_at_module_specifier() {
    let source = "import 'polyfill'\nif (typeof window === 'object') {}\n";
    assert_eq!(rewrite(source), source);
}

#[test]
fn extracts_docs_from_flow_source_with_original_notation() {
    let source = r"// @flow

import type { Node } from 'react';

/** Button props. */
export type Props = {|
  /** The label. */
  +label: string,
  +icon?: ?Node,
  ...Rest,
|};

/**
 * Formats a value.
 *
 * @param value - The value to format.
 */
export function format<T: { toString(): string }>(value: ?T, options?: {| upper: boolean |}): string {
  const text = (value: any) == null ? '' : String(value);
  return options?.upper ? text.toUpperCase() : text;
}

/** A counter. */
export class Counter<+T> {
  +count: number;
  /** Increments. */
  increment(by: ?number): void {}
}
";

    let items =
        DocExtractor::new().extract_source(source, "src/format.js", SourceType::mjs()).unwrap();

    let format = items.iter().find(|item| item.name == "format").unwrap();
    assert_eq!(format.params[0].name, "value");
    assert_eq!(format.params[0].type_annotation.as_deref(), Some("?T"));
    assert_eq!(format.params[1].type_annotation.as_deref(), Some("{| upper: boolean |}"));
    assert_eq!(format.return_type.as_deref(), Some("string"));
    assert_eq!(format.doc.as_deref(), Some("Formats a value."));

    let props = items.iter().find(|item| item.name == "Props").unwrap();
    assert_eq!(
        props.signature.as_deref(),
        Some("export type Props = {| +label: string; +icon?: ?Node |}"),
    );
    assert_eq!(props.children[0].doc.as_deref(), Some("The label."));
    assert_eq!(props.children[1].signature.as_deref(), Some("?Node"));
    let counter = items.iter().find(|item| item.name == "Counter").unwrap();
    let increment = counter.children.iter().find(|item| item.name == "increment").unwrap();
    assert_eq!(increment.params[0].type_annotation.as_deref(), Some("?number"));
}

#[test]
fn leaves_non_flow_javascript_untouched() {
    let error = DocExtractor::new()
        .extract_source("export function f(x: ?string) {}", "a.js", SourceType::mjs())
        .unwrap_err();
    assert!(matches!(error, crate::ExtractError::Parse(_)));
}

#[test]
fn rewrites_bare_function_types_in_type_positions() {
    assert_eq!(rewrite("type F = T => U;"), "type F = T  | U;");
    assert_eq!(
        rewrite("function f(cb: Error => void, n: number): string => void {}"),
        "function f(cb: Error  | void, n: number): string  | void {}",
    );
    assert_eq!(rewrite("const g: A => B = h;"), "const g: A  | B = h;");
    // Arrow functions in value position stay.
    let source = "const a = xs.map(x => x + 1);\nconst b = (c): D => c;\n";
    assert_eq!(rewrite(source), source);
}

#[test]
fn rewrites_component_syntax_and_inline_interfaces() {
    assert_eq!(
        rewrite("export component Button(label: string) renders Text {}"),
        "export function  Button(label: string)              {}",
    );
    assert_eq!(rewrite("type C = component(x: T);"), "type C = any            ;");
    assert_eq!(
        rewrite("type E = Error & interface { code: string };"),
        "type E = Error &           { code: string };"
    );
}

#[test]
fn rewrites_declaration_file_constructs() {
    assert_eq!(rewrite("declare module.exports: Foo;"), "declare const  exports: Foo;");
    assert_eq!(rewrite("declare opaque type Id;"), "declare type Id = any ;");
    assert_eq!(
        rewrite("declare class A { constructor(x: T): void; @@iterator(): Iterator<T>; }"),
        "declare class A { constructor(x: T)      ;                          ; }",
    );
    assert_eq!(
        rewrite("declare module 'm' {\n  declare function f(): void;\n}"),
        "declare module 'm' {\n          function f(): void;\n}",
    );
}

#[test]
fn rewrites_parameter_and_type_quirks() {
    assert_eq!(
        rewrite("function f(a?: T = 1, b?: T, c: T) {}"),
        "function f(a : T = 1, b : T, c: T) {}"
    );
    assert_eq!(rewrite("type A = Foo<>;"), "type A = Foo  ;");
    assert_eq!(rewrite("type M = {[K in Keys]: V, };"), "type M = {[K in Keys]: V; };");
    assert_eq!(rewrite("type I = {[keyof T]: V};"), "type I = {[k      ]: V};");
    assert_eq!(rewrite("class A { static +B: T; +#c: T; }"), "class A { static  B: T;  #c: T; }");
}

#[test]
fn recovers_from_statements_the_rewrite_cannot_express() {
    let source = r"// @flow

/** Reads the value. */
export hook useValue(key: string): string {
  return key;
}

/** Formats a value. */
export function format(value: ?string): string {
  return value ?? '';
}
";
    let items = DocExtractor::new().extract_source(source, "hooks.js", SourceType::mjs()).unwrap();
    let format = items.iter().find(|item| item.name == "format").unwrap();
    assert_eq!(format.params[0].type_annotation.as_deref(), Some("?string"));
}
