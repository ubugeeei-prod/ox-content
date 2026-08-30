use super::*;

/// Runs `git` in `root`, failing the test if it does not succeed. Output is
/// captured rather than inherited so a passing run leaves the log clean.
fn git(root: &std::path::Path, args: &[&str], timestamp: &str) {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "commit.gpgsign=false", "-c", "user.name=Test"])
        .args(["-c", "user.email=test@example.com"])
        .args(args)
        .env("GIT_AUTHOR_DATE", timestamp)
        .env("GIT_COMMITTER_DATE", timestamp)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?} failed: {output:?}");
}

/// A repository whose files are committed at known, distinct times.
fn repository(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("ox-content-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/guide")).unwrap();
    git(&root, &["init"], "@1000000000");

    fs::write(root.join("docs/first.md"), "# First\n").unwrap();
    fs::write(root.join("docs/guide/nested.md"), "# Nested\n").unwrap();
    git(&root, &["add", "."], "@1000000000");
    git(&root, &["commit", "-m", "first"], "@1000000000");

    // A second commit touches only one of them, so "newest wins" is testable
    // and the two files must not report the same time.
    fs::write(root.join("docs/first.md"), "# First, edited\n").unwrap();
    fs::write(root.join("docs/second.md"), "# Second\n").unwrap();
    git(&root, &["add", "."], "@2000000000");
    git(&root, &["commit", "-m", "second"], "@2000000000");

    root
}

fn last_updated(file: &std::path::Path, root: &std::path::Path) -> Option<f64> {
    get_git_last_updated(
        file.to_string_lossy().into_owned(),
        Some(root.to_string_lossy().into_owned()),
    )
}

/// What a per-file `git log -1` would have answered, which is what this
/// lookup did before it was batched into one walk.
fn per_file_log(root: &std::path::Path, relative: &str) -> Option<f64> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%ct", "--", relative])
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    let trimmed = text.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.parse::<f64>().unwrap() * 1000.0) }
}

#[test]
fn lastmod_matches_a_per_file_log() {
    // The batched walk has to answer exactly what the per-file command did,
    // for every file, including the one git has never seen.
    let root = repository("lastmod-diff");
    fs::write(root.join("docs/untracked.md"), "# Untracked\n").unwrap();

    for relative in ["docs/first.md", "docs/second.md", "docs/guide/nested.md", "docs/untracked.md"]
    {
        assert_eq!(
            last_updated(&root.join(relative), &root),
            per_file_log(&root, relative),
            "for {relative}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lastmod_takes_the_newest_commit() {
    let root = repository("lastmod-newest");

    assert_eq!(last_updated(&root.join("docs/first.md"), &root), Some(2_000_000_000_000.0));
    assert_eq!(last_updated(&root.join("docs/guide/nested.md"), &root), Some(1_000_000_000_000.0));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lastmod_is_none_for_a_file_git_has_never_seen() {
    // This is the expensive case the batching exists for: answering "never"
    // used to mean walking the whole history, once per page.
    let root = repository("lastmod-untracked");
    fs::write(root.join("docs/untracked.md"), "# Untracked\n").unwrap();

    assert_eq!(last_updated(&root.join("docs/untracked.md"), &root), None);
    assert_eq!(last_updated(&root.join("docs/missing.md"), &root), None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lastmod_resolves_when_the_root_is_a_subdirectory() {
    // The walk reports paths relative to the directory it runs in, and the
    // lookup key is relative to `root`. When `root` is not the repository top
    // level those two only agree because the walk asks for relative paths.
    let root = repository("lastmod-subdir");
    let docs = root.join("docs");

    assert_eq!(last_updated(&docs.join("first.md"), &docs), Some(2_000_000_000_000.0));
    assert_eq!(last_updated(&docs.join("guide/nested.md"), &docs), Some(1_000_000_000_000.0));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lastmod_handles_paths_that_need_quoting() {
    // Non-ASCII and spaces are why the walk is NUL-framed rather than
    // line-framed: git quotes such paths in line-oriented output.
    let root = repository("lastmod-quoting");
    fs::write(root.join("docs/日本語 ページ.md"), "# Page\n").unwrap();
    git(&root, &["add", "."], "@3000000000");
    git(&root, &["commit", "-m", "unicode"], "@3000000000");

    assert_eq!(last_updated(&root.join("docs/日本語 ページ.md"), &root), Some(3_000_000_000_000.0));
    // The other files keep their own times.
    assert_eq!(last_updated(&root.join("docs/guide/nested.md"), &root), Some(1_000_000_000_000.0));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn lastmod_without_a_repository_is_none() {
    let root =
        std::env::temp_dir().join(format!("ox-content-lastmod-nogit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("page.md"), "# Page\n").unwrap();

    assert_eq!(last_updated(&root.join("page.md"), &root), None);
    assert_eq!(
        get_git_last_updated(root.join("page.md").to_string_lossy().into_owned(), None),
        None
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn looking_up_every_page_costs_one_walk() {
    // The lookup used to run `git log` per file, so a build paid one process
    // spawn and one history walk per page. Now the first lookup builds the
    // whole map and the rest read it, which makes 500 pages cheaper than the
    // one page that filled the cache — a ratio no per-file implementation can
    // reach, and one that does not depend on how fast the machine is.
    let root = repository("lastmod-batched");
    for index in 0..500 {
        fs::write(root.join(format!("docs/page-{index}.md")), "# Page\n").unwrap();
    }

    let started = std::time::Instant::now();
    last_updated(&root.join("docs/first.md"), &root);
    let first = started.elapsed();

    let started = std::time::Instant::now();
    for index in 0..500 {
        last_updated(&root.join(format!("docs/page-{index}.md")), &root);
    }
    let rest = started.elapsed();

    assert!(
        rest < first,
        "500 lookups took {rest:?} against {first:?} for the one that built the map; \
         a per-file lookup would take about 500 times as long"
    );

    let _ = fs::remove_dir_all(root);
}
