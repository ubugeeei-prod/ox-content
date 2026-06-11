use std::sync::OnceLock;

use super::{cached_regex, RegexCache};

pub(super) enum ExampleBlock<'a> {
    /// Pure code: a single fenced block (unwrapped, with its language) or a bare
    /// code body (defaulting to `ts`). Rendered inside a code fence / `<pre>`.
    Code { code: &'a str, language: &'a str },
    /// Mixed Markdown (prose and/or fenced code). Rendered as Markdown as-is so it
    /// is not wrapped in an extra code fence.
    Markdown(&'a str),
}

/// True when any line is a code-fence line (opens with ```` ``` ````). Counts fence
/// *lines* only, so a stray ```` ``` ```` inside a single-line string literal is
/// ignored.
fn example_has_fence_line(text: &str) -> bool {
    text.lines().any(|line| line.trim_start().starts_with("```"))
}

/// Classifies an `@example` body. A whole-body single fence is unwrapped to
/// [`ExampleBlock::Code`]; a body that still contains a fence line (prose + code,
/// multiple blocks, …) is kept verbatim as [`ExampleBlock::Markdown`] so it is not
/// double-wrapped; a fence-free body is treated as bare code (`ts`).
pub(super) fn parse_example_block(example: &str) -> ExampleBlock<'_> {
    static FENCE_RE: RegexCache = OnceLock::new();

    let trimmed = example.trim();
    if let Some(fence_re) = cached_regex(&FENCE_RE, r"(?s)^```([\w-]+)?[^\n]*\n(.*?)\n?```$") {
        if let Some(captures) = fence_re.captures(trimmed) {
            let language = captures.get(1).map_or("ts", |value| value.as_str());
            let code = captures.get(2).map_or("", |value| value.as_str());
            // Only a single whole-body fence when the inner code has no further
            // fence line; otherwise the body is multiple blocks → Markdown.
            if !example_has_fence_line(code) {
                return ExampleBlock::Code { code, language };
            }
        }
    }

    if example_has_fence_line(trimmed) {
        ExampleBlock::Markdown(trimmed)
    } else {
        ExampleBlock::Code { code: trimmed, language: "ts" }
    }
}

pub(super) fn render_module_examples_markdown(examples: &[String]) -> String {
    let mut out = String::new();
    out.push_str("## ");
    out.push_str(if examples.len() == 1 { "Example" } else { "Examples" });
    out.push_str("\n\n");
    for example in examples {
        match parse_example_block(example) {
            ExampleBlock::Code { code, language } => {
                out.push_str("```");
                out.push_str(language);
                out.push('\n');
                out.push_str(code);
                out.push_str("\n```\n\n");
            }
            ExampleBlock::Markdown(markdown) => {
                out.push_str(markdown);
                out.push_str("\n\n");
            }
        }
    }
    out
}
