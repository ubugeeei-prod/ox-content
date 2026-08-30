//! Properties the sanitizer must hold for *any* input, checked against
//! randomized fragments built from the pieces that break HTML parsers:
//! stray quotes, half-open comments, control characters, encoded schemes,
//! and tags that never close.
//!
//! A hand-written case list only covers the payloads someone thought of.
//! These four properties are what the sanitizer actually promises.

use ox_content_transform::sanitize::sanitize_html;

/// xorshift64 — deterministic, so a failure reproduces from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const PIECES: &[&str] = &[
    "<a",
    "<img",
    "<script",
    "<svg",
    "<iframe",
    "<p",
    "</a>",
    "</p>",
    ">",
    "/>",
    "/",
    " ",
    "href=",
    "src=",
    "onerror=",
    "onclick=",
    "class=",
    "srcset=",
    "title=",
    "poster=",
    "=",
    "\"",
    "'",
    "javascript:",
    "vbscript:",
    "data:text/html,",
    "https://x/",
    "/a/b",
    "alert(1)",
    "x",
    "<!--",
    "-->",
    "<!",
    "?",
    "&",
    "&amp;",
    "&#106;",
    "&#58;",
    "&#x3a;",
    "&colon;",
    "&lt;",
    "&#0;",
    "\n",
    "\t",
    "<",
    "\u{0}",
    "\u{a0}",
];

const FORBIDDEN_TAGS: [&str; 9] =
    ["script", "svg", "object", "embed", "form", "base", "meta", "link", "style"];

const URL_ATTRS: [&str; 4] = ["href", "src", "poster", "action"];

const DANGEROUS_SCHEMES: [&str; 3] = ["javascript:", "vbscript:", "data:"];

/// The `<name ...>` runs the sanitizer emitted, without their delimiters.
///
/// Everything the sanitizer refused is escaped text, so a `<` that survives
/// in the output really does open a tag the browser will act on.
fn emitted_tags(html: &str) -> Vec<&str> {
    let bytes = html.as_bytes();
    let mut tags = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = html[cursor..].find('<') {
        let start = cursor + relative;
        let Some(end_relative) = html[start..].find('>') else {
            break;
        };
        let end = start + end_relative;
        if bytes.get(start + 1).is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'/') {
            tags.push(&html[start + 1..end]);
        }
        cursor = end + 1;
    }
    tags
}

fn fragment(seed: u64) -> String {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let pieces = 1 + rng.below(14);
    let mut source = String::new();
    for _ in 0..pieces {
        source.push_str(PIECES[rng.below(PIECES.len())]);
    }
    source
}

#[test]
fn no_fragment_survives_sanitizing_with_teeth() {
    let mut failures = Vec::new();
    for seed in 1..40_000u64 {
        let source = fragment(seed);
        let once = sanitize_html(&source, None);

        // 1. Sanitizing is idempotent: an escape the sanitizer emits has to
        //    survive its own next pass, or values drift on every handling.
        let twice = sanitize_html(&once, None);
        if once != twice {
            failures.push(format!(
                "seed {seed}: not idempotent\n  {source:?}\n  {once:?}\n  {twice:?}"
            ));
        }

        for tag in emitted_tags(&once) {
            let lower = tag.to_ascii_lowercase();
            let name: String = lower
                .trim_start_matches('/')
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();

            // 2. No tag outside the allow list reaches the page.
            if FORBIDDEN_TAGS.contains(&name.as_str()) {
                failures.push(format!("seed {seed}: emitted <{name}>\n  {source:?}\n  {once:?}"));
            }

            // 3. No event handler survives, whatever shape it arrived in.
            if lower.contains(" on") {
                failures.push(format!("seed {seed}: event handler\n  {source:?}\n  {tag:?}"));
            }

            // 4. No navigable attribute carries a scripting scheme.
            for attr in URL_ATTRS {
                for scheme in DANGEROUS_SCHEMES {
                    if lower.contains(&(attr.to_string() + "=\"" + scheme)) {
                        failures
                            .push(format!("seed {seed}: {attr}={scheme}\n  {source:?}\n  {tag:?}"));
                    }
                }
            }
        }
        if failures.len() > 8 {
            break;
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
