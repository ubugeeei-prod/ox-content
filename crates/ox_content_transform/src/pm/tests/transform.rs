use super::*;

#[test]
fn transform_expands_pm_block() {
    let result = transform_pm("<pm>npm install -D vite</pm>", 0, PmOptions { sync: false });
    assert_eq!(result.group_count, 1);
    insta::assert_snapshot!(result.html);
}

#[test]
fn transform_handles_code_block_inner() {
    let result =
        transform_pm("<pm><pre><code>npm i vite</code></pre></pm>", 0, PmOptions { sync: false });
    insta::assert_snapshot!(result.html);
}

#[test]
fn group_attr_only_when_sync_enabled() {
    let off = transform_pm("<pm>npm i vite</pm>", 0, PmOptions { sync: false });
    let on = transform_pm("<pm>npm i vite</pm>", 0, PmOptions { sync: true });
    insta::assert_snapshot!(format!("off:\n{}\n---\non:\n{}", off.html, on.html));
}

#[test]
fn numbers_groups_and_passes_through_without_marker() {
    let result = transform_pm("<p>nothing here</p>", 7, PmOptions::default());
    assert_eq!(result.group_count, 0);
    assert_eq!(result.html, "<p>nothing here</p>");
}

#[test]
fn numbers_multiple_groups_from_start() {
    let result = transform_pm("<pm>npm i a</pm> mid <pm>npm i b</pm>", 3, PmOptions::default());
    assert_eq!(result.group_count, 2);
    insta::assert_snapshot!(result.html);
}

#[test]
fn empty_pm_block_left_untouched() {
    let html = "<pm>   </pm>";
    let result = transform_pm(html, 0, PmOptions::default());
    assert_eq!(result.group_count, 0);
    assert_eq!(result.html, html);
}
