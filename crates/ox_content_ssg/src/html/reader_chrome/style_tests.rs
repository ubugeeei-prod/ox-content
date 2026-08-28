use super::{READER_CHROME_CSS, READER_CHROME_JS};

#[test]
fn copy_control_uses_inline_clearance_without_wasting_vertical_space() {
    assert!(
        READER_CHROME_CSS.contains("inset-block-start: var(--ox-copy-inset, 0.5rem);")
            && READER_CHROME_CSS.contains("inset-inline-end: var(--ox-copy-inset, 0.5rem);"),
        "copy positioning must follow the document writing direction: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(
            ".content .ox-code > pre,\n.content .ox-code:has(ox-code-play) pre {\n  margin: 0;\n  padding-inline-end: calc(var(--ox-copy-reserved-inline-size) + 0.45rem);"
        ),
        "code only needs enough inline clearance for the icon: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(
            ".ox-code:has(> ox-code-play) .ox-code-play__toolbar {\n  padding-inline-end: calc(var(--ox-copy-reserved-inline-size) + 0.65rem);"
        ),
        "Code Play Run must sit left of the copy icon: {READER_CHROME_CSS}"
    );
    assert!(
        !READER_CHROME_CSS.contains("padding-block-start: 3rem"),
        "copy must not push the first code line down: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(
            ".content .ox-code > pre[data-code-title]::before {\n  margin-inline-end: calc(-1 * (var(--ox-copy-reserved-inline-size) + 0.45rem));\n  padding-inline-end: calc(var(--ox-copy-reserved-inline-size) + 0.45rem);"
        ),
        "a code title must keep its header fill aligned behind the fixed-size icon: {READER_CHROME_CSS}"
    );
}

#[test]
fn copy_control_uses_capability_media_queries_and_stable_status_ui() {
    assert!(
        READER_CHROME_CSS.contains("(any-hover: hover) and (any-pointer: fine)"),
        "hover reveal must be restricted to hover-capable devices: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(".ox-code:focus-within > .ox-copy"),
        "keyboard focus must reveal the control: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(".ox-copy::before")
            && READER_CHROME_CSS.contains(".ox-copy-status"),
        "the control needs a visible icon and a non-visual status region: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains(
            ".ox-code > .ox-copy {\n  z-index: 2;\n  min-width: 0;\n  min-height: 0;\n  width: var(--ox-copy-control-size, 1.75rem);\n  height: var(--ox-copy-control-size, 1.75rem);\n  padding: 0;"
        ),
        "custom prose button styles must not resize the token-controlled copy control: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_CSS.contains("border: 1px solid transparent;")
            && READER_CHROME_CSS.contains("background: transparent;")
            && READER_CHROME_CSS.contains(".ox-code:focus-within > .ox-copy {\n    opacity: 0.86;")
            && READER_CHROME_CSS
                .contains("background: color-mix(in srgb, var(--octc-color-code-bg) 82%"),
        "copy should stay nearly icon-only until hover/focus: {READER_CHROME_CSS}"
    );
    assert!(
        READER_CHROME_JS.contains("data-ox-copy-status")
            && READER_CHROME_JS.contains("Copy failed")
            && READER_CHROME_JS.contains(r#"getAttribute("data-ox-code-source")"#)
            && !READER_CHROME_JS.contains("textContent = \"Copy"),
        "copy feedback must stay fixed-size and prefer raw code source when available: {READER_CHROME_JS}"
    );
    assert!(
        READER_CHROME_JS.contains("WeakSet")
            && READER_CHROME_JS.contains("function initReaderChrome(rootInput)"),
        "custom hosts need an idempotent root initializer: {READER_CHROME_JS}"
    );
}
