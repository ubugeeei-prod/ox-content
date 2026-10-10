//! How `oxct lint` finds, reads and combines configuration files.

#[path = "support/oxct.rs"]
mod oxct;

use oxct::Project;

/// Discovery order: the first existing file is the only one read.
const DISCOVERED: [&str; 6] = [
    ".oxlint.json",
    "oxlint.json",
    ".markdownlint.json",
    ".markdownlint.jsonc",
    ".markdownlint.yaml",
    ".markdownlint.yml",
];
const DOCUMENT: &str = "#  Title\n\nwith with \n";
const MARKDOWNLINT: [&str; 2] = ["MD019", "MD009"];
const PROSE: [&str; 2] = ["repeated-word", "trailing-spaces"];

fn project() -> Project {
    let project = Project::new();
    project.write("a.md", DOCUMENT);
    project
}

/// A configuration in `name`'s own format that enables exactly one markdownlint rule.
fn only(name: &str, rule: &str) -> String {
    if name.ends_with("oxlint.json") {
        format!(r#"{{"markdownlint":{{"default":false,"{rule}":true}}}}"#)
    } else if name.ends_with(".json") || name.ends_with(".jsonc") {
        format!(r#"{{"default":false,"{rule}":true}}"#)
    } else {
        format!("default: false\n{rule}: true\n")
    }
}

#[track_caller]
fn rules(project: &Project, args: &[&str]) -> Vec<String> {
    project.run(&[&["lint", "--format", "json"], args].concat()).rules()
}

#[test]
fn without_configuration_only_the_markdownlint_profile_runs() {
    assert_eq!(rules(&project(), &[]), MARKDOWNLINT);
}

#[test]
fn discovered_files_follow_a_fixed_precedence() {
    for (winner, preferred) in DISCOVERED.iter().enumerate() {
        let project = project();
        for (index, name) in DISCOVERED.iter().enumerate().skip(winner) {
            project.write(name, &only(name, if index == winner { "MD009" } else { "MD019" }));
        }
        assert_eq!(rules(&project, &[]), ["MD009"], "{preferred} should win");
    }
}

#[test]
fn every_discovered_name_is_read_on_its_own() {
    for name in DISCOVERED {
        let project = project();
        project.write(name, &only(name, "MD019"));
        assert_eq!(rules(&project, &[]), ["MD019"], "{name}");
    }
}

#[test]
fn an_explicit_config_replaces_discovery_and_may_live_anywhere() {
    let project = project();
    project.write(".oxlint.json", &only(".oxlint.json", "MD019"));
    project.write("config/lint/custom.yaml", "default: false\nMD009: true\n");
    assert_eq!(rules(&project, &["--config", "config/lint/custom.yaml"]), ["MD009"]);
    let absolute = project.file("config/lint/custom.yaml");
    assert_eq!(rules(&project, &["--config", absolute.to_str().unwrap()]), ["MD009"]);
    // Discovery only looks in the working directory, never in parents.
    project.write("nested/b.md", DOCUMENT);
    let nested = project.run_in("nested", &["lint", "--format", "json"]);
    assert_eq!(nested.rules(), MARKDOWNLINT);
}

#[test]
fn markdownlint_profile_toggles_combine_with_ox_configuration() {
    let object = r#"{"markdownlint":{"default":false,"MD009":true}}"#;
    for (config, flags, expected) in [
        (None, &[][..], &MARKDOWNLINT[..]),
        (None, &["--no-markdownlint"], &PROSE),
        (None, &["--markdownlint"], &MARKDOWNLINT),
        // An Ox Content configuration opts out of the profile unless it asks for it.
        (Some(r#"{"rules":{}}"#), &[], &PROSE),
        (Some(r#"{"rules":{}}"#), &["--markdownlint"], &MARKDOWNLINT),
        (
            Some(r#"{"rules":{"repeatedWords":true}}"#),
            &["--markdownlint"],
            &["MD019", "repeated-word", "MD009"],
        ),
        (Some(r#"{"markdownlint":false}"#), &[], &PROSE),
        (Some(r#"{"markdownlint":false}"#), &["--markdownlint"], &MARKDOWNLINT),
        (Some(r#"{"markdownlint":true}"#), &[], &MARKDOWNLINT),
        (Some(r#"{"markdownlint":true}"#), &["--no-markdownlint"], &PROSE),
        (Some(object), &[], &["MD009"]),
        (Some(object), &["--markdownlint"], &["MD009"]),
        (Some(object), &["--no-markdownlint"], &PROSE),
    ] {
        let project = project();
        if let Some(config) = config {
            project.write(".oxlint.json", config);
        }
        assert_eq!(rules(&project, flags), expected, "{config:?} {flags:?}");
    }
}

#[test]
fn file_name_and_keys_decide_between_the_two_configuration_formats() {
    let project = project();
    for (name, content, expected) in [
        // An unrecognised shape under a neutral name is a markdownlint configuration.
        ("custom.json", "{}", &MARKDOWNLINT[..]),
        ("custom.json", r#"{"MD019":false}"#, &["MD009"]),
        ("custom.json", r#"{"noInlineConfig":true}"#, &PROSE),
        ("custom.json", r#"{"ignore":[]}"#, &PROSE),
        ("custom.yaml", "include: []\n", &PROSE),
        ("oxlint.json", "{}", &PROSE),
        ("nested/.oxlint.json", "{}", &PROSE),
    ] {
        project.write(name, content);
        assert_eq!(rules(&project, &["--config", name, "a.md"]), expected, "{name}: {content}");
    }
    project.write("oxlint.json", r#"{"MD009":true}"#);
    project.run(&["lint", "--config", "oxlint.json"]).rejected("unknown field `MD009`");
    project.write("mixed.json", r#"{"rules":{},"MD009":true}"#);
    project.run(&["lint", "--config", "mixed.json"]).rejected("unknown field `MD009`");
}

#[test]
fn json_configuration_accepts_comments_and_trailing_commas() {
    let project = project();
    project.write(
        ".markdownlint.json",
        "{\n  // only whitespace\n  \"default\": false, /* inline */\n  \"MD009\": true,\n}\n",
    );
    assert_eq!(rules(&project, &[]), ["MD009"]);
    project.write(".oxlint.json", "// prose only\n{\"rules\": {\"trailingSpaces\": false,},}\n");
    assert_eq!(rules(&project, &[]), ["repeated-word"]);
}

#[test]
fn markdownlint_rules_accept_aliases_and_options() {
    let project = project();
    for (config, expected) in [
        (r#"{"default":false,"no-trailing-spaces":true}"#, &["MD009"][..]),
        (r#"{"default":false,"no-multiple-space-atx":true}"#, &["MD019"]),
        (r#"{"MD009":false,"MD019":false}"#, &[]),
        // Fewer than two break spaces forbids every trailing space, as in markdownlint.
        (r#"{"MD009":{"br_spaces":1}}"#, &MARKDOWNLINT),
        // A line is only too long when whitespace follows the limit: the heading ends in one word.
        (r#"{"default":false,"MD013":{"line_length":5}}"#, &["MD013"]),
        (r#"{"default":false,"MD013":{"line_length":5,"strict":true}}"#, &["MD013", "MD013"]),
    ] {
        project.write(".markdownlint.json", config);
        assert_eq!(rules(&project, &[]), expected, "{config}");
    }
}

#[test]
fn extends_merges_bases_in_order_and_lets_the_child_win() {
    let project = project();
    project.write("b1.json", r#"{"default":false,"MD009":true,"MD019":true}"#);
    project.write("b2.json", r#"{"MD019":false}"#);
    for (config, expected) in [
        (r#"{"extends":"b1.json"}"#, &MARKDOWNLINT[..]),
        (r#"{"extends":["b1.json"]}"#, &MARKDOWNLINT),
        (r#"{"extends":["b1.json","b2.json"]}"#, &["MD009"]),
        (r#"{"extends":["b2.json","b1.json"]}"#, &MARKDOWNLINT),
        (r#"{"extends":["b1.json","b2.json"],"MD009":false}"#, &[]),
        (r#"{"extends":["b1.json","b2.json"],"MD019":true}"#, &MARKDOWNLINT),
        (r#"{"extends":[]}"#, &MARKDOWNLINT),
    ] {
        project.write(".markdownlint.json", config);
        assert_eq!(rules(&project, &[]), expected, "{config}");
    }
}

#[test]
fn extends_resolves_relative_to_the_extending_file_across_formats() {
    let project = project();
    project.write("shared/base.yaml", "default: false\nMD009: true\n");
    project.write("shared/team/strict.jsonc", "{\n// team\n\"extends\": \"../base.yaml\",\n}");
    project.write(".markdownlint.yml", "extends: shared/team/strict.jsonc\nMD019: true\n");
    assert_eq!(rules(&project, &[]), MARKDOWNLINT);
    // The same chain works from another working directory through `--config`.
    project.write("elsewhere/b.md", "text \n");
    let run = project
        .run_in("elsewhere", &["lint", "--config", "../.markdownlint.yml", "--format", "json"]);
    assert_eq!(run.rules(), ["MD009"]);
}

#[test]
fn ox_configuration_can_extend_another_ox_configuration() {
    let project = project();
    project.write("base.json", r#"{"rules":{"trailingSpaces":false},"ignore":["skip.md"]}"#);
    project.write(".oxlint.json", r#"{"extends":"base.json","rules":{"repeatedWords":true}}"#);
    project.write("skip.md", DOCUMENT);
    // Objects are replaced key by key, so the child's `rules` wins as a whole.
    let run = project.run(&["lint", "--format", "json"]);
    assert_eq!(run.json()["checkedFileCount"], 1);
    assert_eq!(run.rules(), PROSE);
}

#[test]
fn shared_bases_are_not_mistaken_for_cycles() {
    let project = project();
    project.write("root.json", r#"{"default":false,"MD009":true}"#);
    project.write("left.json", r#"{"extends":"root.json"}"#);
    project.write("right.json", r#"{"extends":"root.json"}"#);
    project.write(".markdownlint.json", r#"{"extends":["left.json","right.json"]}"#);
    assert_eq!(rules(&project, &[]), ["MD009"]);
}

#[test]
fn extends_cycles_and_runaway_chains_are_rejected() {
    let project = project();
    project.write(".markdownlint.json", r#"{"extends":".markdownlint.json"}"#);
    project.run(&["lint"]).rejected("Circular markdownlint extends");
    project.write(".markdownlint.json", r#"{"extends":"./sub/../.markdownlint.json"}"#);
    project.write("sub/keep.txt", "");
    project.run(&["lint"]).rejected("Circular markdownlint extends");
    let chain = |length: usize| {
        let project = self::project();
        for index in 0..length {
            let content = if index + 1 == length {
                r#"{"default":false,"MD009":true}"#.to_string()
            } else {
                format!(r#"{{"extends":"chain-{}.json"}}"#, index + 1)
            };
            project.write(&format!("chain-{index}.json"), &content);
        }
        project.run(&["lint", "--config", "chain-0.json", "--format", "json"])
    };
    assert_eq!(chain(32).rules(), ["MD009"]);
    chain(33).rejected("exceeds 32 configuration files");
}

#[test]
fn invalid_configuration_fails_without_linting_or_fixing() {
    for (name, content, reason) in [
        ("custom.json", "{", "Unterminated object"),
        ("custom.yaml", "a: [\n", "while parsing"),
        ("custom.json", "[1]", "markdownlint must be a boolean or configuration object"),
        ("custom.json", "\"text\"", "markdownlint must be a boolean or configuration object"),
        ("custom.json", r#"{"MD099":true}"#, "Unknown"),
        ("custom.json", r#"{"MD009":"info"}"#, "Invalid configuration for MD009"),
        ("custom.json", r#"{"MD013":{"lenght":80}}"#, "Invalid markdownlint configuration"),
        ("custom.json", r#"{"extends":5}"#, "extends must be a path or path array"),
        ("custom.json", r#"{"extends":[5]}"#, "extends must contain paths"),
        ("custom.json", r#"{"extends":"missing.json"}"#, "(os error 2)"),
        ("custom.json", r#"{"extends":"list.json"}"#, "Extended configuration must be an object"),
        ("custom.json", r#"{"unknownKey":1,"rules":{}}"#, "unknown field `unknownKey`"),
        ("custom.json", r#"{"rules":{"nope":true}}"#, "unknown field `nope`"),
        ("custom.json", r#"{"rules":{"trailingSpaces":"yes"}}"#, "Invalid lint rule"),
        ("custom.json", r#"{"textRules":{"sentenseLength":10}}"#, "unknown field `sentenseLength`"),
        ("custom.json", r#"{"severities":{"repeated-word":"fatal"}}"#, "Invalid lint rule"),
        ("custom.json", r#"{"include":"docs"}"#, "Invalid lint rule"),
        ("oxlint.json", "[]", "Invalid lint rule or configuration"),
    ] {
        let project = project();
        project.write("list.json", "[]");
        project.write(name, content);
        project.run(&["lint", "--fix", "--config", name]).rejected(reason);
        assert_eq!(project.read("a.md"), DOCUMENT, "{content}");
    }
}

#[cfg(unix)]
#[test]
fn a_missing_configuration_file_is_an_error_not_a_fallback_to_defaults() {
    let project = project();
    project.run(&["lint", "--config", "missing.json"]).rejected("No such file or directory");
    project.run(&["lint", "--config", "a.md/nested.json"]).rejected("Not a directory");
}
