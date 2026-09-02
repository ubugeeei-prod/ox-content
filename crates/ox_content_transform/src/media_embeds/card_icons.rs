//! Inline SVG marks for provider cards.
//!
//! A registry card is mostly a logo and a row of counts, and spelling each of
//! those out — `Version 7.1.0  License MIT  Downloads 31M/week` — turned the
//! card into a wall of labels with the numbers buried in them. The marks are
//! hand-authored and monochrome so a card stays offline and inherits the
//! card's colour: the Iconify integration is opt-in and reads collections from
//! disk, which a card in the default build cannot assume is installed.
//!
//! Every mark is decorative (`aria-hidden`); the label it replaces stays in
//! the markup as visually hidden text, so a screen reader still hears
//! "Downloads 31M/week".

/// The registry's own mark, keyed by the card modifier.
///
/// Only the package registries have one. Every other provider keeps its name
/// as text: a card that shows an unfamiliar glyph instead of "Observable" is
/// harder to read, not easier.
pub(super) fn network_logo(modifier: &str) -> Option<&'static str> {
    Some(match modifier {
        "npm" => NPM,
        "crates-io" => CRATES_IO,
        "pypi" => PYPI,
        "docker-hub" => DOCKER,
        _ => return None,
    })
}

/// The mark for a metric label, when one reads faster than the word.
///
/// Registry labels are covered as a set, so a package card is all icons rather
/// than a mix. Labels outside it — `Tags`, `Likes`, `Instance` — keep their
/// text.
pub(super) fn metric_icon(label: &str) -> Option<&'static str> {
    Some(match label {
        "Version" => TAG,
        "License" => LICENSE,
        "Repository" => REPOSITORY,
        "Downloads" => DOWNLOADS,
        "Stars" => STARS,
        _ => return None,
    })
}

/// A registry mark: filled paths on the card's own colour.
macro_rules! logo {
    ($($body:literal),+ $(,)?) => {
        concat!(
            r#"<svg class="ox-provider-card__logo" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">"#,
            $($body),+,
            "</svg>",
        )
    };
}

/// A metric mark: one stroked line drawing, sized to sit on the meta line.
macro_rules! metric {
    ($($body:literal),+ $(,)?) => {
        concat!(
            r#"<svg class="ox-provider-card__metric-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">"#,
            $($body),+,
            "</svg>",
        )
    };
}

/// The wordmark bar, on its own box because it is far wider than it is tall.
/// The letters are holes, so they take the card's background the way the white
/// letters take the red square.
const NPM: &str = concat!(
    r#"<svg class="ox-provider-card__logo ox-provider-card__logo--wide" viewBox="0 0 24 10" width="22" height="9" aria-hidden="true" focusable="false">"#,
    r#"<path fill="currentColor" fill-rule="evenodd" d="M0 0h24v10H0z"#,
    r#"M1.5 1.5h5v7H5v-5H3v5H1.5z"#,
    r#"M9 1.5h5.5v7H13v-5h-2.5V10H9z"#,
    r#"M16.5 1.5h6v7H21v-5h-1v5h-1.5v-5h-1v5h-1z"/>"#,
    "</svg>",
);

/// The crate itself, which is what crates.io puts on its own pages.
const CRATES_IO: &str = logo!(
    r#"<path fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round""#,
    r#" d="m12 2.4 8.6 4.3v10.6L12 21.6 3.4 17.3V6.7z"/>"#,
    r#"<path fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round""#,
    r#" d="m3.4 6.7 8.6 4.3 8.6-4.3M12 11v10.6"/>"#,
);

const PYPI: &str = logo!(
    r#"<path fill="currentColor" d="M11.9 2c-1.9 0-3.6.3-3.6 2.4v1.5h3.7v.6H6.6S4 6.2 4 10s2.3 3.7 2.3 3.7h1.4v-1.8s-.1-2.3 2.2-2.3h3.7s2.2.1 2.2-2.1V4.5S16.1 2 11.9 2M9.8 3.2a.7.7 0 1 1 0 1.4.7.7 0 0 1 0-1.4"/>"#,
    r#"<path fill="currentColor" d="M12.1 22c1.9 0 3.6-.3 3.6-2.4v-1.5H12v-.6h5.4s2.6.1 2.6-3.7-2.3-3.7-2.3-3.7h-1.4v1.8s.1 2.3-2.2 2.3h-3.7s-2.2-.1-2.2 2.1v3.2S7.9 22 12.1 22m2.1-1.2a.7.7 0 1 1 0-1.4.7.7 0 0 1 0 1.4"/>"#,
);

/// Containers on a hull: the two halves of the whale that survive at 16px.
const DOCKER: &str = logo!(
    r#"<path fill="currentColor" d="M4.3 10.6h2.9v2.9H4.3zm3.6 0h2.9v2.9H7.9zm3.6 0h2.9v2.9h-2.9zm3.6 0H18v2.9h-2.9zM7.9 7h2.9v2.9H7.9zm3.6 0h2.9v2.9h-2.9zm0-3.6h2.9v2.9h-2.9z"/>"#,
    r#"<path fill="currentColor" d="M23 11.4c-.6-.4-1.9-.6-2.9-.4-.1-1-.7-1.8-1.6-2.6l-.5-.4-.4.5c-.5.8-.7 2-.6 2.9H2.1l-.1.5c-.2 1.9.3 4.3 1.8 6 1.4 1.6 3.6 2.4 6.4 2.4 6.2 0 10.7-2.9 12.8-8 .8 0 2.6 0 3.5-1.7z"/>"#,
);

const TAG: &str = metric!(
    r#"<path d="M3.5 11.3V4.4a.9.9 0 0 1 .9-.9h6.9c.24 0 .47.1.64.26l8 8a.9.9 0 0 1 0 1.28l-6.9 6.9a.9.9 0 0 1-1.28 0l-8-8a.9.9 0 0 1-.26-.64Z"/>"#,
    r#"<path d="M7.6 7.6h.01"/>"#,
);

/// A balance scale: the mark a licence carries everywhere else in developer
/// tooling, so it needs no legend.
const LICENSE: &str = metric!(
    r#"<path d="M12 3.2v17.6M5.5 5.9h13M8.4 20.8h7.2"/>"#,
    r#"<path d="M5.5 5.9 2.6 12.5h5.8zM18.5 5.9l-2.9 6.6h5.8z"/>"#,
    r#"<path d="M2.6 12.5a2.9 2.9 0 0 0 5.8 0M15.6 12.5a2.9 2.9 0 0 0 5.8 0"/>"#,
);

const REPOSITORY: &str = metric!(
    r#"<circle cx="6.6" cy="5.3" r="2.3"/><circle cx="6.6" cy="18.7" r="2.3"/>"#,
    r#"<circle cx="17.4" cy="8.4" r="2.3"/>"#,
    r#"<path d="M6.6 7.6v8.8M17.4 10.7c0 3-2.4 4.4-5.3 5"/>"#,
);

const DOWNLOADS: &str = metric!(
    r#"<path d="M12 3.4v11.2M7.6 10.2l4.4 4.4 4.4-4.4M3.9 18.1v1.6a.9.9 0 0 0 .9.9h14.4a.9.9 0 0 0 .9-.9v-1.6"/>"#,
);

const STARS: &str = metric!(
    r#"<path d="m12 3.1 2.75 5.57 6.15.9-4.45 4.33 1.05 6.12L12 17.13l-5.5 2.89 1.05-6.12L3.1 9.57l6.15-.9z"/>"#,
);
