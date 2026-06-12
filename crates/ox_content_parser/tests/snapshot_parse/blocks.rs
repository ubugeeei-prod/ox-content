use ox_content_parser::ParserOptions;

use super::check;

// --- Code blocks ---

#[test]
fn snapshot_fenced_code_with_lang_and_meta() {
    check(
        "fenced_code_with_lang_and_meta",
        "```ts filename=main.ts\nconst answer = 42;\n```\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_fenced_code_tildes_with_meta() {
    check(
        "fenced_code_tildes_with_meta",
        "~~~rust title=lib.rs\nfn main() {}\n~~~\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_fenced_code_unclosed_until_eof() {
    check("fenced_code_unclosed_until_eof", "```rs\nfn main() {}\n", ParserOptions::default());
}

#[test]
fn snapshot_fenced_code_indented_inside_list_item() {
    check(
        "fenced_code_indented_inside_list_item",
        "1. text\n\n   ```ts\n   const a = 1;\n   ```\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_indented_code_block_preserves_blank_lines() {
    check(
        "indented_code_block_preserves_blank_lines",
        "    line a\n\n    line b\n    line c\n",
        ParserOptions::default(),
    );
}

// --- Block quotes ---

#[test]
fn snapshot_blockquote_single_line() {
    check("blockquote_single_line", "> hello\n", ParserOptions::default());
}

#[test]
fn snapshot_blockquote_multi_paragraph() {
    check("blockquote_multi_paragraph", "> first\n>\n> second\n", ParserOptions::default());
}

#[test]
fn snapshot_blockquote_nested() {
    check(
        "blockquote_nested",
        "> outer\n>\n> > inner\n>\n> outer again\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_blockquote_with_nested_list_and_code() {
    check(
        "blockquote_with_nested_list_and_code",
        "> - item one\n> - item two\n>\n> ```rs\n> fn f() {}\n> ```\n",
        ParserOptions::default(),
    );
}
