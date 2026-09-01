# Incremental HTML/JS Example

Minimal browser demo for Ox Content incremental rendering.

The Node server owns the native Rust/N-API `IncrementalMarkdownRenderer`. The
browser is plain HTML, CSS, and JavaScript; it receives render updates over
server-sent events and applies them with the builtin
`@ox-content/vite-plugin/incremental-dom` helper.

```bash
vp run --filter @ox-content/vite-plugin build
vp run --filter ./examples/incremental-html-js dev
```

Open http://localhost:4174.
