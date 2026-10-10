use super::*;
use std::io::Cursor;

#[test]
fn markdown_glob_covers_every_supported_extension() {
    assert_eq!(MARKDOWN_GLOB, "**/*.{md,markdown,mdx,mdc}");
    let matcher = glob(&[MARKDOWN_GLOB.to_string()]).unwrap();
    for path in ["a.md", "docs/b.markdown", "docs/deep/c.mdx", ".hidden/d.mdc", "日本語/ガイド.md"]
    {
        assert!(matcher.is_match(path), "{path}");
    }
    for path in ["a.txt", "a.md.bak", "md", "docs/readme", "a.MD", "a.mdxx"] {
        assert!(!matcher.is_match(path), "{path}");
    }
}

#[test]
fn glob_patterns_do_not_let_a_star_cross_directories() {
    let matcher = glob(&["docs/*.md".to_string(), "**/keep/**".to_string()]).unwrap();
    assert!(matcher.is_match("docs/a.md") && matcher.is_match("x/keep/y/z.md"));
    assert!(!matcher.is_match("docs/deep/a.md"));
    assert!(glob(&["[".to_string()]).is_err());
    assert!(!glob(&[]).unwrap().is_match("anything"));
}

#[test]
fn slash_normalises_separators_for_reports() {
    assert_eq!(slash(Path::new("docs\\guide\\a.md")), "docs/guide/a.md");
    assert_eq!(slash(Path::new("docs/guide/a.md")), "docs/guide/a.md");
    assert_eq!(slash(Path::new("")), "");
}

#[cfg(unix)]
#[test]
fn absolute_resolves_dots_lexically_without_touching_the_disk() {
    for (input, expected) in [
        ("/a/b/c.md", "/a/b/c.md"),
        ("/a/./b/../c.md", "/a/c.md"),
        ("/a/b/../../c.md", "/c.md"),
        ("/../../a.md", "/a.md"),
        ("/a/b/", "/a/b"),
        ("/does/not/exist/../x.md", "/does/not/x.md"),
        ("/", "/"),
    ] {
        assert_eq!(absolute(Path::new(input)).unwrap(), PathBuf::from(expected), "{input}");
    }
}

#[test]
fn absolute_anchors_relative_paths_at_the_working_directory() {
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(absolute(Path::new("docs/../a.md")).unwrap(), cwd.join("a.md"));
    assert_eq!(absolute(Path::new("./a.md")).unwrap(), cwd.join("a.md"));
    assert_eq!(absolute(Path::new(".")).unwrap(), cwd);
    assert_eq!(absolute(Path::new("..")).unwrap(), cwd.parent().unwrap());
    assert!(absolute(Path::new("")).is_err());
}

#[cfg(unix)]
#[test]
fn relative_walks_up_from_the_base_to_the_path() {
    for (path, base, expected) in [
        ("/a/b/c.md", "/a/b", "c.md"),
        ("/a/b/c/d.md", "/a/b", "c/d.md"),
        ("/a/b/c.md", "/a/x/y", "../../b/c.md"),
        ("/a/c.md", "/a/b", "../c.md"),
        ("/a/b", "/a/b", ""),
        ("/a", "/a/b/c", "../.."),
        ("/x.md", "/a/b", "../../x.md"),
        // Without a shared root there is nothing to be relative to.
        ("rel/x.md", "/a/b", "rel/x.md"),
        ("/a/x.md", "other", "/a/x.md"),
    ] {
        assert_eq!(
            relative(Path::new(path), Path::new(base)),
            PathBuf::from(expected),
            "{path} from {base}"
        );
    }
}

#[cfg(unix)]
#[test]
fn relative_round_trips_through_absolute() {
    for (path, base) in [("/a/b/c.md", "/a/x/y"), ("/a/b/c.md", "/a/b"), ("/x.md", "/a/b/c/d")] {
        let joined = Path::new(base).join(relative(Path::new(path), Path::new(base)));
        assert_eq!(absolute(&joined).unwrap(), PathBuf::from(path), "{path} from {base}");
    }
}

#[test]
fn bounded_reads_accept_documents_up_to_the_limit() {
    assert_eq!(read_bounded(Cursor::new(Vec::new())).unwrap(), "");
    assert_eq!(read_bounded(Cursor::new("# 日本語\n".as_bytes())).unwrap(), "# 日本語\n");
    let limit = usize::try_from(VIEWER_LIMIT).unwrap();
    assert_eq!(read_bounded(Cursor::new(vec![b'a'; limit])).unwrap().len(), limit);
    for size in [limit + 1, limit * 2] {
        let error = read_bounded(Cursor::new(vec![b'a'; size])).unwrap_err();
        assert_eq!(error.to_string(), "Document exceeds the 4 MiB viewer limit");
    }
}

#[test]
fn bounded_reads_never_pull_more_than_one_byte_past_the_limit() {
    struct Endless(u64);
    impl Read for Endless {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            buffer.fill(b'a');
            self.0 += buffer.len() as u64;
            Ok(buffer.len())
        }
    }
    let mut endless = Endless(0);
    assert!(read_bounded(&mut endless).is_err());
    assert_eq!(endless.0, VIEWER_LIMIT + 1);
}

#[test]
fn bounded_reads_reject_text_that_is_not_utf8() {
    for bytes in [&[0xff, 0xfe][..], b"ok \xc3", b"\xed\xa0\x80"] {
        assert!(
            read_bounded(Cursor::new(bytes)).unwrap_err().to_string().contains("utf-8"),
            "{bytes:?}"
        );
    }
}

#[test]
fn documents_are_size_checked_before_they_are_read() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("doc.md");
    std::fs::write(&file, "# Title\n").unwrap();
    assert_eq!(read_document(&file).unwrap(), "# Title\n");
    // Sparse files reach the limit without writing megabytes.
    let handle = std::fs::OpenOptions::new().write(true).open(&file).unwrap();
    handle.set_len(VIEWER_LIMIT).unwrap();
    assert_eq!(read_document(&file).unwrap().len() as u64, VIEWER_LIMIT);
    handle.set_len(VIEWER_LIMIT + 1).unwrap();
    assert_eq!(
        read_document(&file).unwrap_err().to_string(),
        "Document exceeds the 4 MiB viewer limit"
    );
    assert!(read_document(&directory.path().join("missing.md")).is_err());
    assert!(read_document(directory.path()).is_err(), "directories are not documents");
}
