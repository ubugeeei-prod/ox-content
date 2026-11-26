//! Finding the term lines that open a definition-list item.
//!
//! An item is a run of term lines followed by a `: body` line, with blank
//! lines allowed between the two. Whether an item starts on a line is
//! therefore decided by where the run holding that line stops, and every
//! line of one run shares that stop — so a run is walked once, however
//! many of its lines get asked.

use super::Parser;
use super::lines::has_unclosed_inline_code;

/// A run of term lines that a probe has already walked.
#[derive(Clone, Copy)]
pub(in crate::parser) struct TermRun {
    /// Where the walk started.
    from: usize,
    /// Start of the first line after `from` that is not a term line, or the
    /// end of the source.
    end: usize,
    /// Start of the body line that makes the run an item: the line at `end`,
    /// or the one after the blank lines there. `None` when the run is
    /// ordinary text.
    body: Option<usize>,
}

impl Parser<'_> {
    /// Returns where the term lines starting at `start` end and where the
    /// body after them starts, or `None` when no definition item starts
    /// here.
    ///
    /// This is a probe: block dispatch asks it at every block start and a
    /// definition body asks it for every one of its lines. It allocates
    /// nothing, because the arena never gives memory back.
    pub(super) fn definition_terms_at(&self, start: usize) -> Option<(usize, usize)> {
        if !self.is_definition_term_at(start) {
            return None;
        }
        let run = match self.definition_term_run.get() {
            // The walk from `run.from` stepped through whole lines and found
            // nothing but terms before `run.end`. `start` is a term inside
            // that stretch, so the line after it is one the walk stood on
            // (or `run.end` itself), and walking again would stop at the
            // same line and reach the same verdict.
            Some(run) if (run.from..run.end).contains(&start) => run,
            _ => self.walk_definition_term_run(start),
        };
        run.body.map(|body| (run.end, body))
    }

    /// Walks the run of term lines whose first line is at `start` and
    /// remembers where it stopped.
    fn walk_definition_term_run(&self, start: usize) -> TermRun {
        let mut end = self.next_line_start(start);
        while self.is_definition_term_at(end) {
            end = self.next_line_start(end);
        }
        // The run stops at a body line, at blank lines, at the start of some
        // other block, or at the end of the source. Only a body makes it an
        // item, and the body may sit past the blank lines. Skipping blanks
        // first covers both: it moves nowhere from the other stops, and of
        // those only the body line passes the test.
        let after_blanks = self.skip_blank_lines_from(end);
        let body = self.starts_definition_body_at(after_blanks).then_some(after_blanks);
        let run = TermRun { from: start, end, body };
        self.definition_term_run.set(Some(run));
        run
    }

    /// `(start, end)` of each term line in `start..terms_end`, terminators
    /// excluded.
    pub(super) fn definition_term_lines(
        &self,
        start: usize,
        terms_end: usize,
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        let mut cursor = start;
        std::iter::from_fn(move || {
            (cursor < terms_end).then(|| {
                let line = (cursor, self.line_end_at(cursor));
                cursor = self.next_line_start(cursor);
                line
            })
        })
    }

    /// Whether the line at `line_start` can be a term. A blank line ends a
    /// run of terms, and so does a body line, which never passes the term
    /// test because it starts with `:`.
    fn is_definition_term_at(&self, line_start: usize) -> bool {
        line_start < self.source.len()
            && !self.is_blank_line_at(line_start)
            && self.is_definition_term_line(line_start)
    }

    fn is_definition_term_line(&self, line_start: usize) -> bool {
        let Some(trimmed_start) = self.first_non_whitespace_in_line(line_start) else {
            return false;
        };
        if self.line_indent_width(line_start, trimmed_start) >= 4 {
            return false;
        }
        let line = self.line_at(line_start);
        let trimmed = &line[trimmed_start - line_start..];
        !trimmed.starts_with(':')
            && !trimmed.starts_with('>')
            && !trimmed.starts_with(":::")
            && !trimmed.starts_with('<')
            && !Self::try_parse_thematic_break_line(line)
            && !self.try_parse_heading_start(line_start, trimmed_start)
            && !Self::try_parse_fenced_code_at(line, trimmed)
            // Accepts exactly the lines the item parser does, without
            // building the item: that one copies the content of an item
            // whose marker is followed by a tab into the arena.
            && !Self::try_parse_list_line(trimmed)
            && !has_unclosed_inline_code(line)
    }
}

#[cfg(test)]
mod tests {
    use ox_content_allocator::Allocator;

    use super::Parser;

    #[test]
    fn the_list_line_test_agrees_with_the_item_parser() {
        // A term line is one the item parser would turn down, so the cheap
        // test has to draw the line in the same place.
        let allocator = Allocator::new();
        for line in [
            "- a",
            "-\ta",
            "-\t\ta",
            "-",
            "- ",
            "-a",
            "--",
            "* a",
            "*a",
            "+",
            "+\ta",
            "1. a",
            "1.\ta",
            "1.",
            "1) ",
            "1.a",
            "2. a",
            "123456789. a",
            "1234567890. a",
            "1",
            "1 . a",
            "a",
            "",
        ] {
            let parser = Parser::new(&allocator, line);
            assert_eq!(
                Parser::try_parse_list_line(line),
                parser.parse_list_item_line_from_trimmed(0, line, line).is_some(),
                "for {line:?}"
            );
        }
    }
}
