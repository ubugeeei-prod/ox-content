use super::super::*;

pub(in crate::markdown::tests) fn test_entry(
    name: &str,
    kind: &str,
    file: &str,
    description: &str,
) -> ApiDocEntry {
    ApiDocEntry {
        name: name.to_string(),
        kind: kind.to_string(),
        description: description.to_string(),
        file: file.to_string(),
        signature: Some(join3("export function ", name, "(): void")),
        ..ApiDocEntry::default()
    }
}

pub(in crate::markdown::tests) fn link_test_docs() -> Vec<ApiDocModule> {
    vec![
        ApiDocModule {
            file: "/repo/src/context.ts".to_string(),
            entries: vec![test_entry(
                "CommandContext",
                "interface",
                "/repo/src/context.ts",
                "Command context.",
            )],
            ..ApiDocModule::default()
        },
        ApiDocModule {
            file: "/repo/src/command.ts".to_string(),
            entries: vec![test_entry(
                "Command",
                "function",
                "/repo/src/command.ts",
                "Runs with [CommandContext].",
            )],
            ..ApiDocModule::default()
        },
    ]
}

pub(in crate::markdown::tests) fn pure_test_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        file: "/repo/src/cli.ts".to_string(),
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Runs the CLI.".to_string(),
                params: vec![ApiParamDoc {
                    name: "argv".to_string(),
                    type_annotation: "string[]".to_string(),
                    description: "Arguments.".to_string(),
                    ..ApiParamDoc::default()
                }],
                returns: Some(ApiReturnDoc {
                    type_annotation: "void".to_string(),
                    description: "Nothing.".to_string(),
                    ..ApiReturnDoc::default()
                }),
                throws: vec![ApiThrowsDoc {
                    type_annotation: Some("CliError".to_string()),
                    description: "When argument parsing fails.".to_string(),
                }],
                examples: vec!["```ts\ncli([])\n```".to_string()],
                tags: vec![ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() }],
                file: "/repo/src/cli.ts".to_string(),
                end_line: 3,
                signature: Some("export function cli(argv: string[]): void".to_string()),
                ..ApiDocEntry::default()
            },
            ApiDocEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "A command.".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                line: 5,
                end_line: 8,
                signature: Some("export interface Command".to_string()),
                members: vec![ApiDocMember {
                    name: "run".to_string(),
                    kind: "method".to_string(),
                    description: "Runs it.".to_string(),
                    signature: Some("run(): void".to_string()),
                    throws: vec![ApiThrowsDoc {
                        type_annotation: Some("RunError".to_string()),
                        description: "When the command cannot run.".to_string(),
                    }],
                    line: 6,
                    end_line: 6,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            },
        ],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn lifecycle_module(entry: ApiDocEntry) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        file: "combinators".to_string(),
        entries: vec![entry],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn group_order_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: "Module description.".to_string(),
        file: "default".to_string(),
        entries: vec![
            test_entry("alpha", "function", "/repo/src/a.ts", "A function."),
            test_entry("Config", "interface", "/repo/src/c.ts", "An interface."),
            test_entry("Engine", "class", "/repo/src/e.ts", "A class."),
            test_entry("VERSION", "variable", "/repo/src/v.ts", "A variable."),
        ],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn stats_docs() -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        description: "Module description.".to_string(),
        file: "default".to_string(),
        entries: vec![
            test_entry("cli", "function", "/repo/src/cli.ts", "Run the CLI."),
            test_entry("run", "function", "/repo/src/run.ts", "Run again."),
        ],
        ..ApiDocModule::default()
    }]
}
