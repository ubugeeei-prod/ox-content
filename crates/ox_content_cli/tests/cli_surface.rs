//! Top-level `oxct` behaviour: usage, versions, help for every command, and unknown input.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

const USAGE: &str = "oxct <command> [options]\n\nCommands: new [directory], ide install, lint [files/globs], tui [file/directory]\n";

#[test]
fn bare_invocation_and_help_flags_print_the_same_usage() {
    let project = Project::new();
    for args in [&[][..], &["--help"], &["-h"], &["--help", "ignored"], &["-h", "lint"]] {
        let run = project.run(args);
        run.success();
        assert_eq!((run.stdout.as_str(), run.stderr.as_str()), (USAGE, ""), "{args:?}");
    }
}

#[test]
fn version_matches_the_crate_version() {
    let run = Project::new().run(&["--version"]);
    run.success();
    assert_eq!(run.stdout, format!("{}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_commands_fail_with_their_name() {
    let project = Project::new();
    // `typecheck` and `lsp` belong to the npm launcher, not the native binary.
    for command in ["frobnicate", "typecheck", "lsp", "Lint", "-v", "--verbose", ""] {
        project.run(&[command, "--help"]).rejected(&format!("Unknown native command: {command}"));
    }
}

#[test]
fn every_command_documents_all_of_its_options() {
    let project = Project::new();
    for (command, usage, options) in [
        (
            &["new"][..],
            "Usage: oxct new [OPTIONS] [DIRECTORY]",
            &[
                "--template <TEMPLATE>",
                "--skin <SKIN>",
                "--palette <PALETTE>",
                "--package-manager <MANAGER>",
                "--install",
                "--no-install",
                "--yes",
            ][..],
        ),
        (
            &["ide", "install"],
            "Usage: oxct ide install [OPTIONS]",
            &["--ide <IDE>", "--config-only", "--extensions-only", "--dry-run", "--yes"],
        ),
        (
            &["lint"],
            "Usage: oxct lint [OPTIONS] [PATHS]...",
            &[
                "--config <CONFIG>",
                "--markdownlint",
                "--no-markdownlint",
                "--list-rules",
                "--no-inline-config",
                "--ignore <IGNORE>",
                "--format <FORMAT>",
                "--stdin",
                "--stdin-filepath <STDIN_FILEPATH>",
                "--spellcheck",
                "--strict",
                "--fix",
                "--threads <THREADS>",
                "--max-warnings <MAX_WARNINGS>",
                "--no-color",
            ],
        ),
        (
            &["tui"],
            "Usage: oxct tui [OPTIONS] [PATH]",
            &[
                "--theme <THEME>",
                "--print",
                "--stdin",
                "--width <WIDTH>",
                "--no-watch",
                "--no-color",
            ],
        ),
    ] {
        for flag in ["--help", "-h"] {
            let run = project.run(&[command, &[flag]].concat());
            run.success();
            assert_eq!(run.stderr, "", "{command:?} {flag}");
            assert!(run.stdout.starts_with(usage), "{command:?} {flag}: {}", run.stdout);
            for option in options {
                assert!(run.stdout.contains(option), "{command:?} is missing {option}");
            }
            assert!(!run.stdout.contains("--version"), "{command:?} {flag}");
        }
    }
}

#[test]
fn help_lists_the_accepted_values_for_enumerated_options() {
    let project = Project::new();
    let lint = project.run(&["lint", "--help"]).stdout;
    assert!(lint.contains("[default: text] [possible values: text, json]"), "{lint}");
    assert!(lint.contains("[default: stdin.md]"), "{lint}");
    let tui = project.run(&["tui", "--help"]).stdout;
    assert!(tui.contains("[default: nord] [possible values: nord, light, mono]"), "{tui}");
    let ide = project.run(&["ide", "install", "--help"]).stdout;
    assert!(
        ide.contains("[possible values: vscode, cursor, windsurf, vscodium, zed, neovim]"),
        "{ide}"
    );
}

#[test]
fn help_wins_over_other_arguments_without_side_effects() {
    let project = Project::new();
    project.write("a.md", "# Title \n");
    for args in [
        &["new", "site", "--template", "blog", "--yes", "--help"][..],
        &["lint", "--fix", "--help"],
        &["ide", "install", "--ide", "vscode", "--config-only", "--yes", "--help"],
        &["tui", "a.md", "--help"],
    ] {
        let run = project.run(args);
        run.success();
        assert!(run.stdout.starts_with("Usage: oxct "), "{args:?}: {}", run.stdout);
    }
    assert_eq!(project.list("."), ["a.md"]);
    assert_eq!(project.read("a.md"), "# Title \n");
}

#[test]
fn commands_have_no_version_flag_of_their_own() {
    let project = Project::new();
    for args in [&["new", "--version"][..], &["lint", "--version"], &["tui", "--version"]] {
        project.run(args).rejected("unexpected argument '--version'");
    }
    project.run(&["ide", "install", "--version"]).rejected("unexpected argument '--version'");
    assert!(project.list(".").is_empty());
}

#[test]
fn usage_errors_name_the_command_and_point_to_help() {
    let project = Project::new();
    for (args, usage) in [
        (&["new", "--xyzzy"][..], "Usage: oxct new [OPTIONS] [DIRECTORY]"),
        (&["lint", "--xyzzy"], "Usage: oxct lint [OPTIONS] [PATHS]..."),
        (&["tui", "--xyzzy"], "Usage: oxct tui [OPTIONS] [PATH]"),
        (&["ide", "install", "--xyzzy"], "Usage: oxct ide install [OPTIONS]"),
    ] {
        let run = project.run(args);
        run.rejected("unexpected argument '--xyzzy'");
        assert!(run.stderr.contains(usage), "{args:?}: {}", run.stderr);
        assert!(run.stderr.contains("For more information, try '--help'."), "{}", run.stderr);
    }
}
