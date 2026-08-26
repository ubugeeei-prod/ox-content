use ox_content_transform::PublishStateOptions;

use super::{SearchIndexBuildOptions, build_search_index_from_directory_with_options};

fn write_tree(root: &std::path::Path, files: &[(&str, &str)]) {
    for (relative, source) in files {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, source).unwrap();
    }
}

fn titles(index_json: &str) -> Vec<String> {
    let index: serde_json::Value = serde_json::from_str(index_json).unwrap();
    index["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|doc| doc["title"].as_str().unwrap().to_string())
        .collect()
}

fn body(index_json: &str) -> String {
    let index: serde_json::Value = serde_json::from_str(index_json).unwrap();
    index["documents"][0]["body"].as_str().unwrap().to_string()
}

#[test]
fn mdx_extension_enables_mdx_and_indexes_component_children() {
    let root = std::env::temp_dir().join(format!("ox-search-mdx-auto-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[(
            "guide.MDX",
            "import Card from './Card'\n\n# Guide\n\n<Card>Visible search copy</Card>\n",
        )],
    );

    let body = body(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".mdx".to_string()],
        None,
    ));
    assert_eq!(body, "Visible search copy");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn explicit_mdx_setting_overrides_the_source_extension() {
    let root = std::env::temp_dir().join(format!("ox-search-mdx-override-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let source = "import Card from './Card'\n\n# Guide\n\n<Card>Visible search copy</Card>\n";
    write_tree(&root, &[("enabled.md", source), ("disabled.mdx", source)]);

    let enabled = body(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
        Some(&SearchIndexBuildOptions { mdx: Some(true), ..SearchIndexBuildOptions::default() }),
    ));
    assert_eq!(enabled, "Visible search copy");

    let disabled = body(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".mdx".to_string()],
        Some(&SearchIndexBuildOptions { mdx: Some(false), ..SearchIndexBuildOptions::default() }),
    ));
    assert!(disabled.contains("import Card"), "{disabled:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn timeline_markdown_items_stay_searchable_in_source_order() {
    let root = std::env::temp_dir().join(format!("ox-search-timeline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[(
            "releases.md",
            "# Releases\n\n::: timeline\n- 2026-08-26 RC cut\n  Nested migration notes.\n- 2026-09 GA window\n:::\n",
        )],
    );

    let body = body(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
        None,
    ));
    let rc = body.find("RC cut").expect("timeline title indexed");
    let nested = body.find("Nested migration notes").expect("nested body indexed");
    let ga = body.find("GA window").expect("later item indexed");
    assert!(rc < nested && nested < ga, "{body:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn off_indexes_every_page() {
    let root = std::env::temp_dir().join(format!("ox-search-drafts-off-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[
            ("public.md", "---\ntitle: Public\n---\n# Public\n"),
            ("draft.md", "---\ntitle: Draft\ndraft: true\n---\n# Draft\n"),
        ],
    );

    let titles = titles(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
        None,
    ));
    assert!(titles.contains(&"Public".to_string()), "{titles:?}");
    assert!(titles.contains(&"Draft".to_string()), "{titles:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn on_omits_draft_unlisted_and_scheduled() {
    let root = std::env::temp_dir().join(format!("ox-search-drafts-on-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[
            ("public.md", "---\ntitle: Public\n---\n# Public\n"),
            ("draft.md", "---\ntitle: Draft\ndraft: true\n---\n# Draft\n"),
            ("hidden.md", "---\ntitle: Hidden\nunlisted: true\n---\n# Hidden\n"),
            ("later.md", "---\ntitle: Later\nscheduled: 2099-01-01T00:00:00Z\n---\n# Later\n"),
            (
                "xss.md",
                "---\ntitle: \"</script><script>alert(1)</script>\"\ndraft: true\n---\n# X\n",
            ),
        ],
    );

    let titles = titles(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
        Some(&SearchIndexBuildOptions {
            publish_state: Some(PublishStateOptions {
                enabled: true,
                now: Some("2026-08-24T00:00:00Z".to_string()),
                include_drafts: false,
            }),
            ..SearchIndexBuildOptions::default()
        }),
    ));
    assert_eq!(titles, vec!["Public".to_string()]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn include_drafts_keeps_wip_out_of_unlisted() {
    let root = std::env::temp_dir().join(format!("ox-search-drafts-dev-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[
            ("draft.md", "---\ntitle: Draft\ndraft: true\n---\n# Draft\n"),
            ("hidden.md", "---\ntitle: Hidden\nunlisted: true\n---\n# Hidden\n"),
        ],
    );

    let titles = titles(&build_search_index_from_directory_with_options(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
        Some(&SearchIndexBuildOptions {
            publish_state: Some(PublishStateOptions {
                enabled: true,
                now: Some("2026-08-24T00:00:00Z".to_string()),
                include_drafts: true,
            }),
            ..SearchIndexBuildOptions::default()
        }),
    ));
    assert_eq!(titles, vec!["Draft".to_string()]);
    let _ = std::fs::remove_dir_all(&root);
}
