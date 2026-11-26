//! How deep inline content may nest.
//!
//! Image alt text, strikethrough, sub/superscript and JSX children are each
//! parsed by calling back into `parse_inline`, and emphasis nests without
//! any call at all. Neither was bounded: a few kilobytes of `![` or `*`
//! built a call stack, or a tree, tens of thousands of levels deep — and
//! every later pass over that tree recurses once per level.
//!
//! Two counters keep both inside [`ParserOptions::max_nesting_depth`]. The
//! depth is checked on the way in, before the stack grows. The height is
//! reported on the way out, because how deep emphasis nests around a child
//! is only known once the whole sequence has been paired.
//!
//! [`ParserOptions::max_nesting_depth`]: crate::ParserOptions::max_nesting_depth

use ox_content_ast::Span;

use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser<'_> {
    /// The configured cap, with `0` (unlimited) mapped out of reach.
    pub(in crate::parser) fn inline_nesting_cap(&self) -> u32 {
        match u32::try_from(self.options.max_nesting_depth) {
            Ok(0) | Err(_) => u32::MAX,
            Ok(cap) => cap,
        }
    }

    pub(in crate::parser) fn nesting_too_deep(&self, span: Span) -> ParseError {
        ParseError::NestingTooDeep { span, max_depth: self.options.max_nesting_depth }
    }

    /// Enters one `parse_inline` call that may nest further.
    ///
    /// Returns the height its caller had collected so far, which
    /// [`Self::leave_inline`] hands back.
    pub(super) fn enter_inline(&self, span: Span) -> ParseResult<u32> {
        let depth = self.inline_depth.get() + 1;
        if depth > self.inline_nesting_cap() {
            return Err(self.nesting_too_deep(span));
        }
        self.inline_depth.set(depth);
        Ok(self.inline_height.take())
    }

    /// Leaves the call entered by [`Self::enter_inline`], reporting the
    /// height of what it built to whichever construct wraps it.
    pub(super) fn leave_inline(
        &self,
        collected: u32,
        height: ParseResult<u32>,
        span: Span,
    ) -> ParseResult<()> {
        self.inline_depth.set(self.inline_depth.get().saturating_sub(1));
        let height = height?;
        if height > self.inline_nesting_cap() {
            return Err(self.nesting_too_deep(span));
        }
        self.inline_height.set(collected.max(height.saturating_add(1)));
        Ok(())
    }

    /// Takes the height reported by the calls made since the last take.
    pub(super) fn take_inline_height(&self) -> u32 {
        self.inline_height.take()
    }
}
