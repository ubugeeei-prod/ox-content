use super::super::*;
use super::fixtures::test_entry;

pub(in crate::markdown::tests) fn object_literal_parameter_entry() -> ApiDocEntry {
    let mut entry = test_entry("plugin", "function", "/repo/src/plugin.ts", "Define a plugin.");
    entry.params = vec![
        ApiParamDoc {
            name: "options".to_string(),
            type_annotation:
                "{ id: Id; name?: string; setup?: (ctx: PluginContext) => Awaitable<void> }"
                    .to_string(),
            description: "Plugin options.".to_string(),
            ..ApiParamDoc::default()
        },
        ApiParamDoc {
            name: "options.id".to_string(),
            type_annotation: "Id".to_string(),
            description: "Plugin id.".to_string(),
            ..ApiParamDoc::default()
        },
        ApiParamDoc {
            name: "options.name?".to_string(),
            type_annotation: "string".to_string(),
            optional: true,
            ..ApiParamDoc::default()
        },
        ApiParamDoc {
            name: "options.setup?".to_string(),
            type_annotation: "(ctx: PluginContext) => Awaitable<void>".to_string(),
            description: "Setup hook.".to_string(),
            optional: true,
            ..ApiParamDoc::default()
        },
    ];
    entry
}

pub(in crate::markdown::tests) fn member(name: &str, kind: &str, is_static: bool) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: kind.to_string(),
        type_annotation: Some("unknown".to_string()),
        r#static: is_static,
        ..ApiDocMember::default()
    }
}

pub(in crate::markdown::tests) fn function_valued_parse_member() -> ApiDocMember {
    ApiDocMember {
        name: "parse".to_string(),
        kind: "property".to_string(),
        description: "Parses a raw value.".to_string(),
        type_annotation: Some("(value: string) => string | undefined".to_string()),
        params: vec![ApiParamDoc {
            name: "value".to_string(),
            type_annotation: "string".to_string(),
            description: "Raw string value from command line.".to_string(),
            ..ApiParamDoc::default()
        }],
        returns: Some(ApiReturnDoc {
            type_annotation: "string | undefined".to_string(),
            description: "Parsed value.".to_string(),
            ..ApiReturnDoc::default()
        }),
        optional: true,
        line: 5,
        end_line: 10,
        ..ApiDocMember::default()
    }
}

pub(in crate::markdown::tests) fn return_property(
    name: &str,
    type_annotation: &str,
) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: "property".to_string(),
        type_annotation: Some(type_annotation.to_string()),
        ..ApiDocMember::default()
    }
}

pub(in crate::markdown::tests) fn index_signature_docs() -> Vec<ApiDocModule> {
    let mut schema = test_entry("ArgSchema", "interface", "/repo/src/args.ts", "Value type.");
    schema.signature = Some("export interface ArgSchema".to_string());

    let mut args = test_entry("Args", "interface", "/repo/src/args.ts", "Arguments.");
    args.signature = Some("export interface Args".to_string());
    args.members =
        vec![index_signature_member("[option: string]", "option", "string", "ArgSchema", true)];

    vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![schema, args],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn index_signature_member(
    name: &str,
    param_name: &str,
    param_type: &str,
    value_type: &str,
    readonly: bool,
) -> ApiDocMember {
    ApiDocMember {
        name: name.to_string(),
        kind: "indexSignature".to_string(),
        description: "Argument schema by option name.".to_string(),
        signature: Some(if readonly {
            format!("readonly {name}: {value_type}")
        } else {
            format!("{name}: {value_type}")
        }),
        type_annotation: Some(value_type.to_string()),
        params: vec![param(param_name, param_type)],
        readonly,
        ..ApiDocMember::default()
    }
}

fn param(name: &str, type_annotation: &str) -> ApiParamDoc {
    ApiParamDoc {
        name: name.to_string(),
        type_annotation: type_annotation.to_string(),
        ..ApiParamDoc::default()
    }
}
