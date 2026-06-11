use super::transform_tabs;

#[test]
fn expands_two_tabs_matching_characterization() {
    let result =
        transform_tabs(r#"<tabs><tab title="A">alpha</tab><tab title="B">beta</tab></tabs>"#, 0);
    assert_eq!(result.group_count, 1);
    assert_eq!(
        result.html,
        r#"<div class="ox-tabs-container"><div class="ox-tabs" data-group="0"><div class="ox-tabs-header"><input type="radio" name="ox-tabs-0" id="ox-tab-0-0" checked><label for="ox-tab-0-0">Tab 1</label><input type="radio" name="ox-tabs-0" id="ox-tab-0-1"><label for="ox-tab-0-1">Tab 2</label></div><div class="ox-tab-panel" data-tab="0">alpha</div><div class="ox-tab-panel" data-tab="1">beta</div></div><noscript><div class="ox-tabs-fallback"><details open><summary>Tab 1</summary><div class="ox-tabs-fallback-content">alpha</div></details><details><summary>Tab 2</summary><div class="ox-tabs-fallback-content">beta</div></details></div></noscript></div>"#
    );
}

#[test]
fn uses_label_attribute_when_present() {
    let result = transform_tabs(r#"<tabs><tab label="First">x</tab></tabs>"#, 0);
    assert!(result.html.contains("<label for=\"ox-tab-0-0\">First</label>"));
    assert!(result.html.contains("<summary>First</summary>"));
}

#[test]
fn numbers_groups_from_start_and_counts() {
    let result = transform_tabs(r"<tabs><tab>a</tab></tabs> middle <tabs><tab>b</tab></tabs>", 5);
    assert_eq!(result.group_count, 2);
    assert!(result.html.contains(r#"data-group="5""#));
    assert!(result.html.contains(r#"data-group="6""#));
    assert!(result.html.contains(" middle "));
}

#[test]
fn leaves_empty_tabs_untouched_and_uncounted() {
    let html = r"<tabs>   </tabs>";
    let result = transform_tabs(html, 0);
    assert_eq!(result.group_count, 0);
    assert_eq!(result.html, html);
}

#[test]
fn passes_through_without_tabs_marker() {
    let html = r"<p>No tabs here, just a <table><tr><td>cell</td></tr></table>.</p>";
    let result = transform_tabs(html, 0);
    assert_eq!(result.group_count, 0);
    assert_eq!(result.html, html);
}

#[test]
fn preserves_rich_inner_content() {
    let result =
        transform_tabs(r#"<tabs><tab label="Code"><pre><code>x</code></pre></tab></tabs>"#, 0);
    assert!(result
        .html
        .contains(r#"<div class="ox-tab-panel" data-tab="0"><pre><code>x</code></pre></div>"#));
}
