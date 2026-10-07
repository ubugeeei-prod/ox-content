//! `oxct ide install` plans, merges and applies editor configuration without losing user edits.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;
use serde_json::{Value, json};

const VSCODE_SETTINGS: &str = "{\n  \"files.associations\": {\n    \"*.mdc\": \"markdown\"\n  },\n  \"oxContent.frontmatter.projectValidation\": true\n}\n";
const VSCODE_EXTENSIONS: &str = "{\n  \"recommendations\": [\"ubugeeei.vscode-ox-content\"]\n}\n";
const ZED_SETTINGS: &str = "{\n  \"file_types\": {\n    \"Markdown\": [\"md\", \"markdown\", \"mdc\", \"mdx\"]\n  },\n  \"lsp\": {\n    \"ox-content-lsp\": {\n      \"binary\": {\n        \"path\": \"vpx\",\n        \"arguments\": [\"oxct\", \"lsp\", \"--project\"]\n      }\n    }\n  }\n}\n";
const NEOVIM_SETUP: &str = include_str!("../src/ide/neovim.lua");
const TRUST_NOTICE: &str = "Enable frontmatter validation from trusted project Vite configuration.";

fn install<'a>(ides: &[&'a str], flags: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["ide", "install"];
    args.extend(ides.iter().flat_map(|ide| ["--ide", ide]));
    args.extend(flags);
    args.push("--yes");
    args
}

fn jsonc(project: &Project, path: &str) -> Value {
    jsonc_parser::parse_to_serde_value(&project.read(path), &jsonc_parser::ParseOptions::default())
        .unwrap()
}

/// Backups written next to a merged configuration file.
fn backups(project: &Project, directory: &str) -> Vec<String> {
    project.list(directory).into_iter().filter(|name| name.ends_with(".bak")).collect()
}

#[test]
fn the_only_subcommand_is_install_and_it_needs_an_editor() {
    let project = Project::new();
    for args in [&["ide"][..], &["ide", "--help"], &["ide", "uninstall"], &["ide", "--ide", "zed"]]
    {
        project.run(args).rejected("Use oxct ide install");
    }
    project.run(&["ide", "install", "--yes"]).rejected("pass --ide <name>");
    project.run(&["ide", "install"]).rejected("pass --ide <name>");
    assert!(project.list(".").is_empty());
}

#[test]
fn invalid_arguments_are_rejected_before_anything_is_written() {
    let project = Project::new();
    for (args, reason) in [
        (&["--ide", "emacs"][..], "invalid value 'emacs' for '--ide <IDE>'"),
        (&["--ide", "VSCode"], "invalid value 'VSCode'"),
        (&["--ide"], "a value is required"),
        (&["--ide", "zed", "--config-only", "--extensions-only"], "cannot be used with"),
        (&["--ide", "zed", "--force"], "unexpected argument '--force'"),
        (&["zed"], "unexpected argument 'zed'"),
    ] {
        project.run(&[&["ide", "install"], args, &["--yes"]].concat()).rejected(reason);
    }
    assert!(
        project.list(".").is_empty() && std::fs::read_dir(project.home()).unwrap().next().is_none()
    );
}

#[cfg(unix)]
#[test]
fn dry_run_prints_the_whole_plan_and_touches_nothing() {
    let project = Project::new();
    let all = ["vscode", "cursor", "windsurf", "vscodium", "zed", "neovim"];
    let run = project.run(&install(&all, &["--dry-run"]));
    run.success();
    let work = project.work();
    let home = project.home();
    let expected = [
        TRUST_NOTICE.to_string(),
        format!("Create: {}", work.join(".vscode/settings.json").display()),
        format!("Create: {}", work.join(".vscode/extensions.json").display()),
        format!("Create: {}", work.join(".zed/settings.json").display()),
        format!("Create: {}", home.join(".config/zed/settings.json").display()),
        format!("Create: {}", work.join(".ox-content/neovim.lua").display()),
        "code --install-extension ubugeeei.vscode-ox-content".to_string(),
        "cursor --install-extension ubugeeei.vscode-ox-content".to_string(),
        "windsurf --install-extension ubugeeei.vscode-ox-content".to_string(),
        "codium --install-extension ubugeeei.vscode-ox-content".to_string(),
        format!(
            "Install bundled Neovim plugin: {}",
            home.join(".local/share/nvim/site/pack/ox-content/start/ox-content").display()
        ),
    ];
    assert_eq!(run.stdout.lines().collect::<Vec<_>>(), expected);
    assert!(project.list(".").is_empty());
    assert!(std::fs::read_dir(home).unwrap().next().is_none());
}

#[test]
fn every_vs_code_family_editor_shares_the_workspace_configuration() {
    for ide in ["vscode", "cursor", "windsurf", "vscodium"] {
        let project = Project::new();
        let run = project.run(&install(&[ide], &["--config-only"]));
        run.success();
        assert_eq!(project.list("."), [".vscode"], "{ide}");
        assert_eq!(project.list(".vscode"), ["extensions.json", "settings.json"], "{ide}");
        assert_eq!(project.read(".vscode/settings.json"), VSCODE_SETTINGS, "{ide}");
        assert_eq!(project.read(".vscode/extensions.json"), VSCODE_EXTENSIONS, "{ide}");
        assert!(
            run.stdout.starts_with(TRUST_NOTICE) && run.stdout.ends_with("◆ IDE setup applied\n")
        );
        assert!(!run.stdout.contains("--install-extension"), "{}", run.stdout);
    }
}

#[test]
fn zed_and_neovim_get_their_own_workspace_files() {
    let project = Project::new();
    let run = project.run(&install(&["zed", "neovim"], &["--config-only"]));
    run.success();
    assert_eq!(project.list("."), [".ox-content", ".zed"]);
    assert_eq!(project.read(".zed/settings.json"), ZED_SETTINGS);
    assert_eq!(project.read(".ox-content/neovim.lua"), NEOVIM_SETUP);
    assert!(run.stdout.ends_with("Neovim: :luafile .ox-content/neovim.lua (Neovim 0.11+).\n"));
    // The Zed restart hint only matters when its extension is installed.
    assert!(!run.stdout.contains("Zed: restart"), "{}", run.stdout);
}

#[test]
fn repeated_setup_keeps_every_file_and_writes_no_backups() {
    let project = Project::new();
    let args = install(&["vscode", "zed", "neovim"], &["--config-only"]);
    project.run(&args).success();
    let files = [
        ".vscode/settings.json",
        ".vscode/extensions.json",
        ".zed/settings.json",
        ".ox-content/neovim.lua",
    ];
    let before: Vec<_> = files.iter().map(|file| project.read(file)).collect();
    let run = project.run(&args);
    run.success();
    for file in files {
        assert!(run.stdout.contains(&format!("Keep: {}", project.file(file).display())), "{file}");
    }
    assert!(!run.stdout.contains("Create:") && !run.stdout.contains("Merge"), "{}", run.stdout);
    assert_eq!(files.iter().map(|file| project.read(file)).collect::<Vec<_>>(), before);
    for directory in [".vscode", ".zed", ".ox-content"] {
        assert!(backups(&project, directory).is_empty(), "{directory}");
    }
}

#[test]
fn existing_settings_are_merged_and_the_original_is_backed_up() {
    let project = Project::new();
    let original = "{\n  // Team defaults\n  \"editor.tabSize\": 4,\n  \"files.associations\": { \"*.foo\": \"text\", \"*.mdc\": \"mdc\" },\n  \"oxContent.frontmatter.projectValidation\": false, /* off */\n}\n";
    project.write(".vscode/settings.json", original);
    let run = project.run(&install(&["vscode"], &["--config-only"]));
    run.success();
    let file = project.file(".vscode/settings.json");
    assert!(
        run.stdout.contains(&format!("Merge and back up: {}", file.display())),
        "{}",
        run.stdout
    );
    let content = project.read(".vscode/settings.json");
    assert!(content.contains("// Team defaults") && content.contains("/* off */"), "{content}");
    assert_eq!(
        jsonc(&project, ".vscode/settings.json"),
        json!({
            "editor.tabSize": 4,
            "files.associations": { "*.foo": "text", "*.mdc": "markdown" },
            "oxContent.frontmatter.projectValidation": true
        })
    );
    let backups = backups(&project, ".vscode");
    assert_eq!(backups.len(), 1, "{backups:?}");
    assert!(backups[0].starts_with("settings.json.oxct-"), "{backups:?}");
    assert_eq!(project.read(&format!(".vscode/{}", backups[0])), original);
}

#[test]
fn recommendations_and_file_types_are_appended_without_duplicates() {
    let project = Project::new();
    project.write(
        ".vscode/extensions.json",
        r#"{"recommendations":["esbenp.prettier-vscode"],"unwantedRecommendations":["x.y"]}"#,
    );
    project.write(
        ".zed/settings.json",
        r#"{"file_types":{"Markdown":["mdx","txt"],"JSON":["jsonc"]},"lsp":{"rust-analyzer":{"binary":{"path":"ra"}}},"tab_size":2}"#,
    );
    project.run(&install(&["vscode", "zed"], &["--config-only"])).success();
    assert_eq!(
        jsonc(&project, ".vscode/extensions.json"),
        json!({
            "recommendations": ["esbenp.prettier-vscode", "ubugeeei.vscode-ox-content"],
            "unwantedRecommendations": ["x.y"]
        })
    );
    assert_eq!(
        jsonc(&project, ".zed/settings.json"),
        json!({
            "file_types": { "Markdown": ["mdx", "txt", "md", "markdown", "mdc"], "JSON": ["jsonc"] },
            "lsp": {
                "rust-analyzer": { "binary": { "path": "ra" } },
                "ox-content-lsp": { "binary": { "path": "vpx", "arguments": ["oxct", "lsp", "--project"] } }
            },
            "tab_size": 2
        })
    );
    // A second run finds nothing left to merge.
    let second = project.run(&install(&["vscode", "zed"], &["--config-only"]));
    assert_eq!(second.stdout.matches("Keep: ").count(), 3, "{}", second.stdout);
    assert_eq!(backups(&project, ".vscode").len() + backups(&project, ".zed").len(), 2);
}

#[test]
fn unusable_existing_configuration_stops_setup_before_any_write() {
    for (file, content, ides, reason) in [
        (".vscode/settings.json", "{\"broken\":", &["vscode"][..], "Invalid configuration"),
        (".vscode/settings.json", "[]", &["vscode"], "Invalid configuration"),
        (".vscode/settings.json", "null", &["cursor"], "Invalid configuration"),
        (
            ".vscode/extensions.json",
            r#"{"recommendations":"x"}"#,
            &["vscode"],
            "expected an array of strings",
        ),
        (
            ".vscode/extensions.json",
            r#"{"recommendations":[1]}"#,
            &["windsurf"],
            "expected an array of strings",
        ),
        (
            ".zed/settings.json",
            r#"{"file_types":{"Markdown":"md"}}"#,
            &["vscode", "zed"],
            "expected an array of strings",
        ),
        (".ox-content/neovim.lua", "-- mine\n", &["zed", "neovim"], "Preserving existing"),
    ] {
        let project = Project::new();
        project.write(file, content);
        project.run(&install(ides, &["--config-only"])).rejected(reason);
        assert_eq!(project.read(file), content, "{file}");
        // Nothing else was created alongside the rejected file.
        let directory = file.split('/').next().unwrap();
        assert_eq!(project.list("."), [directory], "{file}");
        assert_eq!(project.list(directory).len(), 1, "{file}");
    }
}

#[test]
fn dry_run_describes_merges_without_writing_backups() {
    let project = Project::new();
    project.write(".vscode/settings.json", "{\"editor.tabSize\": 2}\n");
    project.write(".vscode/extensions.json", VSCODE_EXTENSIONS);
    let run = project.run(&install(&["vscode"], &["--config-only", "--dry-run"]));
    run.success();
    let lines: Vec<_> = run.stdout.lines().collect();
    assert_eq!(
        lines,
        [
            TRUST_NOTICE.to_string(),
            format!("Merge and back up: {}", project.file(".vscode/settings.json").display()),
            format!("Keep: {}", project.file(".vscode/extensions.json").display()),
        ]
    );
    assert_eq!(project.read(".vscode/settings.json"), "{\"editor.tabSize\": 2}\n");
    assert_eq!(project.list(".vscode"), ["extensions.json", "settings.json"]);
}
