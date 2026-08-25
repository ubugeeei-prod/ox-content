//! Per-instance scratch state for the synchronous wasm entry points.
//!
//! A `parseAndRender` call on a small document spends most of its time in
//! setup rather than Markdown work: building a fresh arena, constructing an
//! [`HtmlRenderer`] (whose options own heap strings), and growing an output
//! buffer up from zero — the same fixed costs the NAPI binding shed in its
//! `render_scratch` module. Wasm runs single-threaded, so one cached arena
//! and renderer per thread-local slot serves every call.
//!
//! Reuse is safe for the same reason as on the NAPI side: every entry point
//! returns owned data (a JS string / object), so nothing borrowing the arena
//! survives the call to observe the next reset. The renderer clears its
//! per-document state (output, TOC entries, heading-id counts) at the top of
//! each render.

use std::cell::RefCell;

use ox_content_allocator::Allocator;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

/// Largest arena capacity carried over to the next call. One oversized
/// document should not pin an oversized chunk for the rest of the instance's
/// lifetime; anything past this bound is dropped and re-sized.
const MAX_RETAINED_ARENA_BYTES: usize = 1 << 20;

/// The renderer-affecting subset of `WasmParserOptions`, kept alongside the
/// cached renderer so a call with different options rebuilds it instead of
/// silently rendering with stale ones.
#[derive(PartialEq, Clone)]
pub struct RendererKey {
    pub toc_max_depth: u8,
    pub autolink_urls: bool,
    pub autolink_target_blank: bool,
    pub autolink_patterns: Vec<String>,
    pub semantic_footnotes: bool,
    pub heading_permalinks: bool,
}

struct Scratch {
    allocator: Allocator,
    renderer: HtmlRenderer,
    renderer_key: RendererKey,
}

fn default_key() -> RendererKey {
    let defaults = HtmlRendererOptions::default();
    RendererKey {
        toc_max_depth: defaults.toc_max_depth,
        autolink_urls: defaults.autolink_urls,
        autolink_target_blank: defaults.autolink_target_blank,
        autolink_patterns: defaults.autolink_patterns,
        semantic_footnotes: defaults.semantic_footnotes,
        heading_permalinks: defaults.heading_permalinks,
    }
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch {
        allocator: Allocator::new(),
        renderer: HtmlRenderer::new(),
        renderer_key: default_key(),
    });
}

/// Runs `f` with an arena readied for `source_len` bytes and a renderer
/// configured for `key` — both reused across calls when possible.
pub fn with_scratch<R>(
    source_len: usize,
    key: &RendererKey,
    f: impl FnOnce(&Allocator, &mut HtmlRenderer) -> R,
) -> R {
    SCRATCH.with_borrow_mut(|scratch| {
        let retained = scratch.allocator.allocated_bytes();
        if retained < Allocator::capacity_for_source_len(source_len)
            || retained > MAX_RETAINED_ARENA_BYTES
        {
            scratch.allocator = Allocator::for_source_len(source_len);
        } else {
            scratch.allocator.reset();
        }

        // Rebuilding the renderer costs the same as the old fresh-per-call
        // path, so a caller cycling through option sets is no worse off; a
        // caller with stable options (the common case, and the default) pays
        // one comparison.
        if scratch.renderer_key != *key {
            scratch.renderer = HtmlRenderer::with_options(HtmlRendererOptions {
                toc_max_depth: key.toc_max_depth,
                autolink_urls: key.autolink_urls,
                autolink_target_blank: key.autolink_target_blank,
                autolink_patterns: key.autolink_patterns.clone(),
                semantic_footnotes: key.semantic_footnotes,
                heading_permalinks: key.heading_permalinks,
                ..Default::default()
            });
            scratch.renderer_key = key.clone();
        }

        f(&scratch.allocator, &mut scratch.renderer)
    })
}
