//! Thread-local scratch state for the synchronous parse/render entry points.
//!
//! A `parseAndRender` call on a small document spends a surprising share of its
//! time in setup rather than in Markdown work: building a fresh arena,
//! constructing an [`HtmlRenderer`] (whose options own six heap strings), and
//! growing an output buffer up from zero. Those costs are per call, so they hurt
//! exactly where a throughput table hides them — short documents, which is what
//! a per-request or per-file caller actually feeds us.
//!
//! Both pieces are reusable by design: bumpalo can [`Allocator::reset`] back to
//! the start of its largest chunk, and `HtmlRenderer` already documents that
//! reusing an instance avoids hot-path allocations. Keeping one of each per
//! thread turns that setup into a pointer reset after the first call.
//!
//! Reuse is safe here because every helper below returns owned data (a `String`,
//! a `Vec<u8>`): nothing that borrows the arena survives the call to observe the
//! next reset. The state is thread-local, so Node worker threads and the async
//! task pool each get their own and never share one.

use std::cell::RefCell;

use ox_content_allocator::Allocator;
use ox_content_mdast::{mdast, mdast_raw};
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

/// Largest arena capacity carried over to the next call.
///
/// `reset` keeps the arena's largest chunk, which is what makes reuse free —
/// but it also pins that chunk for the lifetime of the thread. One oversized
/// document should not leave a permanently oversized arena behind, so anything
/// past this bound is dropped and re-sized for the next source.
const MAX_RETAINED_ARENA_BYTES: usize = 1 << 20;

struct Scratch {
    allocator: Allocator,
    renderer: HtmlRenderer,
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch {
        allocator: Allocator::new(),
        renderer: HtmlRenderer::new(),
    });
}

impl Scratch {
    /// Readies the arena for a source of `source_len` bytes.
    ///
    /// The retained chunk is kept when it is both large enough for this source
    /// and not oversized; otherwise the arena is replaced with one sized for
    /// the source, which is the same capacity a non-reusing caller would have
    /// picked.
    fn prepare(&mut self, source_len: usize) {
        let retained = self.allocator.allocated_bytes();
        if retained < Allocator::capacity_for_source_len(source_len)
            || retained > MAX_RETAINED_ARENA_BYTES
        {
            self.allocator = Allocator::for_source_len(source_len);
            return;
        }
        self.allocator.reset();
    }
}

/// Parses `source` and renders it to HTML with a reused arena and renderer.
pub fn parse_and_render_html(source: &str, options: ParserOptions) -> Result<String, String> {
    SCRATCH.with_borrow_mut(|scratch| {
        scratch.prepare(source.len());
        // Split the borrow so the parsed document (which borrows the arena) and
        // the renderer can be live at the same time.
        let Scratch { allocator, renderer } = scratch;
        match Parser::with_options(allocator, source, options).parse() {
            Ok(document) => Ok(renderer.render_borrowed(&document).to_owned()),
            Err(error) => Err(error.to_string()),
        }
    })
}

/// Parses `source` into mdast JSON with a reused arena.
pub fn parse_to_mdast_json(source: &str, options: ParserOptions) -> Result<String, String> {
    SCRATCH.with_borrow_mut(|scratch| {
        scratch.prepare(source.len());
        match Parser::with_options(&scratch.allocator, source, options).parse() {
            Ok(document) => Ok(mdast::to_mdast_json(&document)),
            Err(error) => Err(error.to_string()),
        }
    })
}

/// Parses `source` into a raw mdast transfer buffer with a reused arena.
pub fn parse_to_mdast_raw(source: &str, options: ParserOptions) -> Result<Vec<u8>, String> {
    SCRATCH.with_borrow_mut(|scratch| {
        scratch.prepare(source.len());
        match Parser::with_options(&scratch.allocator, source, options).parse() {
            Ok(document) => mdast_raw::to_mdast_raw(&document).map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        }
    })
}
