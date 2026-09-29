use super::*;

/// A module with `entries` next to types named like type parameters (`T`, `U`,
/// `K`), and the types `Box` and `Options`.
fn type_parameter_module(mut entries: Vec<ApiDocEntry>) -> Vec<ApiDocModule> {
    for name in ["T", "U", "K", "Box", "Options"] {
        entries.push(type_stub(name));
    }
    vec![ApiDocModule { file: "combinators".to_string(), entries, ..ApiDocModule::default() }]
}

fn markdown_table() -> (&'static str, MarkdownDocsOptions) {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    ("markdown_table", options)
}

fn markdown_list() -> (&'static str, MarkdownDocsOptions) {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::List,
        interface_properties_format: MarkdownDisplayFormat::List,
        ..markdown_typedoc_options()
    };
    ("markdown_list", options)
}

fn html_table() -> (&'static str, MarkdownDocsOptions) {
    let options = MarkdownDocsOptions {
        parameters_format: MarkdownDisplayFormat::Table,
        property_members_format: MarkdownDisplayFormat::Table,
        ..html_typedoc_options()
    };
    ("html_table", options)
}

fn html_list() -> (&'static str, MarkdownDocsOptions) {
    let options = MarkdownDocsOptions {
        interface_properties_format: MarkdownDisplayFormat::List,
        property_members_format: MarkdownDisplayFormat::List,
        ..html_typedoc_options()
    };
    ("html_list", options)
}

/// Renders `docs` in each of `formats`, as the snapshot `<name>__<format>`.
fn assert_format_snapshots(
    name: &str,
    docs: &[ApiDocModule],
    formats: &[(&'static str, MarkdownDocsOptions)],
) {
    for (format, options) in formats {
        let out = generate_markdown(docs, options);
        assert_markdown_map_snapshot(&format!("{name}__{format}"), &out);
    }
}

fn type_params(names: &[&str]) -> Vec<ApiTypeParamDoc> {
    names
        .iter()
        .map(|name| ApiTypeParamDoc { name: (*name).to_string(), ..ApiTypeParamDoc::default() })
        .collect()
}

fn returns(type_annotation: &str) -> Option<ApiReturnDoc> {
    Some(ApiReturnDoc { type_annotation: type_annotation.to_string(), ..ApiReturnDoc::default() })
}

#[test]
fn typedoc_does_not_link_type_parameters_of_the_declaration() {
    // `U` in `id()` is its type parameter, but `Box` is still a link.
    let mut id = test_entry("id", "function", "/repo/src/id.ts", "Identity.");
    id.type_parameters = type_params(&["U"]);
    id.params = vec![param("x", "U"), param("list", "Box<U>")];
    id.returns = returns("U");
    // `other()` has no type parameter `U`, so its `U` is the type `U`.
    let mut other = test_entry("other", "function", "/repo/src/id.ts", "Other.");
    other.params = vec![param("x", "U")];
    assert_format_snapshots(
        "typedoc_does_not_link_type_parameters_of_the_declaration",
        &type_parameter_module(vec![id, other]),
        &[markdown_table(), html_table()],
    );
}

#[test]
fn typedoc_does_not_link_type_parameters_of_an_interface_and_its_method() {
    let mut boxed = test_entry("Boxed", "interface", "/repo/src/box.ts", "A box.");
    boxed.type_parameters = type_params(&["T"]);
    let mut value = member("value", "property", false);
    value.description = "The value.".to_string();
    value.type_annotation = Some("T".to_string());
    let mut map = member("map", "method", false);
    map.description = "Maps the value.".to_string();
    map.signature = Some("map<U>(fn: (value: T) => U): Box<U>".to_string());
    map.type_parameters = type_params(&["U"]);
    map.params = vec![param("fn", "(value: T) => U")];
    map.returns = returns("Box<U>");
    // A property of a generic function type has type parameters of its own too.
    let mut transform = member("transform", "property", false);
    transform.description = "Transforms the value.".to_string();
    transform.type_annotation = Some("(value: T, other: U) => U".to_string());
    transform.type_parameters = type_params(&["U"]);
    transform.params = vec![param("value", "T"), param("other", "U")];
    transform.returns = returns("U");
    boxed.members = vec![value, map, transform];
    assert_format_snapshots(
        "typedoc_does_not_link_type_parameters_of_an_interface_and_its_method",
        &type_parameter_module(vec![boxed]),
        &[markdown_table(), markdown_list(), html_table(), html_list()],
    );
}

#[test]
fn typedoc_does_not_link_type_parameters_of_a_return_member() {
    // A member of an object return type, of a generic function type. Only HTML
    // renders the type of a return member apart from the signature code block.
    let mut map = return_property("map", "(value: U) => U");
    map.type_parameters = type_params(&["U"]);
    map.params = vec![param("value", "U")];
    let mut make = test_entry("make", "function", "/repo/src/make.ts", "Make.");
    make.returns = Some(ApiReturnDoc {
        type_annotation: "object".to_string(),
        description: "The maker.".to_string(),
        members: vec![map],
    });
    assert_format_snapshots(
        "typedoc_does_not_link_type_parameters_of_a_return_member",
        &type_parameter_module(vec![make]),
        &[html_table()],
    );
}

#[test]
fn typedoc_does_not_link_type_parameters_of_a_nested_member() {
    // A member of an object type property, of a generic function type. Only HTML
    // renders the members of a property.
    let mut map = return_property("map", "(value: U) => U");
    map.type_parameters = type_params(&["U"]);
    map.params = vec![param("value", "U")];
    let mut handlers = member("handlers", "property", false);
    handlers.description = "The handlers.".to_string();
    handlers.type_annotation = Some("Handlers".to_string());
    handlers.members = vec![map];
    let mut settings = test_entry("Settings", "interface", "/repo/src/settings.ts", "Settings.");
    settings.members = vec![handlers];
    assert_format_snapshots(
        "typedoc_does_not_link_type_parameters_of_a_nested_member",
        &type_parameter_module(vec![settings]),
        &[html_table(), html_list()],
    );
}

#[test]
fn typedoc_does_not_link_type_parameters_of_an_overload() {
    let mut first = overload_entry(
        "pick",
        "/repo/src/pick.ts",
        "Pick one.",
        "export function pick<U>(x: U): U",
        false,
    );
    first.type_parameters = type_params(&["U"]);
    first.params = vec![param("x", "U")];
    first.returns = returns("U");
    let mut second = overload_entry(
        "pick",
        "/repo/src/pick.ts",
        "Pick two.",
        "export function pick<U>(x: U, y: U): U",
        false,
    );
    second.type_parameters = type_params(&["U"]);
    second.params = vec![param("x", "U"), param("y", "U")];
    second.returns = returns("U");
    let implementation = overload_entry(
        "pick",
        "/repo/src/pick.ts",
        "Pick.",
        "export function pick(x: any, y?: any): any",
        true,
    );
    let mut docs = overload_module(vec![first, second, implementation]);
    docs[0].entries.push(type_stub("U"));
    assert_format_snapshots(
        "typedoc_does_not_link_type_parameters_of_an_overload",
        &docs,
        &[markdown_table(), html_table()],
    );
}
