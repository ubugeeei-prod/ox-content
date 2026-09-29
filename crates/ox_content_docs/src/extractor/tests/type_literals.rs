use super::super::*;

#[test]
fn type_alias_object_literal_emits_property_children() {
    let source = r"
/**
 * Command options.
 */
export type CommandOptions = {
    name: string;
    aliases?: string[];
};
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "CommandOptions");
    assert_eq!(items[0].children.len(), 2);
    assert_eq!(items[0].children[0].name, "name");
    assert_eq!(items[0].children[0].kind, DocItemKind::Property);
    assert_eq!(items[0].children[0].signature.as_deref(), Some("string"));
    assert!(!items[0].children[0].optional);
    assert_eq!(items[0].children[1].name, "aliases");
    assert_eq!(items[0].children[1].signature.as_deref(), Some("string[]"));
    assert!(items[0].children[1].optional);
}

#[test]
fn interface_property_type_literal_emits_property_members() {
    let source = r"
/**
 * Request options.
 */
export interface RequestOptions {
    /** HTTP options. */
    http: {
        /** Request timeout. */
        timeout?: number;
        /** Request headers. */
        headers: Record<string, string>;
    };
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "request.ts", SourceType::ts()).unwrap();
    let http = &items[0].children[0];

    assert_eq!(http.name, "http");
    assert_eq!(http.kind, DocItemKind::Property);
    assert_eq!(http.children.len(), 2);
    assert_eq!(http.children[0].name, "timeout");
    assert_eq!(http.children[0].doc.as_deref(), Some("Request timeout."));
    assert_eq!(http.children[0].signature.as_deref(), Some("number"));
    assert!(http.children[0].optional);
    assert_eq!(http.children[1].name, "headers");
    assert_eq!(http.children[1].signature.as_deref(), Some("Record<string, string>"));
}

#[test]
fn type_alias_signature_omits_nested_property_jsdoc_comments() {
    let source = r"
/**
 * A combinator produced by combinator factory functions.
 */
export type Combinator<T> = {
    /**
     * The parse function that converts a string to the desired type.
     *
     * @param value - The input string value.
     * @returns The parsed value of type T.
     */
    parse: (value: string) => T;
};
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "combinators.ts", SourceType::ts()).unwrap();
    let combinator = items.iter().find(|item| item.name == "Combinator").unwrap();
    let signature = combinator.signature.as_deref().unwrap();

    assert_eq!(signature, "export type Combinator<T> = { parse: (value: string) => T }");

    let parse = &combinator.children[0];
    assert_eq!(
        parse.doc.as_deref(),
        Some("The parse function that converts a string to the desired type.")
    );
    assert_eq!(parse.signature.as_deref(), Some("(value: string) => T"));
    assert_eq!(parse.params[0].name, "value");
    assert_eq!(parse.params[0].description.as_deref(), Some("The input string value."));
    assert_eq!(parse.return_type.as_deref(), Some("T"));
    let returns_tag = parse.tags.iter().find(|tag| tag.tag == "returns").unwrap();
    assert_eq!(returns_tag.description.as_deref(), Some("The parsed value of type T."));
}

#[test]
fn type_alias_object_literal_with_method_signature() {
    let source = r"
/**
 * Command options.
 */
export type CommandOptions = {
    /**
     * Runs the command.
     * @param ctx - Runtime context
     */
    run(ctx: Context): void;
};
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
    let signature = items[0].signature.as_deref().unwrap();
    let member = &items[0].children[0];

    assert_eq!(signature, "export type CommandOptions = { run(ctx: Context): void }");
    assert_eq!(member.name, "run");
    assert_eq!(member.kind, DocItemKind::Method);
    assert_eq!(member.signature.as_deref(), Some("run(ctx: Context): void"));
    assert_eq!(member.params.len(), 1);
    assert_eq!(member.params[0].description.as_deref(), Some("Runtime context"));
}

fn param_types(source: &str) -> Vec<String> {
    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "use.ts", SourceType::ts()).unwrap();
    // A parameter of an object literal type also gives a row per member (`flat.args`).
    items[0]
        .params
        .iter()
        .filter(|param| !param.name.contains('.'))
        .map(|param| param.type_annotation.clone().unwrap_or_default())
        .collect()
}

#[test]
fn keeps_the_members_of_a_type_literal_inside_a_type_apart() {
    let source = r"
/**
 * Use.
 */
export function useIt(
  list: Array<{
    args: Args
    label: string
  }>,
  cond: Args extends object
    ? {
        args: Args
        label: string
      }
    : never,
  map: {
    [K in keyof Args]: {
      args: Args
      label: string
    }
  },
  pair: [first: {
    args: Args
    label: string
  }],
): void {}
";
    assert_eq!(
        param_types(source),
        [
            "Array<{ args: Args; label: string }>",
            "Args extends object ? { args: Args; label: string } : never",
            "{ [K in keyof Args]: { args: Args; label: string } }",
            "[first: { args: Args; label: string }]",
        ]
    );
}

#[test]
fn leaves_member_comments_out_of_a_type_literal_inside_a_type() {
    let source = r"
/**
 * Use.
 */
export function useIt(
  list: Array<{
    /** The args. */
    args: Args, label: string
  }>,
  flat: {
    /** The args. */
    args: Args, label: string
  },
): void {}
";
    assert_eq!(
        param_types(source),
        ["Array<{ args: Args; label: string }>", "{ args: Args; label: string }"]
    );
}

#[test]
fn keeps_the_members_of_a_type_literal_inside_a_type_as_their_source() {
    let source = r#"
/**
 * Use.
 */
export function useIt(
  literals: Array<{
    sep: "\\"
    quote: '"'
    code: -1 | 1n
  }>,
  keys: Partial<{
    'content-type': string,
    <E>(event: E): void
    new (value: string): Args
  }>,
  functions: Partial<{
    map: <U extends Args>(value: U) => U
    onClick(this: Args, [first]: Args[]): void
    pick<U extends { a: string
      b: number }>(value: U): U
  }>,
  accessors: Array<{
    get size(): number
    set size(value: number)
    loose()
    any
  }>,
): void {}
"#;
    assert_eq!(
        param_types(source),
        [
            r#"Array<{ sep: "\\"; quote: '"'; code: -1 | 1n }>"#,
            "Partial<{ 'content-type': string; <E>(event: E): void; new (value: string): Args }>",
            "Partial<{ map: <U extends Args>(value: U) => U; onClick(this: Args, [first]: Args[]): void; pick<U extends { a: string; b: number }>(value: U): U }>",
            "Array<{ get size(): number; set size(value: number); loose(); any }>",
        ]
    );
}

#[test]
fn keeps_a_type_without_a_type_literal_as_its_source() {
    let source = r"
/**
 * Use.
 */
export function useIt(
  map: Map<
    string,
    number
  >,
  cond: Args extends object ? 'yes' : 'no',
): void {}
";
    assert_eq!(param_types(source), ["Map<string, number>", "Args extends object ? 'yes' : 'no'"]);
}

#[test]
fn keeps_the_members_of_a_type_parameter_constraint_apart() {
    let source = r"
/**
 * Params.
 */
export interface Params<
  P extends {
    args?: Args
    label?: string
  } = {
    args: Args
    label: string
  },
  Q extends Record<
    string,
    unknown
  > = {}
> {
  params: P
}
";
    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "params.ts", SourceType::ts()).unwrap();
    let type_parameters = &items[0].type_parameters;

    assert_eq!(type_parameters.len(), 2);
    assert_eq!(type_parameters[0].constraint.as_deref(), Some("{ args?: Args; label?: string }"));
    assert_eq!(type_parameters[0].default.as_deref(), Some("{ args: Args; label: string }"));
    assert_eq!(type_parameters[1].constraint.as_deref(), Some("Record<string, unknown>"));
    assert_eq!(type_parameters[1].default.as_deref(), Some("{}"));
}
