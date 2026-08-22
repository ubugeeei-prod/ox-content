//! Turns a tree-sitter highlight event stream into HTML.
//!
//! The markup matches what the renderer emitted before: a `<pre class="shiki
//! css-variables">` carrying the background and foreground colors, a `<code>`,
//! and one `<span class="line">` per line whose runs are `<span
//! style="color:var(...)">`. Downstream CSS, the code-annotation transforms and
//! the copy button all key off that shape.

use tree_sitter_highlight::HighlightEvent;

use crate::theme::{self, BACKGROUND, FOREGROUND, Token};

/// Writes `text` with the five HTML-significant bytes escaped.
fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}

/// Emits one styled run, splitting it across lines so every line stays a
/// self-contained `<span class="line">`.
struct Writer {
    out: String,
    open_line: bool,
}

impl Writer {
    fn new(capacity: usize) -> Self {
        let mut out = String::with_capacity(capacity);
        out.push_str(
            "<pre class=\"shiki css-variables\" style=\"background-color:var(--octc-shiki-",
        );
        out.push_str(BACKGROUND.name);
        out.push_str(", ");
        out.push_str(BACKGROUND.fallback);
        out.push_str(");");
        theme::push_color(&mut out, FOREGROUND);
        out.push_str("\" tabindex=\"0\"><code>");
        Self { out, open_line: false }
    }

    fn open_line(&mut self) {
        if !self.open_line {
            self.out.push_str("<span class=\"line\">");
            self.open_line = true;
        }
    }

    fn end_line(&mut self) {
        self.open_line();
        self.out.push_str("</span>\n");
        self.open_line = false;
    }

    /// Writes `text` under `token`, breaking runs at newlines.
    fn push_run(&mut self, text: &str, token: Option<Token>) {
        for (index, piece) in text.split('\n').enumerate() {
            if index > 0 {
                self.end_line();
            }
            if piece.is_empty() {
                continue;
            }
            self.open_line();
            self.out.push_str("<span style=\"");
            theme::push_color(&mut self.out, token.unwrap_or(FOREGROUND));
            self.out.push_str("\">");
            push_escaped(&mut self.out, piece);
            self.out.push_str("</span>");
        }
    }

    fn finish(mut self) -> String {
        // A trailing `<span class="line"></span>` is what the previous
        // highlighter emitted for the final newline, and the CSS line-number
        // counter is written against it.
        self.open_line();
        self.out.push_str("</span></code></pre>");
        self.out
    }
}

/// Renders `code` given the events tree-sitter produced for it.
///
/// `capture_name` resolves a highlight index back to the capture name the
/// theme maps, which the caller holds because it owns the configuration.
pub fn render<'a, I, F>(code: &str, events: I, capture_name: F) -> Option<String>
where
    I: Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>>,
    F: Fn(usize) -> Option<&'a str>,
{
    let mut writer = Writer::new(code.len() * 4);
    let mut stack: Vec<Option<Token>> = Vec::new();

    for event in events {
        match event.ok()? {
            HighlightEvent::HighlightStart(highlight) => {
                stack.push(capture_name(highlight.0).and_then(theme::token_for));
            }
            HighlightEvent::HighlightEnd => {
                stack.pop();
            }
            HighlightEvent::Source { start, end } => {
                let text = code.get(start..end)?;
                writer.push_run(text, stack.last().copied().flatten());
            }
        }
    }

    Some(writer.finish())
}
