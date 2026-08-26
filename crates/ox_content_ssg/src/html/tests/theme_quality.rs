use super::super::{ENTRY_CSS, SSG_CSS, header_chrome::HEADER_CHROME_CSS};

const QUALITY_TOKENS: &[&str] = &[
    "--octc-color-on-primary:",
    "--octc-color-tip:",
    "--octc-color-info:",
    "--octc-color-warning:",
    "--octc-color-danger:",
    "--octc-color-success:",
    "--octc-focus-ring:",
    "--octc-focus-offset:",
    "--octc-motion-base:",
    "--octc-motion-ease:",
    "--octc-code-pad-block:",
    "--octc-code-pad-inline:",
    "--octc-table-cell-pad-block:",
    "--octc-table-cell-pad-inline:",
    "--octc-touch-target:",
];

#[test]
fn default_theme_exposes_quality_tokens() {
    for token in QUALITY_TOKENS {
        assert!(SSG_CSS.contains(token), "missing quality token {token}");
    }
    assert!(
        !SSG_CSS.contains("--octc-callout-accent: #")
            && !SSG_CSS.contains("--octc-badge-accent: #")
            && !SSG_CSS.contains("--octc-container-accent: #"),
        "status chrome must use semantic tokens instead of hex escapes"
    );
}

#[test]
fn default_theme_focus_and_reduced_motion_are_tokenized() {
    assert!(
        SSG_CSS.contains(":focus-visible {\n  outline: var(--octc-focus-ring);")
            && SSG_CSS.contains("outline-offset: var(--octc-focus-offset);"),
        "keyboard focus must share one tokenized ring"
    );
    assert!(
        SSG_CSS.contains("@media (prefers-reduced-motion: no-preference)")
            && SSG_CSS.contains("scroll-behavior: smooth;"),
        "smooth scrolling must opt in only when motion is allowed"
    );
    assert!(
        SSG_CSS.contains("@media (prefers-reduced-motion: reduce)")
            && SSG_CSS.contains("transition: none;")
            && SSG_CSS.contains("filter: none;"),
        "reduced motion must disable nav motion and code-line blur"
    );
    assert!(
        SSG_CSS.contains("transition: transform var(--octc-motion-base) var(--octc-motion-ease);"),
        "nav disclosure motion must use the shared motion tokens"
    );
}

#[test]
fn code_table_and_mobile_nav_share_spacing_tokens() {
    assert!(
        SSG_CSS.contains("padding: var(--octc-code-pad-block) var(--octc-code-pad-inline);")
            && SSG_CSS
                .contains("padding: var(--octc-code-title-pad-block) var(--octc-code-pad-inline);")
            && SSG_CSS.contains("left: var(--octc-code-pad-inline);"),
        "code frames, titles, and line gutters must stay on one pad contract"
    );
    assert!(
        SSG_CSS.contains(
            "padding: var(--octc-table-cell-pad-block) var(--octc-table-cell-pad-inline);"
        ),
        "table cells must use the shared cell-pad tokens"
    );
    assert!(
        SSG_CSS.contains("min-height: var(--octc-touch-target);"),
        "mobile nav targets must use the shared touch token"
    );
    assert!(
        !SSG_CSS.contains("padding: 0.85rem 0.95rem;")
            && !SSG_CSS.contains("padding: 0.78rem 0.85rem;")
            && !SSG_CSS.contains("padding: 0.375rem 0.5rem;"),
        "mobile breakpoints must retune tokens instead of restating pad literals"
    );
}

#[test]
fn code_block_affordances_keep_wrapping_and_targets_stable() {
    assert!(
        SSG_CSS.contains(".content pre.ox-code-block--wrap {\n  overflow-x: hidden;\n  white-space: pre-wrap;")
            && SSG_CSS.contains(".content pre.ox-code-block--wrap code,\n.content pre.ox-code-block--wrap .line {\n  white-space: inherit;\n  overflow-wrap: anywhere;")
            && SSG_CSS.contains(".content pre.ox-code-block--nowrap {\n  overflow-x: auto;\n  white-space: pre;"),
        "code wrap modes must choose either wrapping or horizontal scrolling explicitly"
    );
    assert!(
        SSG_CSS.contains(".content pre.ox-code-block .ox-code-line:target {")
            && SSG_CSS.contains("background: var(--octc-color-code-line-focus);")
            && SSG_CSS.contains("outline-offset: -1px;"),
        "line-link targets must reuse code focus tokens without shifting layout"
    );
}

#[test]
fn default_theme_replaces_on_primary_hex_escapes() {
    assert!(
        ENTRY_CSS.contains("color: var(--octc-color-on-primary);")
            && !ENTRY_CSS.contains("color: #ffffff")
            && !ENTRY_CSS.contains("color: #fff"),
        "entry chrome must put on-primary ink on brand fills"
    );
    assert!(
        HEADER_CHROME_CSS.contains("color: var(--octc-color-on-primary);")
            && !HEADER_CHROME_CSS.contains("color: #fff"),
        "announcement chrome must use on-primary instead of hardcoded white"
    );
}

#[test]
fn default_theme_css_payload_stays_visible() {
    let bytes = SSG_CSS.len();
    assert!(bytes > 40_000, "default stylesheet is unexpectedly small: {bytes}");
    assert!(bytes < 90_000, "default stylesheet grew past the visible budget: {bytes}");
}
