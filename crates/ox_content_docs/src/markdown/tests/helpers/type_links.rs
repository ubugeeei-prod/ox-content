use super::super::*;
use super::fixtures::test_entry;

pub(in crate::markdown::tests) fn type_param(name: &str) -> ApiParamDoc {
    ApiParamDoc { name: name.to_string(), ..ApiParamDoc::default() }
}

/// A parameter with a name and a type annotation (no description/flags).
pub(in crate::markdown::tests) fn param(name: &str, type_annotation: &str) -> ApiParamDoc {
    ApiParamDoc { type_annotation: type_annotation.to_string(), ..type_param(name) }
}

/// A `type` entry stub so its name resolves in the symbol map (for type links).
pub(in crate::markdown::tests) fn type_stub(name: &str) -> ApiDocEntry {
    let mut entry = test_entry(name, "type", "/repo/src/types.ts", "");
    entry.signature = None;
    entry
}

/// A `function` entry stub (e.g. a combinator) so its name resolves in the
/// symbol map even when it collides with a primitive type name.
pub(in crate::markdown::tests) fn function_stub(name: &str) -> ApiDocEntry {
    test_entry(name, "function", "/repo/src/combinators.ts", "")
}

/// A module containing `entry` plus stub `type` entries whose names are used as
/// linkable symbols inside type annotations in the type-link tests.
pub(in crate::markdown::tests) fn type_link_module(entry: ApiDocEntry) -> Vec<ApiDocModule> {
    vec![ApiDocModule {
        file: "combinators".to_string(),
        entries: vec![
            entry,
            type_stub("RenderingOptions"),
            type_stub("SubCommandable"),
            type_stub("CommandRunner"),
            type_stub("GunshiParamsConstraint"),
            type_stub("DefaultGunshiParams"),
            type_stub("PluginExtension"),
            type_stub("ArgValues"),
            type_stub("ArgExplicitlyProvided"),
            type_stub("U"),
            // Symbols that collide with TypeScript intrinsic primitive types,
            // mirroring gunshi's `string()` / `boolean()` / `number()`
            // combinators. These must never be linked inside a type annotation.
            function_stub("string"),
            function_stub("boolean"),
            function_stub("number"),
        ],
        ..ApiDocModule::default()
    }]
}

pub(in crate::markdown::tests) fn multiline_plugin_ext_type_parameters() -> Vec<ApiTypeParamDoc> {
    vec![
        ApiTypeParamDoc { name: "Extension".to_string(), ..ApiTypeParamDoc::default() },
        ApiTypeParamDoc { name: "ResolvedDepExtensions".to_string(), ..ApiTypeParamDoc::default() },
        ApiTypeParamDoc {
            name: "PluginExt".to_string(),
            constraint: Some("PluginExtension<Extension, DefaultGunshiParams>".to_string()),
            default: Some(
                "PluginExtension<\n    Extension,\n    ResolvedDepExtensions\n  >".to_string(),
            ),
            ..ApiTypeParamDoc::default()
        },
    ]
}
