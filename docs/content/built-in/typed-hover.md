---
title: Typed Hover
description: Opt-in build-time TypeScript hover overlays for twoslash fences. No compiler ships to the browser.
---

# Typed Hover

TypeScript samples can already be type-checked at build time with
[`codeBlockTypecheck`](./quality-checks.md). Readers still cannot see those
types in the rendered fence unless you opt in here.

`typedHover` is off by default. When enabled, **only** TypeScript and TSX
fences tagged `twoslash` receive hover payloads. Types are computed during
the Markdown transform. The page ships JSON plus a tiny overlay script —
**no TypeScript compiler runs in the browser**.

| Option       | Type                            | Default |
| ------------ | ------------------------------- | ------- |
| `typedHover` | `boolean` / `TypedHoverOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      typedHover: true,
    }),
  ],
};
```

`true` and `{}` enable the defaults. Omit the option or pass `false` to leave
every fence unchanged. An object enables the feature and can override
`languages` (default `["ts", "tsx"]`).

This site enables `typedHover`, so the next block is a live overlay. Hover or
tab to `value`:

```ts twoslash
const value = 1;
```

## Fence meta

Authors opt in **per fence** with the `twoslash` meta token. Fences without
that token are skipped, even when the site option is on:

````md
```ts twoslash
const value = 1;
```

```ts
const skipped = 1;
```
````

| Fence                          | Overlay |
| ------------------------------ | ------- |
| ` ```ts twoslash `             | yes     |
| ` ```tsx twoslash `            | yes     |
| ` ```ts ` (no meta)            | no      |
| ` ```js twoslash `             | no      |
| Inline `` `const value = 1` `` | no      |

`twoslash` is the same meta that [`codeBlockTypecheck`](./quality-checks.md)
already recognizes. A fence can be type-checked and receive hovers without a
second marker. Incomplete snippets that you do not want to check can omit
`typecheck` and still use `twoslash` for overlays.

## Build time, not the browser

Payloads are `{ start, end, type }` ranges generated while the page is
transformed. The plugin reuses the existing TypeScript fence path:
`extractCodeBlocks` writes matching snippets and asks `tsgo` (via
`@typescript/native-preview`) for types at identifier offsets. The browser
never downloads `typescript`, `tsgo`, or a language service.

Install the same compiler used by [`codeBlockTypecheck`](./quality-checks.md):

<pm>npm install -D @typescript/native-preview</pm>

Each opted-in fence is wrapped with `class="ox-typed-hover"`. The payload
sits in a neighboring `<script type="application/json">` after `<` / `>`
have been escaped as `\u003c` / `\u003e`, so a type string cannot break out
of the script or inject markup.

## Keyboard and pointer

Identifiers that have a type become

`<span class="ox-typed-hover-token" tabindex="0">`.

- **Pointer:** hover the token to open a small overlay.
- **Keyboard:** tab to the token. Focus shows the same overlay. `Escape`
  dismisses it.
- The overlay uses `role="tooltip"` and is filled with `textContent`, never
  `innerHTML`.

Tokens are underlined with a dotted decoration so they are discoverable
without a mouse.

## What stays literal

The transform does not rewrite:

- fences that omit `twoslash`
- JavaScript, JSON, or other non-`ts` / `tsx` fences
- inline code spans
- indented code
- unclosed fences (they do not consume the rest of the file as hover
  targets)

Hostile type strings such as `<img onerror>` or `</script>` are escaped in
the JSON payload and rendered as text in the overlay.

## Related

- [Quality Checks](./quality-checks.md) — `codeBlockTypecheck` via `tsgo`
- [Code Blocks](./code-blocks.md) — highlighting and annotations
- [Built-in Features overview](../built-in-features.md)
