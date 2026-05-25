---
layout: entry
title: Ox Content
description: Rust-powered document generator and high-performance Markdown toolkit for JavaScript, with framework-agnostic pipelines, OG image support, and zero-JavaScript-first MPA output.
hero:
  text: cargo doc for JavaScript
  tagline: A framework-agnostic document generator and high-performance Markdown toolkit for the Vite era, with OG images, theming, search, Rust speed, and zero-JavaScript-first MPA output.
  notice:
    title: Unofficial project notice
    body:
      - This project is not an official VoidZero product.
      - ubugeeei is building ox-content as an unofficial proposal, hoping it could someday be adopted as vp doc.
      - The current branding and visual references are unofficial fan work and will be revised or taken down if VoidZero or the relevant rights holders ask.
  image:
    src: oxcontent-dark.svg
    lightSrc: oxcontent-dark.svg
    darkSrc: oxcontent-light.svg
    alt: Ox Content wordmark
    width: 302
    height: 64
  actions:
    - theme: brand
      text: Get Started
      link: getting-started.md
    - theme: alt
      text: View on GitHub
      link: https://github.com/ubugeeei/ox-content
    - theme: alt
      text: Sponsor
      link: https://github.com/sponsors/ubugeeei
features:
  - icon: "mdi:file-document-outline"
    title: cargo doc for JavaScript
    details: "Generate docs for JavaScript and TypeScript projects with a docs.rs-like bias, plus first-class Markdown pages."
    link: getting-started.md
  - icon: "mdi:layers-triple"
    title: Framework Agnostic, Vite Native
    details: A framework-agnostic pipeline with built-in OG image generation, search, theming, API docs, and content processing.
    link: theming.md
  - icon: "mdi:lightning-bolt"
    title: Rust + VoidZero DNA
    details: Implemented in Rust for speed and built to feel at home in the Vite, Oxc, Rolldown, and Vitest ecosystem.
    link: architecture.md
  - icon: "mdi:web"
    title: Zero-JavaScript-First MPA
    details: Ships as a fast multi-page app by default, then adds JavaScript only where islands or interactive features actually need it.
  - icon: "mdi:puzzle-outline"
    title: High-Performance Markdown Engine
    details: The parser, renderer, and plugin system are reusable as a Markdown library, not just internals behind the default docs theme.
    link: architecture.md
  - icon: "mdi:connection"
    title: Vue, Svelte, React Integrations
    details: First-party integrations let you embed framework components into Markdown without giving up the core pipeline.
---

## What Ox Content Is

Ox Content is a Rust-powered document generator and high-performance Markdown processing toolkit for JavaScript and TypeScript projects.

If you want the shortest explanation, it is best understood as `cargo doc` for JavaScript with a Vite-native workflow.

It also works as a framework-agnostic documentation pipeline with extra batteries included: built-in OG image generation, full-text search, theming hooks, API doc generation, and a reusable content engine.

The site output is zero-JavaScript-first and MPA-oriented by default, so the baseline experience stays fast and simple. When you do need interactivity, Ox Content can hydrate islands and integrate with Vue, Svelte, and React.

Under the hood, Ox Content is not only a docs theme. It also exposes the Markdown parser, renderer, transforms, and plugin system as reusable pieces, so you can use it as a Markdown library outside the default site generator.

## Why It Is Fast

- Implemented in Rust with arena-based allocation and zero-copy parsing where possible
- Designed to reuse ideas and ecosystem conventions that feel natural alongside the VoidZero family
- Optimized for static, cache-friendly multi-page output instead of assuming a heavy client runtime

## Integrations

- [Vue Integration Example](./examples/integ-vue.md) - Embed Vue components in Markdown
- [React Integration Example](./examples/integ-react.md) - Use React islands inside Markdown pages
- [Svelte Integration Example](./examples/integ-svelte.md) - Bring Svelte components into the same pipeline

## Quick Links

- [Getting Started](./getting-started.md) - Installation and first steps
- [Development Setup](./development-setup.md) - Build ox-content itself and work on the repo
- [Architecture](./architecture.md) - Deep dive into the design
- [unplugin mdast Bridge Example](./examples/unplugin-mdast-bridge.md) - Native parser plus unified-compatible mdast plugins
- [unplugin markdown-it Token Bridge](./examples/unplugin-markdown-it-token-bridge.md) - `markdown-it` plugins plus downstream unified token access
- [Theming](./theming.md) - Customize your documentation site
- [API Reference](./api/index.md) - Generated API docs for the public surface
- [GitHub](https://github.com/ubugeeei/ox-content) - Source code and issues

## Community Credits

Special thanks to [kazupon](https://github.com/kazupon) for substantial community contributions around JSDoc support, including the API docs generation pipeline and documentation quality.

## Benchmarks

Ox Content is positioned both as a document generator and as a high-performance Markdown toolkit. The numbers below focus on the Markdown engine side.

Latest local benchmark sweep on 2026-05-25 with Node `v24.15.0` on Apple M5 Pro. The tables below show median results from 7 local runs of the benchmark harness for the large 48.7 KB case.

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    4207 |  0.24 ms | 200.20 MB/s |
| `md4x (napi)`      |    1231 |  0.81 ms |  58.56 MB/s |
| `md4w (md4c)`      |    1143 |  0.87 ms |  54.41 MB/s |
| `markdown-it`      |    1035 |  0.97 ms |  49.24 MB/s |
| `marked`           |     530 |  1.89 ms |  25.23 MB/s |
| `remark`           |      44 | 22.74 ms |   2.09 MB/s |

### Parse + Render (48.7 KB)

| Library             | ops/sec | avg time |  throughput |
| ------------------- | ------: | -------: | ----------: |
| `@ox-content/napi`  |    4503 |  0.22 ms | 214.26 MB/s |
| `Bun.markdown.html` |    4225 |  0.24 ms | 201.06 MB/s |
| `md4x (napi)`       |    4014 |  0.25 ms | 191.02 MB/s |
| `md4w (md4c)`       |    2653 |  0.38 ms | 126.23 MB/s |
| `markdown-it`       |     840 |  1.19 ms |  39.96 MB/s |
| `marked`            |     470 |  2.13 ms |  22.36 MB/s |
| `micromark`         |      45 | 22.35 ms |   2.13 MB/s |
| `remark`            |      36 | 28.16 ms |   1.69 MB/s |

In this latest local release-build sweep, Ox Content leads every comparison: 3.4× ahead of the next-fastest native parser (`md4x (napi)`) on parse-only and 1.07× ahead of `Bun.markdown.html` on parse+render, while remaining the native core that drives the full documentation pipeline. Margins widen further on small documents — see `node benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, and large inputs.

Reproduce with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The benchmark includes `md4w (md4c)` and `md4x (napi)` by default and adds `Bun.markdown.html` automatically when `bun` is available.
