//! `oxct ide install` extension setup: user-level configuration, bundled plugins and editor CLIs.
//!
//! Editor CLIs are stand-in shell scripts, so these tests only run on Unix.

#![cfg(unix)]

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

const VSCODE_SETTINGS: &str = "{\n  \"files.associations\": {\n    \"*.mdc\": \"markdown\"\n  },\n  \"oxContent.frontmatter.projectValidation\": true\n}\n";
const VSCODE_EXTENSIONS: &str = "{\n  \"recommendations\": [\"ubugeeei.vscode-ox-content\"]\n}\n";
const TRUST_NOTICE: &str = "Enable frontmatter validation from trusted project Vite configuration.";

fn install<'a>(ides: &[&'a str], flags: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["ide", "install"];
    args.extend(ides.iter().flat_map(|ide| ["--ide", ide]));
    args.extend(flags);
    args.push("--yes");
    args
}

#[test]
fn extension_setup_for_zed_only_edits_the_user_configuration() {
    let project = Project::new();
    let run = project.run(&install(&["zed"], &["--extensions-only"]));
    run.success();
    assert!(project.list(".").is_empty());
    let settings = project.home().join(".config/zed/settings.json");
    assert_eq!(
        std::fs::read_to_string(&settings).unwrap(),
        "{\n  \"auto_install_extensions\": {\n    \"ox-content\": true\n  }\n}\n"
    );
    assert!(!run.stdout.contains(TRUST_NOTICE), "{}", run.stdout);
    assert!(run.stdout.contains("Zed: restart to apply extension auto-install"), "{}", run.stdout);
    // Without XDG_CONFIG_HOME the configuration lives below HOME.
    let fallback = Project::new();
    let mut command = fallback.command(&install(&["zed"], &["--extensions-only"]));
    command.env_remove("XDG_CONFIG_HOME");
    oxct::Run::of(command, None).success();
    assert!(fallback.home().join(".config/zed/settings.json").is_file());
}

#[test]
fn the_bundled_neovim_plugin_is_copied_once_and_never_overwritten() {
    let project = Project::new();
    let args = install(&["neovim"], &["--extensions-only"]);
    project.run(&args).success();
    let plugin = project.home().join(".local/share/nvim/site/pack/ox-content/start/ox-content");
    let bundled = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/neovim");
    for file in [
        "plugin/ox-content.lua",
        "lua/ox-content/init.lua",
        "lua/ox-content/client.lua",
        "README.md",
    ] {
        assert_eq!(
            std::fs::read(plugin.join(file)).unwrap(),
            std::fs::read(bundled.join(file)).unwrap(),
            "{file}"
        );
    }
    assert!(project.list(".").is_empty(), "extension setup must not write workspace files");
    std::fs::write(plugin.join("plugin/ox-content.lua"), "-- locally patched\n").unwrap();
    let second = project.run(&args);
    assert_eq!(second.code, Some(1));
    assert!(second.stderr.contains("Plugin already exists at"), "{}", second.stderr);
    assert!(second.stderr.contains("Update it through your plugin manager."), "{}", second.stderr);
    assert_eq!(
        std::fs::read_to_string(plugin.join("plugin/ox-content.lua")).unwrap(),
        "-- locally patched\n"
    );
}

#[test]
fn editor_clis_install_the_extension_once_per_editor_in_order() {
    let project = Project::new();
    for tool in ["code", "cursor", "windsurf", "codium"] {
        project.tool(tool, &format!(r#"echo "{tool} $@" >> "$HOME/tools.log""#));
    }
    let ides = ["cursor", "vscode", "cursor", "vscodium", "windsurf", "vscode"];
    let run = project.run(&install(&ides, &["--extensions-only"]));
    run.success();
    assert_eq!(
        std::fs::read_to_string(project.home().join("tools.log")).unwrap(),
        "cursor --install-extension ubugeeei.vscode-ox-content\n\
         code --install-extension ubugeeei.vscode-ox-content\n\
         codium --install-extension ubugeeei.vscode-ox-content\n\
         windsurf --install-extension ubugeeei.vscode-ox-content\n"
    );
    assert!(project.list(".").is_empty());
    assert!(run.stdout.ends_with("◆ IDE setup applied\n"), "{}", run.stdout);
}

#[test]
fn a_failing_or_missing_editor_cli_is_reported_after_the_workspace_is_configured() {
    let project = Project::new();
    project.tool("code", "exit 0");
    project.tool("cursor", "exit 3");
    let run = project.run(&install(&["vscode", "cursor", "windsurf"], &[]));
    assert_eq!(run.code, Some(1));
    assert!(
        run.stderr.starts_with(
            "Workspace setup was applied, but extension installation needs attention:\n"
        ),
        "{}",
        run.stderr
    );
    let failures: Vec<_> = run.stderr.lines().skip(1).collect();
    assert_eq!(failures.len(), 2, "{failures:?}");
    assert!(
        failures[0].starts_with("cursor: ") && failures[1].starts_with("windsurf: "),
        "{failures:?}"
    );
    assert!(
        failures.iter().all(|line| line.ends_with("Install the IDE CLI and retry.")),
        "{failures:?}"
    );
    assert_eq!(project.read(".vscode/settings.json"), VSCODE_SETTINGS);
    assert_eq!(project.read(".vscode/extensions.json"), VSCODE_EXTENSIONS);
    assert!(!run.stdout.contains("IDE setup applied"), "{}", run.stdout);
}

#[test]
fn default_setup_configures_the_workspace_and_installs_the_extension() {
    let project = Project::new();
    project.tool("code", r#"echo "$@" > "$HOME/code.log""#);
    project.run(&install(&["vscode"], &[])).success();
    assert_eq!(project.read(".vscode/settings.json"), VSCODE_SETTINGS);
    assert_eq!(
        std::fs::read_to_string(project.home().join("code.log")).unwrap(),
        "--install-extension ubugeeei.vscode-ox-content\n"
    );
}
