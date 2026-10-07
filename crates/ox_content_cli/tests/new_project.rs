//! `oxct new` scaffolds a project atomically and explains what to do next.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;
use serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHARED: [&str; 6] =
    [".gitignore", "README.md", "index.html", "package.json", "tsconfig.json", "vite.config.ts"];

/// Every file below a generated project, relative and sorted.
fn tree(project: &Project, directory: &str) -> Vec<String> {
    fn walk(root: &std::path::Path, directory: &std::path::Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, found);
            } else {
                found.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let root = project.file(directory);
    let mut found = Vec::new();
    walk(&root, &root, &mut found);
    found.sort();
    found
}

fn package(project: &Project, directory: &str) -> Value {
    serde_json::from_str(&project.read(&format!("{directory}/package.json"))).unwrap()
}

#[test]
fn defaults_create_a_docs_site_and_print_the_next_steps() {
    let project = Project::new();
    let run = project.run(&["new", "--yes"]);
    run.success();
    let target = project.file("my-content");
    assert_eq!(
        run.stdout,
        format!("◆ Created {}\ncd 'my-content'\nvp install\nvp dev\n", target.display())
    );
    assert_eq!(run.stderr, "");
    let mut expected: Vec<_> = SHARED.iter().map(ToString::to_string).collect();
    expected.extend(["content/guide.md".to_string(), "content/index.md".to_string()]);
    expected.sort();
    assert_eq!(tree(&project, "my-content"), expected);
    let config = project.read("my-content/vite.config.ts");
    for expected in [
        r#"import { defineConfig } from "vite-plus";"#,
        r#"import skin from "@ox-content/theme-editorial";"#,
        r#"import palette from "@ox-content/theme-color-nord";"#,
        r#"siteName: "my-content","#,
        r#"srcDir: "content","#,
    ] {
        assert!(config.contains(expected), "Missing {expected}: {config}");
    }
}

#[test]
fn each_template_writes_its_own_content_and_nothing_unresolved() {
    for (template, content) in [
        ("docs", &["content/guide.md", "content/index.md"][..]),
        ("blog", &["content/index.md", "content/posts/welcome.md"]),
        ("minimal", &["content/index.md"]),
    ] {
        let project = Project::new();
        project.run(&["new", "site", "--template", template, "--yes"]).success();
        let files = tree(&project, "site");
        let mut expected: Vec<_> = SHARED.iter().chain(content).map(ToString::to_string).collect();
        expected.sort();
        assert_eq!(files, expected, "{template}");
        for file in &files {
            let text = project.text(&format!("site/{file}"));
            assert!(!text.contains("__OX_"), "{template}/{file}: {text}");
            assert!(text.ends_with('\n'), "{template}/{file}");
        }
        assert!(project.read("site/README.md").contains(&format!("An Ox Content {template} site")));
        let config = project.read("site/vite.config.ts");
        assert_eq!(config.contains("defineCollections"), template == "blog", "{template}");
        assert_eq!(config.contains(r#"link: "/guide""#), template == "docs", "{template}");
    }
}

#[test]
fn the_generated_lint_script_finds_every_generated_document() {
    for (template, documents) in [("docs", 2), ("blog", 2), ("minimal", 1)] {
        let project = Project::new();
        project.run(&["new", "site", "--template", template, "--yes"]).success();
        assert_eq!(
            package(&project, "site")["scripts"]["lint"],
            r#"oxct lint "content/**/*.{md,mdx,mdc}""#
        );
        let lint =
            project.run_in("site", &["lint", "content/**/*.{md,mdx,mdc}", "--format", "json"]);
        assert_eq!(lint.json()["checkedFileCount"], documents, "{template}");
        // Prose and structure are clean; only the markdownlint profile is not pinned here.
        let prose = project.run_in("site", &["lint", "content", "--no-markdownlint", "--strict"]);
        assert_eq!(prose.code, Some(0), "{template}: {}", prose.stdout);
    }
}

#[test]
fn package_manager_selects_the_toolchain_and_the_printed_commands() {
    for (manager, runner, bundler, range, start) in [
        ("vp", "vp", "vite-plus", "^0.3.2", "vp dev"),
        ("pnpm", "vite", "vite", "^8.0.0", "pnpm run dev"),
        ("npm", "vite", "vite", "^8.0.0", "npm run dev"),
        ("yarn", "vite", "vite", "^8.0.0", "yarn run dev"),
        ("bun", "vite", "vite", "^8.0.0", "bun run dev"),
    ] {
        let project = Project::new();
        let run = project.run(&["new", "site", "--package-manager", manager, "--yes"]);
        run.success();
        assert!(
            run.stdout.ends_with(&format!("cd 'site'\n{manager} install\n{start}\n")),
            "{}",
            run.stdout
        );
        let package = package(&project, "site");
        assert_eq!(package["scripts"]["dev"], format!("{runner} dev"));
        assert_eq!(package["scripts"]["build"], format!("{runner} build"));
        assert_eq!(package["scripts"]["preview"], format!("{runner} preview"));
        let dependencies = package["devDependencies"].as_object().unwrap();
        assert_eq!(dependencies[bundler], range, "{manager}");
        assert_eq!(dependencies.contains_key("vite"), bundler == "vite", "{manager}");
        assert_eq!(dependencies.contains_key("vite-plus"), bundler == "vite-plus", "{manager}");
        let import = format!(r#"import {{ defineConfig }} from "{bundler}";"#);
        assert!(project.read("site/vite.config.ts").starts_with(&import), "{manager}");
    }
}

#[test]
fn skin_and_palette_reach_the_dependencies_and_the_configuration() {
    for (skin, palette) in
        [("paper", "sepia"), ("liquid-glass", "high-contrast"), ("zine", "arctic")]
    {
        let project = Project::new();
        project.run(&["new", "site", "--skin", skin, "--palette", palette, "--yes"]).success();
        let package = package(&project, "site");
        let dependencies = package["devDependencies"].as_object().unwrap();
        assert_eq!(dependencies[&format!("@ox-content/theme-{skin}")], VERSION);
        assert_eq!(dependencies[&format!("@ox-content/theme-color-{palette}")], VERSION);
        assert_eq!(dependencies["@ox-content/vite-plugin"], VERSION);
        assert_eq!(dependencies.len(), 5, "{dependencies:?}");
        let config = project.read("site/vite.config.ts");
        assert!(config.contains(&format!(r#"from "@ox-content/theme-{skin}";"#)));
        assert!(config.contains(&format!(r#"from "@ox-content/theme-color-{palette}";"#)));
    }
}

#[test]
fn package_names_are_derived_from_the_directory_name() {
    for (directory, name) in [
        ("My Site", "my-site"),
        ("docs_v2.1", "docs-v2-1"),
        ("UPPER", "upper"),
        ("_edge_", "edge"),
        ("a/b/Nested Site", "nested-site"),
        ("日本語", "my-content"),
        ("café", "caf"),
        ("123", "123"),
    ] {
        let project = Project::new();
        project.run(&["new", directory, "--yes"]).success();
        let package = package(&project, directory);
        assert_eq!(package["name"], name, "{directory}");
        assert_eq!((&package["private"], &package["type"]), (&true.into(), &"module".into()));
        assert_eq!(package["version"], "0.0.0");
    }
}

/// Quotes and backslashes are only legal in file names on Unix.
#[cfg(unix)]
#[test]
fn display_names_are_escaped_for_every_file_format() {
    let project = Project::new();
    let name = r#"My Site's "Docs" \o"#;
    let run = project.run(&["new", name, "--template", "blog", "--yes"]);
    run.success();
    assert!(run.stdout.contains(r#"cd 'My Site'\''s "Docs" \o'"#), "{}", run.stdout);
    let quoted = serde_json::to_string(name).unwrap();
    let config = project.read(&format!("{name}/vite.config.ts"));
    assert!(config.contains(&format!("siteName: {quoted},")), "{config}");
    assert!(config.contains(&format!("footer: {{ copyright: {quoted} }}")), "{config}");
    let index = project.text(&format!("{name}/content/index.md"));
    assert!(index.starts_with(&format!("---\ntitle: {quoted}\n")), "{index}");
    assert!(index.contains(&format!("\n# {name}\n")), "{index}");
    assert!(project.read(&format!("{name}/README.md")).starts_with(&format!("# {name}\n")));
    assert_eq!(package(&project, name)["name"], "my-site-s--docs---o");
}

#[test]
fn unicode_project_names_are_preserved_in_content() {
    let project = Project::new();
    project.run(&["new", "日本語 👩‍💻", "--template", "minimal", "--yes"]).success();
    let index = project.text("日本語 👩‍💻/content/index.md");
    assert!(
        index.contains("title: \"日本語 👩‍💻\"\n") && index.contains("\n# 日本語 👩‍💻\n"),
        "{index}"
    );
    assert!(project.read("日本語 👩‍💻/vite.config.ts").contains("siteName: \"日本語 👩‍💻\","));
}

#[test]
fn empty_directories_are_filled_and_occupied_ones_are_left_untouched() {
    let project = Project::new();
    std::fs::create_dir(project.file("empty")).unwrap();
    project.run(&["new", "empty", "--yes"]).success();
    assert!(project.exists("empty/package.json"));
    let before = tree(&project, "empty");
    let package = project.read("empty/package.json");
    for args in [&["new", "empty", "--yes"][..], &["new", "empty", "--template", "blog", "--yes"]] {
        project.run(args).rejected("Directory is not empty");
    }
    assert_eq!((tree(&project, "empty"), project.read("empty/package.json")), (before, package));
    project.write("notes/todo.txt", "keep");
    project.run(&["new", "notes", "--yes"]).rejected("Choose a new or empty directory");
    assert_eq!(project.list("notes"), ["todo.txt"]);
    project.run(&["new", ".", "--yes"]).rejected("Directory is not empty");
    project.write("file.txt", "keep");
    assert_eq!(project.run(&["new", "file.txt", "--yes"]).code, Some(1));
    assert_eq!(project.read("file.txt"), "keep");
}

#[test]
fn invalid_choices_are_rejected_before_anything_is_written() {
    for (args, reason) in [
        (&["--template", "wiki"][..], "Invalid template: wiki. Choose docs, blog, minimal."),
        (&["--skin", "missing"], "Invalid skin: missing. Choose analog-film, atlas,"),
        (&["--palette", "missing"], "Invalid palette: missing. Choose arctic, ayu,"),
        (&["--package-manager", "deno"], "Invalid manager: deno. Choose vp, pnpm, npm, yarn, bun."),
        (&["--template", "Docs"], "Invalid template: Docs."),
        (&["--template", ""], "Invalid template: ."),
        (&["--template"], "a value is required"),
        (&["--unknown"], "unexpected argument '--unknown'"),
        (&["second", "--yes"], "unexpected argument 'second'"),
    ] {
        let project = Project::new();
        project.run(&[&["new", "site"], args, &["--yes"]].concat()).rejected(reason);
        assert!(project.list(".").is_empty(), "{args:?} wrote {:?}", project.list("."));
    }
}

#[test]
fn without_a_terminal_setup_never_waits_for_answers() {
    let project = Project::new();
    // Piped stdio is not interactive, so the defaults apply even without --yes.
    let run = project.run(&["new", "site"]);
    run.success();
    assert!(run.stdout.contains("vp install\nvp dev\n"), "{}", run.stdout);
    assert!(!run.stdout.contains("Create a project"), "{}", run.stdout);
}

#[cfg(unix)]
#[test]
fn install_runs_the_chosen_package_manager_inside_the_project() {
    let project = Project::new();
    project.tool("npm", r#"echo "$@ in $(pwd)" > "$HOME/npm.log""#);
    let run = project.run(&["new", "site", "--package-manager", "npm", "--install", "--yes"]);
    run.success();
    let log = std::fs::read_to_string(project.home().join("npm.log")).unwrap();
    assert_eq!(log, format!("install in {}\n", project.file("site").display()));
    // The install hint disappears once it has already been done.
    assert!(run.stdout.ends_with("cd 'site'\nnpm run dev\n"), "{}", run.stdout);
}

#[cfg(unix)]
#[test]
fn a_failed_install_keeps_the_project_and_says_how_to_retry() {
    for script in [Some("exit 7"), None] {
        let project = Project::new();
        if let Some(script) = script {
            project.tool("pnpm", script);
        }
        let run = project.run(&["new", "site", "--package-manager", "pnpm", "--install", "--yes"]);
        assert_eq!(run.code, Some(1));
        let target = project.file("site");
        let message = format!(
            "Dependency installation failed. Project files are saved in {}; retry pnpm install there.",
            target.display()
        );
        assert!(run.stderr.contains(&message), "{}", run.stderr);
        assert!(project.exists("site/package.json") && project.exists("site/content/index.md"));
    }
}

#[cfg(unix)]
#[test]
fn the_last_install_flag_wins() {
    for (flags, installs) in [
        (&["--install", "--no-install"][..], false),
        (&["--no-install", "--install"], true),
        (&["--no-install"], false),
        (&[], false),
    ] {
        let project = Project::new();
        project.tool("npm", r#": > "$HOME/installed""#);
        let args = [&["new", "site", "--package-manager", "npm", "--yes"], flags].concat();
        project.run(&args).success();
        assert_eq!(project.home().join("installed").exists(), installs, "{flags:?}");
    }
}
