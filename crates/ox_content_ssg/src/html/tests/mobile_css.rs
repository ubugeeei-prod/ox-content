use super::super::SSG_CSS;

#[test]
fn content_preserves_safe_reading_gutters() {
    assert!(
        SSG_CSS.contains("--octc-mobile-gutter: clamp(1rem, 4vw, 1.25rem);"),
        "mobile layouts need a shared readable gutter token"
    );
    assert!(
        SSG_CSS.contains("--octc-mobile-footer-height: 56px;"),
        "mobile chrome spacing should use one shared footer height token"
    );
    assert!(
        SSG_CSS.contains(
            "padding-left: max(var(--octc-mobile-gutter), env(safe-area-inset-left, 0px));"
        ) && SSG_CSS.contains(
            "padding-right: max(var(--octc-mobile-gutter), env(safe-area-inset-right, 0px));"
        ),
        "content gutters must include each physical display safe area"
    );
    assert!(
        SSG_CSS.contains(".content table {\n  display: block;")
            && SSG_CSS.contains("width: max-content;")
            && SSG_CSS.contains("max-width: 100%;")
            && SSG_CSS.contains("overflow-x: auto;")
            && !SSG_CSS
                .contains("  .content table {\n    display: block;\n    width: max-content;\n    min-width: 100%;"),
        "wide tables must scroll inside the safe content gutter"
    );
    assert!(
        !SSG_CSS.contains("padding: 0.75rem 0.4rem;"),
        "the narrow breakpoint must not collapse back to a 6.4px gutter"
    );
}

#[test]
fn menu_stays_reachable_and_touch_safe() {
    assert!(
        SSG_CSS.contains("body.menu-open {\n  overflow: hidden;\n  overscroll-behavior: none;"),
        "an open drawer must not scroll the page behind it"
    );
    assert!(
        SSG_CSS.contains(
            ".sidebar {\n    position: fixed;\n    top: calc(var(--octc-header-height) + 0.75rem);"
        )
            && SSG_CSS.contains(
                "left: max(var(--octc-mobile-gutter), env(safe-area-inset-left, 0px));"
            )
            && SSG_CSS.contains(
                "right: max(var(--octc-mobile-gutter), env(safe-area-inset-right, 0px));"
            )
            && SSG_CSS.contains(
                "bottom: calc(var(--octc-mobile-footer-height) + env(safe-area-inset-bottom, 0px) + 0.75rem);"
            )
            && SSG_CSS.contains("width: auto;")
            && SSG_CSS.contains("max-height: calc(")
            && SSG_CSS.contains("visibility: hidden;\n    pointer-events: none;")
            && SSG_CSS.contains("overflow-y: auto;\n    overflow-x: hidden;")
            && SSG_CSS.contains("overscroll-behavior-y: contain;")
            && SSG_CSS.contains("scrollbar-gutter: stable;")
            && SSG_CSS.contains("contain: paint;"),
        "the mobile drawer must stay inset between header and footer with internal scrolling"
    );
    assert!(
        SSG_CSS.contains(".sidebar.open {\n    transform: translateY(0);")
            && SSG_CSS.contains("visibility: visible;\n    pointer-events: auto;"),
        "only the visible mobile drawer should receive pointer events"
    );
    assert!(
        SSG_CSS.contains(".sidebar::before {\n    display: none;"),
        "the mobile drawer should not rely on a fake sticky bar above long navigation"
    );
    assert!(
        SSG_CSS.contains(".overlay {\n    display: none;\n    position: fixed;")
            && SSG_CSS
                .contains("background: color-mix(in srgb, var(--octc-color-bg) 72%, transparent);"),
        "the menu overlay must cover the reading pane instead of staying transparent"
    );
    assert!(
        SSG_CSS.contains(".sidebar .nav-link {\n    min-height: 44px;")
            && SSG_CSS.contains(".sidebar .nav-title--summary {\n    min-height: 44px;"),
        "every mobile navigation control needs a reliable touch target"
    );
    assert!(
        SSG_CSS
            .contains("@media (any-hover: hover) and (any-pointer: fine) {\n  .nav-link:hover {")
            && SSG_CSS.contains(
                "@media (any-hover: hover) and (any-pointer: fine) {\n  .mobile-footer-btn:hover {"
            ),
        "touch input must not retain mouse-only hover treatments"
    );
    assert!(
        SSG_CSS.contains(
            "@media (any-hover: none) and (any-pointer: coarse) {\n  .mobile-footer-btn:focus {\n    outline: none;"
        ),
        "touch-only focus treatment must be selected with input media queries"
    );
}
