---
title: Code Play Roadmap
description: PR sequence for the opt-in @ox-content/code-play plugin.
---

# Code Play Roadmap

Tracking issue: [#648](https://github.com/ubugeeei-prod/ox-content/issues/648).

Code Play is an opt-in plugin, `@ox-content/code-play`. Installing
`@ox-content/vite-plugin` must not pull it in, and installing Code Play must
not enable any language until the site lists that language.

The work is split into small pull requests so each one can land with its own
tests and changelog entry. A later PR may depend on an earlier one, never the
reverse.

## Architecture Principles

1. **Two opt-in layers.** Install the package, then enable languages. Disabled
   languages stay ordinary highlighted fences.
2. **Headless first.** `createCodePlay()` / `CodePlaySession` are the source of
   truth. Presets and viewers render that data; they do not own execution.
3. **No transform-time execution.** Markdown transform and SSG never run sample
   code. Readers trigger execute / type-check on demand.
4. **Sandboxes stay sandboxed.** JavaScript and TypeScript run in `node:vm` or
   a browser iframe. Native languages use official playgrounds or a
   user-configured HTTP executor. Code Play does not spawn `sh`, `python`, or
   `rustc` on the docs host unless a later local-runtime PR says so.
5. **Observability is API data.** stdio, dedicated `stdout` / `stderr` strings,
   config, provenance, and timing are returned on every `RunResult`, not only
   painted in the default UI.

## Language Matrix

| Language                             | Execute | Type-check | Default backend                       |
| ------------------------------------ | ------- | ---------- | ------------------------------------- |
| TypeScript                           | yes     | yes        | local `tsc` + `node:vm` / iframe      |
| Rust                                 | yes     | yes        | play.rust-lang.org                    |
| Go                                   | yes     | yes        | play.golang.org                       |
| JavaScript                           | yes     | no         | `node:vm` / iframe                    |
| Vue, React, Svelte, Solid            | yes     | no         | compiled iframe preview               |
| Python, PHP, Ruby, sh                | yes     | no         | configured Piston-compatible endpoint |
| Java, Swift, Kotlin                  | yes     | no         | configured Piston-compatible endpoint |
| C, C++, Zig, Haskell, OCaml          | yes     | no         | configured Piston-compatible endpoint |
| C#, Elixir, F#                       | yes     | no         | configured Piston-compatible endpoint |
| Lean, Rocq, Clojure, Scheme, MoonBit | yes     | no         | configured Piston-compatible endpoint |

## PR Sequence

### 1. `feat(code-play): plugin scaffold, headless API, viewers`

Shipped in #649. Package, catalog, headless client, default/compact/headless
UI, config / stdio / provenance / timing viewers, Vite plugin, and tests with
injected transports (no live network in CI).

### 1b. `feat(code-play): dedicated stderr viewer`

This PR. First-class `RunResult.stdout` / `RunResult.stderr`, a dedicated
stderr viewer (stream chunks plus error/warning diagnostics), and compact
preset coverage for stderr.

### 2. `feat(code-play): Vite SSG hydration and docs dogfood`

Harden page-level script emission for ox-content SSG (dev middleware + written
HTML), enable JavaScript / TypeScript widgets on the docs example page, and
add a visual check for the default preset.

### 3. `feat(code-play): official playground proxies`

Keep the allowlisted `/__ox-code-play/rust` and `/__ox-code-play/go` proxies,
add production `endpoints` documentation, and cover proxy failure modes.

### 4. `feat(code-play): framework preview compilers`

Optional peer compilers for Vue SFC, React JSX, Svelte, and Solid so previews
compile locally instead of shipping raw source into an import-map iframe.

### 5. `feat(code-play): optional in-browser runtimes`

Opt-in loaders such as Pyodide or Scheme interpreters. Still off by default;
each runtime is its own language enable flag.

### 6. `docs(code-play): security and privacy notes`

Document third-party playgrounds, endpoint trust, iframe sandbox flags, and
the "no local shell" guarantee in SECURITY.md and the package guide.

## Out of Scope

- Making Code Play a built-in `@ox-content/vite-plugin` option.
- Replacing WebContainer or StackBlitz embeds.
- Vendoring compilers or a hosted execute service.
- Running samples during `transformMarkdown` or `buildSsg`.

## Tracking

Progress is tracked through the conventional commit log and #648. This
document is updated in the same PR when an item lands.
