//! Application, scripting, and shader grammars.

use super::Grammar;

pub(super) fn grammars() -> &'static [Grammar] {
    &[
        grammar!(
            "wgsl",
            ["wgsl"],
            tree_sitter_wgsl_bevy::LANGUAGE,
            include_str!("../../queries/wgsl-highlights.scm"),
            "",
            "",
        ),
        grammar!(
            "sql",
            ["sql"],
            tree_sitter_sequel::LANGUAGE,
            tree_sitter_sequel::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "graphql",
            ["graphql", "gql"],
            tree_sitter_graphql::LANGUAGE,
            include_str!("../../queries/graphql-highlights.scm"),
            "",
            "",
        ),
        grammar!(
            "dockerfile",
            ["dockerfile", "docker", "containerfile"],
            tree_sitter_containerfile::LANGUAGE,
            tree_sitter_containerfile::HIGHLIGHTS_QUERY,
            tree_sitter_containerfile::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "ruby",
            ["ruby", "rb"],
            tree_sitter_ruby::LANGUAGE,
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_ruby::LOCALS_QUERY,
        ),
        grammar!(
            "php",
            ["php"],
            tree_sitter_php::LANGUAGE_PHP,
            tree_sitter_php::HIGHLIGHTS_QUERY,
            tree_sitter_php::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "nix",
            ["nix"],
            tree_sitter_nix::LANGUAGE,
            tree_sitter_nix::HIGHLIGHTS_QUERY,
            tree_sitter_nix::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "csharp",
            ["csharp", "cs"],
            tree_sitter_c_sharp::LANGUAGE,
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "swift",
            ["swift"],
            tree_sitter_swift::LANGUAGE,
            tree_sitter_swift::HIGHLIGHTS_QUERY,
            tree_sitter_swift::INJECTIONS_QUERY,
            tree_sitter_swift::LOCALS_QUERY,
        ),
        grammar!(
            "kotlin",
            ["kotlin", "kt"],
            tree_sitter_kotlin_ng::LANGUAGE,
            include_str!("../../queries/kotlin-highlights.scm"),
            "",
            "",
        ),
        grammar!(
            "lua",
            ["lua"],
            tree_sitter_lua::LANGUAGE,
            tree_sitter_lua::HIGHLIGHTS_QUERY,
            tree_sitter_lua::INJECTIONS_QUERY,
            tree_sitter_lua::LOCALS_QUERY,
        ),
        grammar!(
            "glsl",
            ["glsl"],
            tree_sitter_glsl::LANGUAGE_GLSL,
            include_str!("../../queries/glsl-highlights.scm"),
            "",
            "",
        ),
    ]
}
