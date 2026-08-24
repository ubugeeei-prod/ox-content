use ox_content_transform::PublishStateOptions;

use super::{
    SearchIndexBuildOptions, build_search_index_from_directory,
    build_search_index_from_directory_with_options,
};

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
        }),
    ));
    assert_eq!(titles, vec!["Draft".to_string()]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn omits_404_and_noindex_documents() {
    let root =
        std::env::temp_dir().join(format!("ox-content-search-not-found-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_tree(
        &root,
        &[
            ("guide.md", "---\ntitle: Guide\n---\n\nPublished.\n"),
            ("404.md", "---\ntitle: Lost\n---\n\nNot indexed.\n"),
            ("secret.md", "---\ntitle: Secret\nnoindex: true\n---\n\nHidden.\n"),
        ],
    );

    let titles = titles(&build_search_index_from_directory(
        root.to_str().unwrap(),
        "/",
        &[".md".to_string()],
    ));
    assert_eq!(titles, vec!["Guide".to_string()]);
    let _ = std::fs::remove_dir_all(&root);
}
