//! Systems and general-purpose language grammars.

use super::Grammar;

/// C++ extends C the same way TypeScript extends JavaScript: its queries
/// declare only the additions, so C's have to come first.
fn cpp_highlights() -> String {
    format!("{}\n{}", tree_sitter_c::HIGHLIGHT_QUERY, tree_sitter_cpp::HIGHLIGHT_QUERY)
}

pub(super) fn grammars() -> &'static [Grammar] {
    &[
        grammar!(
            "rust",
            ["rust", "rs"],
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        ),
        grammar!(
            "python",
            ["python", "py"],
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "go",
            ["go", "golang"],
            tree_sitter_go::LANGUAGE,
            tree_sitter_go::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "java",
            ["java"],
            tree_sitter_java::LANGUAGE,
            tree_sitter_java::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!("c", ["c", "h"], tree_sitter_c::LANGUAGE, tree_sitter_c::HIGHLIGHT_QUERY, "", "",),
        grammar!(
            "cpp",
            ["cpp", "c++", "cc", "hpp", "cxx"],
            tree_sitter_cpp::LANGUAGE,
            cpp_highlights(),
            "",
            "",
        ),
    ]
}
