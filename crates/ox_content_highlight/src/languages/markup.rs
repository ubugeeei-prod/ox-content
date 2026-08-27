//! Markup, config, and shell grammars.

use super::Grammar;

/// Markdown ships as a pair of grammars: a block one that leaves each run of
/// prose as a single `inline` node, and a separate inline one for what is
/// inside it. Injecting the second into the first is what gets emphasis, links
/// and code spans highlighted; the block grammar alone sees only headings,
/// fences and punctuation.
///
/// `injection.include-children` is load-bearing. Without it the injected layer
/// is created — the language callback even fires — and then emits nothing at
/// all, because the range handed to it has the content node's children carved
/// out of it. It fails silently, as unhighlighted prose.
fn markdown_block_injections() -> String {
    format!(
        "{}\n((inline) @injection.content \
         (#set! injection.language \"markdown_inline\") \
         (#set! injection.include-children))",
        tree_sitter_md::INJECTION_QUERY_BLOCK,
    )
}

pub(super) fn grammars() -> &'static [Grammar] {
    &[
        grammar!(
            "yaml",
            ["yaml", "yml"],
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "markdown",
            ["markdown", "md"],
            tree_sitter_md::LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            markdown_block_injections(),
            "",
        ),
        // Reachable only as an injection target, never as a fence tag.
        grammar!(
            "markdown_inline",
            [],
            tree_sitter_md::INLINE_LANGUAGE,
            tree_sitter_md::HIGHLIGHT_QUERY_INLINE,
            tree_sitter_md::INJECTION_QUERY_INLINE,
            "",
        ),
        grammar!(
            "bash",
            ["bash", "sh", "shell", "zsh", "shellscript"],
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY,
            "",
            "",
        ),
        grammar!(
            "toml",
            ["toml"],
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "xml",
            ["xml", "svg", "xsl", "xslt", "rss", "atom", "plist", "xsd"],
            tree_sitter_xml::LANGUAGE_XML,
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
            "",
            "",
        ),
        grammar!(
            "diff",
            ["diff", "patch", "udiff"],
            tree_sitter_diff::LANGUAGE,
            tree_sitter_diff::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
        grammar!(
            "hcl",
            ["hcl", "terraform", "tf", "tfvars"],
            tree_sitter_hcl::LANGUAGE,
            include_str!("../../queries/hcl-highlights.scm"),
            "",
            "",
        ),
        grammar!(
            "make",
            ["make", "makefile", "mk"],
            tree_sitter_make::LANGUAGE,
            tree_sitter_make::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
    ]
}
