//! Which documents `oxct lint` checks, and how it names them.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

const DIRTY: &str = "with with\n";

/// Every fixture document repeats one word, so the reported files are exactly the checked ones.
fn project(files: &[&str]) -> Project {
    let project = Project::new();
    for file in files {
        project.write(file, DIRTY);
    }
    project
}

fn tree() -> Project {
    project(&[
        "a.md",
        "b.markdown",
        "docs/c.mdx",
        "docs/deep/d.mdc",
        "docs/e.txt",
        ".hidden/h.md",
        "node_modules/x.md",
        "docs/node_modules/x.md",
        "dist/x.md",
        "target/x.md",
        ".git/x.md",
    ])
}

#[track_caller]
fn checked(project: &Project, args: &[&str]) -> Vec<String> {
    let run = project.run(&[&["lint", "--no-markdownlint", "--format", "json"], args].concat());
    let report = run.json();
    let files: Vec<_> = run.diagnostics().into_iter().map(|(file, ..)| file).collect();
    assert_eq!(report["checkedFileCount"], files.len(), "{}", run.stdout);
    files
}

#[test]
fn walks_every_markdown_extension_and_skips_dependency_and_build_directories() {
    assert_eq!(
        checked(&tree(), &[]),
        [".hidden/h.md", "a.md", "b.markdown", "docs/c.mdx", "docs/deep/d.mdc"]
    );
}

#[test]
fn default_input_is_the_working_directory() {
    let project = tree();
    assert_eq!(checked(&project, &[]), checked(&project, &["."]));
}

#[test]
fn explicit_files_skip_the_extension_filter_but_never_the_directory_denylist() {
    let project = tree();
    assert_eq!(checked(&project, &["docs/e.txt"]), ["docs/e.txt"]);
    for denied in ["node_modules/x.md", "docs/node_modules/x.md", "dist/x.md", ".git/x.md"] {
        project.run(&["lint", denied]).rejected("No Markdown files matched");
    }
}

#[test]
fn files_directories_and_globs_combine_without_duplicates() {
    let files =
        checked(&tree(), &["a.md", "./a.md", "docs/../a.md", "docs", "docs/**/*.mdc", "a.md"]);
    assert_eq!(files, ["a.md", "docs/c.mdx", "docs/deep/d.mdc"]);
}

#[test]
fn report_order_follows_file_names_not_argument_order() {
    let project = tree();
    assert_eq!(
        checked(&project, &["docs/c.mdx", "b.markdown", "a.md"]),
        ["a.md", "b.markdown", "docs/c.mdx"]
    );
}

#[test]
fn glob_syntax_matches_like_a_shell() {
    let project = tree();
    for (pattern, expected) in [
        ("*.md", &["a.md"][..]),
        ("*.{md,markdown}", &["a.md", "b.markdown"]),
        ("?.md", &["a.md"]),
        ("[ab].*", &["a.md", "b.markdown"]),
        ("[!a].markdown", &["b.markdown"]),
        ("docs/*", &["docs/c.mdx", "docs/e.txt"]),
        ("docs/**/*.{mdx,mdc}", &["docs/c.mdx", "docs/deep/d.mdc"]),
        ("**/deep/*", &["docs/deep/d.mdc"]),
        ("**/*.mdc", &["docs/deep/d.mdc"]),
        ("./docs/../*.md", &["a.md"]),
    ] {
        assert_eq!(checked(&project, &[pattern]), expected, "{pattern}");
    }
}

#[test]
fn a_single_star_never_crosses_directories() {
    let project = tree();
    assert_eq!(checked(&project, &["*/*.mdx"]), ["docs/c.mdx"]);
    project.run(&["lint", "*.mdc"]).rejected("No Markdown files matched");
}

#[test]
fn ignore_patterns_are_relative_to_the_working_directory() {
    let project = tree();
    let all = [".hidden/h.md", "a.md", "b.markdown", "docs/c.mdx", "docs/deep/d.mdc"];
    for (ignore, removed) in [
        (&["docs"][..], &["docs/c.mdx", "docs/deep/d.mdc"][..]),
        (&["docs/**"], &["docs/c.mdx", "docs/deep/d.mdc"]),
        (&["docs/deep"], &["docs/deep/d.mdc"]),
        (&["**/deep/**"], &["docs/deep/d.mdc"]),
        (&["*.md"], &["a.md"]),
        (&["**/*.md"], &[".hidden/h.md", "a.md"]),
        (&["a.md", "b.markdown"], &["a.md", "b.markdown"]),
        (&[".hidden"], &[".hidden/h.md"]),
        (&["missing/**"], &[]),
    ] {
        let args: Vec<_> = ignore.iter().flat_map(|pattern| ["--ignore", pattern]).collect();
        let expected: Vec<_> = all.iter().filter(|file| !removed.contains(file)).collect();
        assert_eq!(checked(&project, &args).iter().collect::<Vec<_>>(), expected, "{ignore:?}");
    }
}

#[test]
fn ignoring_every_input_is_an_error_instead_of_a_silent_pass() {
    let project = tree();
    for args in [
        &["lint", "a.md", "--ignore", "a.md"][..],
        &["lint", "docs", "--ignore", "docs/**"],
        &["lint", "--ignore", "**"],
    ] {
        project.run(args).rejected("No Markdown files matched");
    }
}

#[test]
fn markdownlintignore_uses_gitignore_semantics_for_walked_and_explicit_files() {
    let project = tree();
    project.write(".markdownlintignore", "# generated\n*.md\n!a.md\ndocs/\n");
    assert_eq!(checked(&project, &[]), ["a.md", "b.markdown"]);
    assert_eq!(
        checked(&project, &["a.md", "b.markdown", ".hidden/h.md", "docs"]),
        ["a.md", "b.markdown"]
    );
    project.run(&["lint", "docs/c.mdx"]).rejected("No Markdown files matched");
}

#[test]
fn configuration_supplies_default_inputs_and_extra_ignores() {
    let project = tree();
    project.write("lint.json", r#"{"include":["docs","*.markdown"],"ignore":["docs/deep"]}"#);
    let config = ["--config", "lint.json"];
    assert_eq!(checked(&project, &config), ["b.markdown", "docs/c.mdx"]);
    // Command-line inputs replace `include`; ignores from both sources apply.
    assert_eq!(
        checked(&project, &[&config[..], &["a.md", "docs"]].concat()),
        ["a.md", "docs/c.mdx"]
    );
    assert_eq!(
        checked(&project, &[&config[..], &["--ignore", "*.markdown"]].concat()),
        ["docs/c.mdx"]
    );
}

#[test]
fn labels_are_relative_to_the_working_directory_even_above_it() {
    let project = tree();
    let run = project.run_in("docs", &["lint", "--no-markdownlint", "--format", "json", ".."]);
    let files: Vec<_> = run.diagnostics().into_iter().map(|(file, ..)| file).collect();
    assert_eq!(files, ["../.hidden/h.md", "../a.md", "../b.markdown", "c.mdx", "deep/d.mdc"]);
}

#[test]
fn unicode_and_spaced_names_survive_discovery_and_reporting() {
    let project = project(&["日本語/ガイド.md", "sp ace.md", "émoji 👩‍💻.mdx"]);
    assert_eq!(checked(&project, &[]), ["sp ace.md", "émoji 👩‍💻.mdx", "日本語/ガイド.md"]);
    assert_eq!(checked(&project, &["日本語"]), ["日本語/ガイド.md"]);
    let text = project.run(&["lint", "--no-markdownlint", "sp ace.md"]);
    assert!(text.stdout.contains("sp ace.md:1:6 warning"), "{}", text.stdout);
}

#[test]
fn malformed_patterns_are_reported_instead_of_matching_nothing() {
    let project = tree();
    project.run(&["lint", "docs/["]).rejected("error parsing glob");
    project.run(&["lint", "--ignore", "["]).rejected("error parsing glob");
    project.run(&["lint", "{a,b"]).rejected("error parsing glob");
}

#[cfg(unix)]
#[test]
fn symlinked_files_are_checked_but_symlinked_directories_are_only_followed_when_named() {
    let project = tree();
    std::os::unix::fs::symlink("a.md", project.file("link.md")).unwrap();
    std::os::unix::fs::symlink("docs", project.file("linked")).unwrap();
    std::os::unix::fs::symlink("missing.md", project.file("dangling.md")).unwrap();
    assert_eq!(checked(&project, &["*.md"]), ["a.md", "link.md"]);
    assert!(!checked(&project, &[]).iter().any(|file| file.starts_with("linked/")));
    assert_eq!(checked(&project, &["linked"]), ["linked/c.mdx", "linked/deep/d.mdc"]);
    project.run(&["lint", "dangling.md"]).rejected("No Markdown files matched");
}

/// Workers receive 128 documents at a time; diagnostics must keep their own file across batches.
#[test]
fn diagnostics_stay_attached_to_their_file_across_worker_batches() {
    let project = Project::new();
    let dirty = [0, 127, 128, 129, 255, 256, 299];
    for index in 0..300 {
        let content = if dirty.contains(&index) { DIRTY } else { "Clean prose.\n" };
        project.write(&format!("{index:03}.md"), content);
    }
    let expected: Vec<_> = dirty.iter().map(|index| format!("{index:03}.md")).collect();
    for threads in ["1", "2", "7"] {
        let run =
            project.run(&["lint", "--no-markdownlint", "--format", "json", "--threads", threads]);
        let report = run.json();
        assert_eq!(report["checkedFileCount"], 300);
        assert_eq!(report["warningCount"], dirty.len());
        let files: Vec<_> = run.diagnostics().into_iter().map(|(file, ..)| file).collect();
        assert_eq!(files, expected, "--threads {threads}");
    }
}
