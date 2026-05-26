use std::fs;
use std::path::PathBuf;

use super::*;

fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ox-content-assets-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

mod detect_context_markdown {
    use super::*;

    #[test]
    fn link_opener_returns_link_context_with_empty_partial() {
        let (ctx, partial) = detect_context("see also [docs](").expect("detected");
        assert_eq!(ctx, AssetContext::Link);
        assert_eq!(partial, "");
    }

    #[test]
    fn image_opener_returns_image_context() {
        let (ctx, partial) = detect_context("![alt](./").expect("detected");
        assert_eq!(ctx, AssetContext::Image);
        assert_eq!(partial, "./");
    }

    #[test]
    fn partial_path_after_directory_separator() {
        let (ctx, partial) = detect_context("[guide](./docs/get").expect("detected");
        assert_eq!(ctx, AssetContext::Link);
        assert_eq!(partial, "./docs/get");
    }

    #[test]
    fn space_inside_parens_means_we_are_not_in_a_path() {
        // `[text](https://… "title goes here")` — once we hit a space
        // we know the cursor is in the title slot, not the URL.
        assert!(detect_context("[home](https://example.com \"Title").is_none());
    }

    #[test]
    fn lone_open_paren_without_a_link_bracket_is_ignored() {
        assert!(detect_context("plain (text").is_none());
    }

    #[test]
    fn balanced_inner_parens_do_not_break_detection() {
        // The shortcut `[txt](url-with-(paren))` is rare but should
        // still detect when we're past the inner balanced pair.
        let (_, partial) = detect_context("[x](outer(inner)/").expect("detected");
        assert_eq!(partial, "outer(inner)/");
    }
}

mod detect_context_html {
    use super::*;

    #[test]
    fn img_src_returns_image_context() {
        let (ctx, partial) = detect_context(r#"<img src=""#).expect("detected");
        assert_eq!(ctx, AssetContext::Image);
        assert_eq!(partial, "");
    }

    #[test]
    fn a_href_returns_link_context() {
        let (ctx, partial) = detect_context(r#"<a href="./doc"#).expect("detected");
        assert_eq!(ctx, AssetContext::Link);
        assert_eq!(partial, "./doc");
    }

    #[test]
    fn closing_quote_means_we_are_past_the_attribute() {
        // src="done" — we're now after the value, not inside it.
        assert!(detect_context(r#"<img src="done""#).is_none());
    }

    #[test]
    fn video_src_is_treated_as_image_context() {
        let (ctx, _) = detect_context(r#"<video src=""#).expect("detected");
        assert_eq!(ctx, AssetContext::Image);
    }

    #[test]
    fn audio_src_is_treated_as_image_context() {
        let (ctx, _) = detect_context(r#"<audio src=""#).expect("detected");
        assert_eq!(ctx, AssetContext::Image);
    }

    #[test]
    fn unknown_tag_src_falls_back_to_link_context() {
        // We don't want to over-filter when the user is using a custom
        // element that happens to have a `src` attribute.
        let (ctx, _) = detect_context(r#"<my-thing src=""#).expect("detected");
        assert_eq!(ctx, AssetContext::Link);
    }
}

mod completion_items {
    use super::*;

    fn labels(items: &[CompletionItem]) -> Vec<&str> {
        items.iter().map(|i| i.label.as_str()).collect()
    }

    #[test]
    fn lists_files_and_directories_in_the_document_directory() {
        let dir = tmp_dir("listdir");
        fs::write(dir.join("alpha.md"), "").unwrap();
        fs::write(dir.join("beta.png"), "").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();

        let items = completion_items(AssetContext::Link, "", Some(&dir), None);
        let names = labels(&items);
        assert!(names.contains(&"alpha.md"), "{names:?}");
        assert!(names.contains(&"beta.png"), "{names:?}");
        assert!(names.contains(&"subdir/"), "{names:?}");
    }

    #[test]
    fn image_context_filters_to_media_extensions() {
        let dir = tmp_dir("image-filter");
        fs::write(dir.join("photo.png"), "").unwrap();
        fs::write(dir.join("notes.md"), "").unwrap();
        fs::write(dir.join("clip.mp4"), "").unwrap();
        fs::write(dir.join("data.json"), "").unwrap();

        let items = completion_items(AssetContext::Image, "", Some(&dir), None);
        let names = labels(&items);
        assert!(names.contains(&"photo.png"), "{names:?}");
        assert!(names.contains(&"clip.mp4"), "{names:?}");
        assert!(!names.contains(&"notes.md"), "{names:?}");
        assert!(!names.contains(&"data.json"), "{names:?}");
    }

    #[test]
    fn prefix_filters_entries() {
        let dir = tmp_dir("prefix");
        fs::write(dir.join("alpha.md"), "").unwrap();
        fs::write(dir.join("apple.md"), "").unwrap();
        fs::write(dir.join("beta.md"), "").unwrap();

        let items = completion_items(AssetContext::Link, "a", Some(&dir), None);
        let names = labels(&items);
        assert_eq!(names, vec!["alpha.md", "apple.md"]);
    }

    #[test]
    fn nested_path_lists_the_named_subdirectory() {
        let dir = tmp_dir("nested");
        fs::create_dir(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/hero.png"), "").unwrap();
        fs::write(dir.join("assets/footer.png"), "").unwrap();

        let items = completion_items(AssetContext::Image, "assets/", Some(&dir), None);
        let names = labels(&items);
        assert!(names.contains(&"hero.png"), "{names:?}");
        assert!(names.contains(&"footer.png"), "{names:?}");
    }

    #[test]
    fn absolute_path_resolves_under_src_dir() {
        let dir = tmp_dir("abs-src");
        fs::create_dir(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/hero.png"), "").unwrap();

        let doc_dir = dir.join("docs");
        fs::create_dir_all(&doc_dir).unwrap();
        let items = completion_items(AssetContext::Image, "/assets/", Some(&doc_dir), Some(&dir));
        let names = labels(&items);
        assert!(names.contains(&"hero.png"), "{names:?}");
    }

    #[test]
    fn hidden_entries_are_skipped() {
        let dir = tmp_dir("hidden");
        fs::write(dir.join(".env"), "").unwrap();
        fs::write(dir.join("visible.md"), "").unwrap();

        let items = completion_items(AssetContext::Link, "", Some(&dir), None);
        let names = labels(&items);
        assert!(!names.iter().any(|n| n.starts_with('.')), "{names:?}");
        assert!(names.contains(&"visible.md"), "{names:?}");
    }

    #[test]
    fn nonexistent_directory_returns_empty() {
        let dir = tmp_dir("missing-base");
        let items = completion_items(AssetContext::Link, "no-such-dir/", Some(&dir), None);
        assert!(items.is_empty());
    }
}

mod line_prefix {
    use super::*;

    #[test]
    fn returns_slice_up_to_utf16_column() {
        let line = "abc def";
        assert_eq!(line_prefix(line, 0), "");
        assert_eq!(line_prefix(line, 3), "abc");
        assert_eq!(line_prefix(line, 7), "abc def");
    }

    #[test]
    fn handles_supplementary_characters_via_utf16_units() {
        // 𝄞 is U+1D11E, a supplementary character: 2 UTF-16 code units.
        let line = "a𝄞b";
        // After "a", the cursor is at character 1.
        assert_eq!(line_prefix(line, 1), "a");
        // After "a𝄞", the cursor is at character 3 (1 + 2 surrogates).
        assert_eq!(line_prefix(line, 3), "a𝄞");
    }
}
