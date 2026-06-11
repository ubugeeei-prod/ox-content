use super::*;

#[test]
fn order_by_group_title_none_preserves_order() {
    let sections = vec![("Functions".to_string(), 1), ("Variables".to_string(), 2)];
    assert_eq!(order_by_group_title(sections.clone(), None), sections);
}

#[test]
fn order_by_group_title_orders_listed_then_alphabetical() {
    let sections = vec![
        ("Functions".to_string(), 1),
        ("Classes".to_string(), 2),
        ("Interfaces".to_string(), 3),
        ("References".to_string(), 4),
        ("Type Aliases".to_string(), 5),
        ("Variables".to_string(), 6),
    ];
    let group_order = ["Variables".to_string(), "Functions".to_string(), "Class".to_string()];
    let titles = order_by_group_title(sections, Some(&group_order))
        .into_iter()
        .map(|(title, _)| title)
        .collect::<Vec<_>>();

    // `Class` does not match the `Classes` group and is ignored; unlisted
    // groups (including References) follow alphabetically.
    assert_eq!(
        titles,
        vec!["Variables", "Functions", "Classes", "Interfaces", "References", "Type Aliases"]
    );
}

#[test]
fn order_by_group_title_places_unspecified_at_star() {
    let sections = vec![
        ("Functions".to_string(), 1),
        ("Classes".to_string(), 2),
        ("Variables".to_string(), 3),
    ];
    let group_order = ["Variables".to_string(), "*".to_string(), "Functions".to_string()];
    let titles = order_by_group_title(sections, Some(&group_order))
        .into_iter()
        .map(|(title, _)| title)
        .collect::<Vec<_>>();

    assert_eq!(titles, vec!["Variables", "Classes", "Functions"]);
}

#[test]
fn typedoc_group_order_defaults_to_fixed_kind_order() {
    let out = generate_markdown(&group_order_docs(), &markdown_typedoc_options());
    let index = out.get("default/index.md").unwrap();
    let functions = index.find("## Functions").unwrap();
    let classes = index.find("## Classes").unwrap();
    let interfaces = index.find("## Interfaces").unwrap();
    let variables = index.find("## Variables").unwrap();

    // Unchanged historical order (DOC_KIND_ORDER).
    assert!(functions < classes);
    assert!(classes < interfaces);
    assert!(interfaces < variables);
}

#[test]
fn typedoc_group_order_reorders_module_index_sections() {
    let options = MarkdownDocsOptions {
        group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&group_order_docs(), &options);
    let index = out.get("default/index.md").unwrap();
    let variables = index.find("## Variables").unwrap();
    let functions = index.find("## Functions").unwrap();
    let classes = index.find("## Classes").unwrap();
    let interfaces = index.find("## Interfaces").unwrap();

    // Listed groups lead in order; the rest follow alphabetically.
    assert!(variables < functions);
    assert!(functions < classes);
    assert!(classes < interfaces);
}

#[test]
fn typedoc_group_order_supports_star_wildcard() {
    let options = MarkdownDocsOptions {
        group_order: Some(vec!["Variables".to_string(), "*".to_string(), "Functions".to_string()]),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&group_order_docs(), &options);
    let index = out.get("default/index.md").unwrap();
    let variables = index.find("## Variables").unwrap();
    let classes = index.find("## Classes").unwrap();
    let interfaces = index.find("## Interfaces").unwrap();
    let functions = index.find("## Functions").unwrap();

    // Variables first, unspecified groups (alphabetical) in the middle, Functions last.
    assert!(variables < classes);
    assert!(classes < interfaces);
    assert!(interfaces < functions);
}

#[test]
fn typedoc_kind_sort_order_reorders_module_index_sections() {
    let options = MarkdownDocsOptions {
        kind_sort_order: Some(vec![
            "variable".to_string(),
            "function".to_string(),
            "class".to_string(),
            "interface".to_string(),
        ]),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&group_order_docs(), &options);
    let index = out.get("default/index.md").unwrap();
    let variables = index.find("## Variables").unwrap();
    let functions = index.find("## Functions").unwrap();
    let classes = index.find("## Classes").unwrap();
    let interfaces = index.find("## Interfaces").unwrap();

    // Sections follow the configured kind order.
    assert!(variables < functions);
    assert!(functions < classes);
    assert!(classes < interfaces);
}

#[test]
fn typedoc_group_order_takes_precedence_over_kind_sort_order() {
    let options = MarkdownDocsOptions {
        kind_sort_order: Some(vec![
            "variable".to_string(),
            "function".to_string(),
            "class".to_string(),
            "interface".to_string(),
        ]),
        group_order: Some(vec!["Interfaces".to_string(), "Functions".to_string()]),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&group_order_docs(), &options);
    let index = out.get("default/index.md").unwrap();
    let interfaces = index.find("## Interfaces").unwrap();
    let functions = index.find("## Functions").unwrap();
    let classes = index.find("## Classes").unwrap();
    let variables = index.find("## Variables").unwrap();

    // group_order leads (Interfaces, Functions); unlisted groups follow
    // alphabetically (Classes, Variables) — group_order wins over kind_sort_order.
    assert!(interfaces < functions);
    assert!(functions < classes);
    assert!(classes < variables);
}

#[test]
fn typedoc_sort_source_order_orders_entries_by_line() {
    let mut zebra = test_entry("zebra", "function", "/repo/src/z.ts", "Z.");
    zebra.line = 1;
    let mut alpha = test_entry("alpha", "function", "/repo/src/a.ts", "A.");
    alpha.line = 2;
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![zebra, alpha],
        ..ApiDocModule::default()
    }];

    let options = MarkdownDocsOptions {
        sort: Some(vec!["source-order".to_string()]),
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&docs, &options);
    let index = out.get("default/index.md").unwrap();
    // Source order: `zebra` (line 1) before `alpha` (line 2).
    assert!(index.find("zebra").unwrap() < index.find("alpha").unwrap());

    // The default (alphabetical) ordering is the opposite.
    let default_out = generate_markdown(&docs, &markdown_typedoc_options());
    let default_index = default_out.get("default/index.md").unwrap();
    assert!(default_index.find("alpha").unwrap() < default_index.find("zebra").unwrap());
}

#[test]
fn typedoc_sort_required_first_then_alphabetical_orders_members() {
    let mut iface = test_entry("Opts", "interface", "/repo/src/o.ts", "Options.");
    let mut optional_b = member("b", "property", false);
    optional_b.optional = true;
    let mut required_z = member("z", "property", false);
    required_z.optional = false;
    let mut required_a = member("a", "property", false);
    required_a.optional = false;
    iface.members = vec![optional_b, required_z, required_a];
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![iface],
        ..ApiDocModule::default()
    }];

    let options = MarkdownDocsOptions {
        sort: Some(vec!["required-first".to_string(), "alphabetical".to_string()]),
        interface_properties_format: MarkdownDisplayFormat::Table,
        ..markdown_typedoc_options()
    };
    let out = generate_markdown(&docs, &options);
    let page = out.get("default/interfaces/Opts.md").unwrap();
    let a = page.find("`a`").unwrap();
    let z = page.find("`z`").unwrap();
    let b = page.find("`b`").unwrap();

    // Required members first (a, z), then optional (b); the later `alphabetical`
    // strategy breaks the tie between the two required members.
    assert!(a < z);
    assert!(z < b);
}
