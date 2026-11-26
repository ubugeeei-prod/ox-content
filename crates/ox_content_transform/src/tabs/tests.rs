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
    insta::assert_snapshot!(result.html);
}

#[test]
fn numbers_groups_from_start_and_counts() {
    let result = transform_tabs(r"<tabs><tab>a</tab></tabs> middle <tabs><tab>b</tab></tabs>", 5);
    assert_eq!(result.group_count, 2);
    insta::assert_snapshot!(result.html);
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
    insta::assert_snapshot!(result.html);
}

#[test]
fn settled_blocks_agree_with_searching_afresh() {
    const PIECES: [&str; 17] = [
        "<tabs>",
        "</tabs>",
        "<TABS>",
        "<tabs/>",
        "<tab label=\"a\">",
        "<tab>",
        "<tab/>",
        "</tab>",
        "</TAB>",
        "<tabs a=\"",
        "\"",
        "'",
        ">",
        "<tabset>",
        "<table>",
        "x",
        " ",
    ];
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for round in 0..5000 {
        let pieces = 1 + round % 40;
        let mut html = String::new();
        for _ in 0..pieces {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            html.push_str(PIECES[(state % PIECES.len() as u64) as usize]);
        }
        let remembered = super::transform_tabs_with(&html, 0, true);
        let afresh = super::transform_tabs_with(&html, 0, false);
        assert_eq!(remembered.html, afresh.html, "for {html:?}");
        assert_eq!(remembered.group_count, afresh.group_count, "for {html:?}");
    }
}

#[test]
fn nested_and_unclosed_blocks_cost_linear_time() {
    // Measured before: 64 KiB of the third shape took 118 s, growing x70 for
    // every x4 of input; the others grew x10 to x20.
    let time = |html: &str| {
        (0..3)
            .map(|_| {
                let started = std::time::Instant::now();
                std::hint::black_box(transform_tabs(html, 0).html.len());
                started.elapsed()
            })
            .min()
            .unwrap_or(std::time::Duration::MAX)
    };
    for (unit, suffix) in [
        ("<tabs>", ""),
        ("<tabs ", ""),
        ("<tabs><tab label=\"a\">x</tab>", "</tabs>"),
        ("<tabs><tab label=\"a\">", ""),
    ] {
        let build = |bytes: usize| format!("{}{suffix}", unit.repeat(bytes / unit.len()));
        let (small, large) = (build(16 * 1024), build(64 * 1024));
        let (small_time, large_time) = (time(&small), time(&large));
        // 4x the input: linear is about x4, quadratic about x16.
        assert!(
            large_time < small_time * 8 + std::time::Duration::from_millis(20),
            "{unit:?}: 64 KiB took {large_time:?} against {small_time:?} for 16 KiB"
        );
    }
}
