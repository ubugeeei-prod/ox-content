use super::*;

#[test]
fn typedoc_sorts_interface_members_alphabetically() {
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    // Declared out of alphabetical order.
    entry.members = vec![
        member("zebra", "property", false),
        member("apple", "property", false),
        member("mango", "property", false),
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    let pos = |name: &str| page.find(&join3("`", name, "`")).unwrap();
    assert!(pos("apple") < pos("mango"));
    assert!(pos("mango") < pos("zebra"));
}

#[test]
fn typedoc_sorts_class_members_within_each_group() {
    let mut entry = test_entry("Engine", "class", "/repo/src/engine.ts", "Engine.");
    entry.members = vec![
        member("zeta", "property", false),
        member("alpha", "property", false),
        member("run", "method", false),
        member("build", "method", false),
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/classes/Engine.md").unwrap();

    let property_pos = |name: &str| page.find(&join3("`", name, "`")).unwrap();
    let method_pos = |name: &str| page.find(&join3("### ", name, "()")).unwrap();
    // Properties group alphabetical.
    assert!(property_pos("alpha") < property_pos("zeta"));
    // Methods group alphabetical.
    assert!(method_pos("build") < method_pos("run"));
}

#[test]
fn typedoc_keeps_enum_members_in_declaration_order() {
    let mut entry = test_entry("Level", "enum", "/repo/src/level.ts", "Level.");
    // Non-alphabetical declaration order, which must be preserved for enums.
    entry.members = vec![
        member("Medium", "enumMember", false),
        member("High", "enumMember", false),
        member("Low", "enumMember", false),
    ];
    let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
    let page = out.get("combinators/enumerations/Level.md").unwrap();

    let pos = |name: &str| page.find(&join3("`", name, "`")).unwrap();
    assert!(pos("Medium") < pos("High"));
    assert!(pos("High") < pos("Low"));
}

#[test]
fn typedoc_html_sorts_interface_members_alphabetically() {
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    entry.members = vec![
        member("zebra", "property", false),
        member("apple", "property", false),
        member("mango", "property", false),
    ];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    let pos = |name: &str| page.find(&join3("<code>", name, "</code>")).unwrap();
    assert!(pos("apple") < pos("mango"));
    assert!(pos("mango") < pos("zebra"));
}

#[test]
fn typedoc_member_table_drops_kind_for_named_groups() {
    // The `Properties` heading already states the kind, so the table omits it.
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    entry.members = vec![member("name", "property", false)];
    let options = MarkdownDocsOptions {
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&lifecycle_module(entry), &options);
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    assert!(page.contains("| Name | Type | Description |"));
    assert!(!page.contains("| Name | Kind | Type | Description |"));
    // The redundant per-row kind cell is gone too.
    assert!(!page.contains("| property |"));
}

#[test]
fn typedoc_enum_member_table_drops_kind() {
    let mut entry = test_entry("Level", "enum", "/repo/src/level.ts", "Level.");
    entry.members = vec![member("Low", "enumMember", false)];
    let options = MarkdownDocsOptions {
        enum_members_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&lifecycle_module(entry), &options);
    let page = out.get("combinators/enumerations/Level.md").unwrap();

    assert!(page.contains("| Name | Type | Description |"));
    assert!(!page.contains("| Name | Kind | Type | Description |"));
}

#[test]
fn typedoc_html_member_table_drops_kind_for_named_groups() {
    let mut entry = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
    entry.members = vec![member("name", "property", false)];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    let page = out.get("combinators/interfaces/Command.md").unwrap();

    assert!(page.contains("<th>Name</th><th>Type</th><th>Description</th>"));
    assert!(!page.contains("<th>Kind</th>"));
    assert!(!page.contains("ox-api-entry__member-kind"));
}

#[test]
fn typedoc_html_enum_member_table_drops_kind() {
    // The html renderer groups enum members under `Enum Members` (parity with
    // the markdown renderer), so the redundant Kind column is dropped.
    let mut entry = test_entry("Level", "enum", "/repo/src/level.ts", "Level.");
    entry.members = vec![member("Low", "enumMember", false), member("High", "enumMember", false)];
    let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
    let page = out.get("combinators/enumerations/Level.md").unwrap();

    assert!(page.contains("<h5>Enum Members</h5>"));
    assert!(page.contains("<th>Name</th><th>Type</th><th>Description</th>"));
    assert!(!page.contains("<th>Kind</th>"));
}
