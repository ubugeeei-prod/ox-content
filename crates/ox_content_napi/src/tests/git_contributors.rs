use super::*;

#[test]
fn git_contributors_without_git_returns_empty() {
    let root = std::env::temp_dir().join(format!("ox-content-no-git-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("page.md"), "# Page").unwrap();

    let contributors = get_git_contributors(
        root.join("page.md").to_string_lossy().into_owned(),
        Some(root.to_string_lossy().into_owned()),
    );
    assert!(contributors.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_contributors_are_unique_by_email() {
    let root = std::env::temp_dir().join(format!("ox-content-git-authors-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/page.md"), "# Page\n").unwrap();

    let run = |args: &[&str], name: &str, email: &str| {
        let status = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .env("GIT_AUTHOR_NAME", name)
            .env("GIT_AUTHOR_EMAIL", email)
            .env("GIT_COMMITTER_NAME", name)
            .env("GIT_COMMITTER_EMAIL", email)
            .status()
            .unwrap();
        assert!(status.success(), "{args:?}");
    };

    run(&["init"], "Ada", "ada@example.com");
    run(&["add", "docs/page.md"], "Ada", "ada@example.com");
    run(&["commit", "-m", "first"], "Ada", "ada@example.com");
    fs::write(root.join("docs/page.md"), "# Page\n\nAda again\n").unwrap();
    run(&["add", "docs/page.md"], "Ada", "ada@example.com");
    run(&["commit", "-m", "second"], "Ada", "ada@example.com");
    fs::write(root.join("docs/page.md"), "# Page\n\nAda again\n\nGrace\n").unwrap();
    run(&["add", "docs/page.md"], "Grace Hopper", "grace@example.com");
    run(&["commit", "-m", "third"], "Grace Hopper", "grace@example.com");

    let contributors = get_git_contributors(
        root.join("docs/page.md").to_string_lossy().into_owned(),
        Some(root.to_string_lossy().into_owned()),
    );
    assert_eq!(contributors.len(), 2);
    let ada = contributors.iter().find(|author| author.name == "Ada").expect("Ada");
    let grace = contributors.iter().find(|author| author.name == "Grace Hopper").expect("Grace");
    assert_eq!(ada.email.as_deref(), Some("ada@example.com"));
    assert_eq!(ada.commits, Some(2));
    assert_eq!(grace.email.as_deref(), Some("grace@example.com"));
    assert_eq!(grace.commits, Some(1));
    let _ = fs::remove_dir_all(root);
}
