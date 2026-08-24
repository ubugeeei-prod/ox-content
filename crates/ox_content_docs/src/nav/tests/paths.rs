use super::super::*;

#[test]
fn generates_nav_metadata_from_file_paths() {
    let nav = generate_nav_metadata(
        &[
            "/repo/src/types.ts".to_string(),
            "/repo/src/index.ts".to_string(),
            "/repo/src/nav-generator.ts".to_string(),
        ],
        Some("/api"),
    );

    assert_eq!(
        nav,
        vec![
            DocsNavItem {
                title: "Nav Generator".to_string(),
                path: "/api/nav-generator".to_string(),
                ..DocsNavItem::default()
            },
            DocsNavItem {
                title: "Overview".to_string(),
                path: "/api/index".to_string(),
                ..DocsNavItem::default()
            },
            DocsNavItem {
                title: "Types".to_string(),
                path: "/api/types".to_string(),
                ..DocsNavItem::default()
            },
        ]
    );
}

#[test]
fn normalizes_nav_base_path() {
    let nav = generate_nav_metadata(&["/repo/src/context.ts".to_string()], Some("api-ox/"));

    assert_eq!(nav[0].path, "/api-ox/context");
}

#[test]
fn flat_nav_uses_collision_free_generated_module_routes() {
    let docs = [
        ApiDocModule { file: "/repo/src/index.ts".to_string(), ..ApiDocModule::default() },
        ApiDocModule { file: "/repo/src/plugins/index.ts".to_string(), ..ApiDocModule::default() },
    ];

    let nav = generate_nav_metadata_from_docs(
        &docs,
        Some("/api"),
        MarkdownPathStrategy::Flat,
        None,
        None,
        true,
        None,
    );

    assert_eq!(nav[0].path, "/api/index-module");
    assert_eq!(nav[1].path, "/api/plugins-index");
}

#[test]
fn generates_nav_code() {
    let code = generate_nav_code(
        &[DocsNavItem {
            title: "Docs".to_string(),
            path: "/api/docs".to_string(),
            ..DocsNavItem::default()
        }],
        Some("apiNav"),
    );

    insta::assert_snapshot!(code);
}
