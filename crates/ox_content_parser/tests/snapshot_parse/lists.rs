use ox_content_parser::ParserOptions;

use super::check;

// --- Lists ---

#[test]
fn snapshot_unordered_list_dash_marker() {
    check("unordered_list_dash_marker", "- a\n- b\n- c\n", ParserOptions::default());
}

#[test]
fn snapshot_wrapped_list_item_continuation() {
    check(
        "wrapped_list_item_continuation",
        "- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI and\n  Testbox infrastructure across projects.\n- [Mates Inc.](https://eng.mates.education/) for supporting OSS and\n  adopting Vize in production.\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_ordered_list_with_start() {
    check("ordered_list_with_start", "3. third\n4. fourth\n", ParserOptions::default());
}

#[test]
fn snapshot_ordered_list_parenthesis_marker() {
    check("ordered_list_parenthesis_marker", "3) third\n4) fourth\n", ParserOptions::default());
}

#[test]
fn snapshot_nested_lists_mixed_markers() {
    check(
        "nested_lists_mixed_markers",
        "- parent\n  - child\n  - second child\n    1. deep one\n    2. deep two\n- sibling\n",
        ParserOptions::default(),
    );
}

#[test]
fn snapshot_task_list_literal_without_gfm() {
    check("task_list_literal_without_gfm", "- [x] done\n- [ ] todo\n", ParserOptions::default());
}

#[test]
fn snapshot_task_list_with_gfm() {
    check("task_list_with_gfm", "- [ ] todo\n- [x] done\n", ParserOptions::gfm());
}

#[test]
fn snapshot_loose_vs_tight_list() {
    check(
        "loose_vs_tight_list",
        "- tight a\n- tight b\n\n- loose a\n\n- loose b\n",
        ParserOptions::default(),
    );
}
