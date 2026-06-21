use super::*;

#[test]
fn prepare_source_returns_object_shaped_frontmatter_and_origin() {
    let result = crate::prepare_source(
        "---\ntitle: Guide\nmeta:\n  draft: false\n---\n# Body".to_string(),
        None,
    );

    assert_eq!(result.content, "# Body");
    assert_eq!(result.frontmatter.get("title"), Some(&json!("Guide")));
    assert_eq!(result.frontmatter.get("meta"), Some(&json!({"draft": false})));
    assert_eq!(result.source_offset.line, 6);
    assert_eq!(result.source_offset.column, 1);
}

#[test]
fn javascript_wrapper_and_declarations_cover_expected_exports() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_js = fs::read_to_string(manifest_dir.join("index.js")).unwrap();
    let declarations = fs::read_to_string(manifest_dir.join("index.d.ts")).unwrap();
    let expected_exports = [
        "buildSearchIndex",
        "buildSearchIndexFromDirectory",
        "buildSsgNavItems",
        "buildSsgThemeNavItems",
        "buildExportGraph",
        "checkI18n",
        "checkI18nProject",
        "collectDocsSourceFiles",
        "collectSearchMarkdownFiles",
        "collectSsgMarkdownFiles",
        "escapeSvelteMarkup",
        "externalizeSsgAssets",
        "extractDocsFromDirectories",
        "extractDocsFromEntryPoints",
        "extractFileDocEntries",
        "extractFileDocs",
        "extractSearchContent",
        "extractSsgTitle",
        "extractTranslationKeys",
        "formatSsgTitle",
        "generateDocsDataJson",
        "generateDocsMarkdown",
        "generateDocsNavCode",
        "generateDocsNavMetadata",
        "generateDocsNavMetadataFromDocs",
        "generateI18nModule",
        "generateOgImageSvg",
        "generateSearchModule",
        "generateSearchModuleFromOptions",
        "generateSsgBareHtml",
        "generateSsgHtml",
        "getGitLastUpdated",
        "getSearchDocumentScopes",
        "getSsgHref",
        "getSsgOutputPath",
        "getSsgPageLocale",
        "getSsgUrlPath",
        "lintMarkdown",
        "lintMarkdownDocuments",
        "loadDictionaries",
        "loadDictionariesFlat",
        "matchesSearchScopes",
        "mergeHighlightedCodeBlocks",
        "normalizeVitePressFrontmatter",
        "parse",
        "parseAndRender",
        "parseAndRenderAsync",
        "parseMdastRaw",
        "parseScopedSearchQuery",
        "parseTransferRaw",
        "prepareSource",
        "prepareSourceRaw",
        "render",
        "renderFrameworkComponentCode",
        "resolveSsgNavigationGroups",
        "resolveSsgRoutePaths",
        "searchIndex",
        "transform",
        "transformAsync",
        "transformMdastRaw",
        "transformMermaid",
        "validateMf2",
        "version",
        "writeGeneratedDocs",
        "writeSearchIndex",
    ];

    insta::assert_snapshot!("javascript_wrapper_expected_exports", expected_exports.join("\n"));
    insta::assert_snapshot!("javascript_wrapper_index_js", index_js);
    insta::assert_snapshot!("javascript_wrapper_declarations", declarations);
}

#[test]
fn javascript_wrapper_reports_native_load_errors() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_js = fs::read_to_string(manifest_dir.join("index.js")).unwrap();

    insta::assert_snapshot!("javascript_wrapper_native_load_errors", index_js);
}

#[test]
fn transform_passes_toc_depth_to_inline_toc() {
    let result = crate::transform(
        "[[toc]]\n\n## Intro\n### API".to_string(),
        Some(crate::JsTransformOptions { toc_max_depth: Some(2), ..Default::default() }),
    );

    insta::assert_snapshot!(result.html);
}

#[test]
fn builds_search_index_from_directory() {
    let root = std::env::temp_dir().join(format!("ox-content-napi-search-{}", std::process::id()));
    let docs_dir = root.join("docs");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(docs_dir.join("guide")).unwrap();
    fs::write(
        docs_dir.join("guide/intro.markdown"),
        "---\ntitle: Native Search\n---\n# Intro\n\nSearch body text.",
    )
    .unwrap();

    let index_json = crate::build_search_index_from_directory(
        docs_dir.to_string_lossy().into_owned(),
        "/docs/".to_string(),
        vec![".md".to_string(), ".markdown".to_string()],
    );
    let index = ox_content_search::SearchIndex::from_json(&index_json).unwrap();

    assert_eq!(index.doc_count, 1);
    assert_eq!(index.documents[0].id, "guide/intro");
    assert_eq!(index.documents[0].title, "Native Search");
    assert_eq!(index.documents[0].url, "/docs/guide/intro");
    insta::assert_snapshot!(index.documents[0].body);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn writes_search_index_through_napi() {
    let root =
        std::env::temp_dir().join(format!("ox-content-napi-search-out-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    crate::write_search_index(r#"{"doc_count":0}"#.to_string(), root.to_string_lossy().into())
        .unwrap();

    assert_eq!(fs::read_to_string(root.join("search-index.json")).unwrap(), r#"{"doc_count":0}"#);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn check_i18n_project_collects_source_and_markdown_keys() {
    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root =
        std::env::temp_dir().join(format!("ox-content-napi-i18n-{}-{unique}", std::process::id()));
    let dict_root = root.join("content/i18n");
    let src_dir = root.join("src");
    let content_dir = root.join("content");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(dict_root.join("en")).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        dict_root.join("en/common.json"),
        r#"{"fromSrc":"From source","fromMd":"From markdown"}"#,
    )
    .unwrap();
    fs::write(src_dir.join("app.ts"), "const label = t('common.fromSrc');").unwrap();
    fs::write(content_dir.join("guide.md"), "{{t('common.fromMd')}}").unwrap();

    let result = crate::check_i18n_project(
        dict_root.to_string_lossy().into_owned(),
        vec![src_dir.to_string_lossy().into_owned(), content_dir.to_string_lossy().into_owned()],
        vec!["t".to_string(), "$t".to_string()],
        "en".to_string(),
    );
    let messages: Vec<&str> = result.diagnostics.iter().map(|d| d.message.as_str()).collect();

    assert_eq!(result.error_count, 0, "diagnostics: {messages:?}");
    assert_eq!(result.warning_count, 0, "diagnostics: {messages:?}");
    assert!(result.diagnostics.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn generates_search_module_from_typed_options() {
    let module = crate::generate_search_module_from_options(
        crate::JsSearchRuntimeOptions {
            enabled: true,
            limit: 7,
            prefix: false,
            placeholder: "Find".to_string(),
            hotkey: "k".to_string(),
        },
        "/docs/search-index.json".to_string(),
    );

    insta::assert_snapshot!(module);
}

#[test]
fn git_last_updated_uses_root_relative_path() {
    let root = std::env::temp_dir().join(format!("ox-content-git-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/page.md"), "# Page").unwrap();

    for args in [
        vec!["init"],
        vec!["add", "docs/page.md"],
        vec!["-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-m", "init"],
    ] {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&root).args(args);
        cmd.env("GIT_AUTHOR_DATE", "@1234567890");
        cmd.env("GIT_COMMITTER_DATE", "@1234567890");
        assert!(cmd.status().unwrap().success());
    }

    let updated = get_git_last_updated(
        root.join("docs/page.md").to_string_lossy().into_owned(),
        Some(root.to_string_lossy().into_owned()),
    );
    assert_eq!(updated, Some(1_234_567_890_000.0));
    let _ = fs::remove_dir_all(root);
}
