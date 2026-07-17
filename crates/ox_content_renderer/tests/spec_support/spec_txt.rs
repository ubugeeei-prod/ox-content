//! Loader for CommonMark-style `spec.txt` fixtures.
//!
//! The spec format stores each conformance case as a fenced block:
//!
//! ```text
//! `````````````````````````````````` example
//! markdown input
//! .
//! expected html
//! ``````````````````````````````````
//! ```
//!
//! Tabs are encoded as `→` inside examples so they survive editors that
//! expand tabs; the loader converts them back before returning.

/// One conformance example extracted from a spec fixture.
pub struct SpecExample {
    /// 1-based example number, matching the official spec numbering.
    pub number: usize,
    /// Title of the closest enclosing ATX section heading.
    pub section: String,
    /// Markdown input fed to the parser.
    pub markdown: String,
    /// Expected HTML output as printed in the spec.
    pub html: String,
}

const FENCE: &str = "````````````````````````````````";

/// Parses every example block out of a spec fixture.
pub fn parse_spec(text: &str) -> Vec<SpecExample> {
    let mut examples = Vec::new();
    let mut section = String::new();
    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        if let Some(heading) = line.strip_prefix('#') {
            section = heading.trim_start_matches('#').trim().to_string();
            continue;
        }

        let is_example_fence = line
            .strip_prefix(FENCE)
            .is_some_and(|rest| rest.trim() == "example" || rest.trim().starts_with("example "));
        if !is_example_fence {
            continue;
        }

        let mut markdown = String::new();
        let mut html = String::new();
        let mut in_html = false;
        for body_line in lines.by_ref() {
            if body_line.starts_with(FENCE) {
                break;
            }
            if !in_html && body_line == "." {
                in_html = true;
                continue;
            }
            let target = if in_html { &mut html } else { &mut markdown };
            target.push_str(&body_line.replace('→', "\t"));
            target.push('\n');
        }

        examples.push(SpecExample {
            number: examples.len() + 1,
            section: section.clone(),
            markdown,
            html,
        });
    }

    examples
}

#[test]
fn parses_examples_and_sections() {
    let spec = format!(
        "# One\n\n{FENCE} example\na→b\n.\n<p>a\tb</p>\n{FENCE}\n\n\
         ## Two\n\n{FENCE} example table\n| a |\n.\n<table></table>\n{FENCE}\n"
    );
    let examples = parse_spec(&spec);
    assert_eq!(examples.len(), 2);
    assert_eq!(examples[0].number, 1);
    assert_eq!(examples[0].section, "One");
    assert_eq!(examples[0].markdown, "a\tb\n");
    assert_eq!(examples[0].html, "<p>a\tb</p>\n");
    assert_eq!(examples[1].number, 2);
    assert_eq!(examples[1].section, "Two");
    assert_eq!(examples[1].markdown, "| a |\n");
}
