//! Source span utilities for tracking positions in source text.

/// A span representing a range in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// Start offset (inclusive), zero-indexed.
    pub start: u32,
    /// End offset (exclusive), zero-indexed.
    pub end: u32,
}

impl Span {
    /// Creates a new span.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns an empty span at position 0.
    #[must_use]
    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Returns the length of the span.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merges two spans into a single span covering both.
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        let start = if self.start < other.start { self.start } else { other.start };
        let end = if self.end > other.end { self.end } else { other.end };
        Self { start, end }
    }

    /// Returns true if this span contains the given offset.
    #[must_use]
    pub const fn contains(&self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Returns true if this span contains the given span.
    #[must_use]
    pub const fn contains_span(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Extracts the source text for this span.
    #[must_use]
    pub fn source_text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}

/// Position in source text with line and column information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number.
    pub column: u32,
    /// 0-indexed byte offset.
    pub offset: u32,
}

impl Position {
    /// Creates a new position.
    #[must_use]
    pub const fn new(line: u32, column: u32, offset: u32) -> Self {
        Self { line, column, offset }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 10);
        let span2 = Span::new(5, 20);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);
        assert_eq!(
            [span.contains(10), span.contains(15), span.contains(20), span.contains(5)],
            [true, true, false, false]
        );
    }

    #[test]
    fn test_source_text() {
        let source = "hello world";
        let span = Span::new(0, 5);
        assert_eq!(span.source_text(source), "hello");
    }
}
