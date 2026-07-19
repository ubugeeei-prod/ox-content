---
title: Syntax Extensions
description: Opt-in authoring syntax - emoji shortcodes, wiki links, attribute syntax, and CJK emphasis.
---

# Syntax Extensions

Non-standard Markdown syntax is opt-in, so ordinary documents render the same
everywhere until a site explicitly enables an extension.

| Option            | Type                                | Default |
| ----------------- | ----------------------------------- | ------- |
| `emojiShortcodes` | `boolean` / `EmojiShortcodeOptions` | `false` |
| `wikiLinks`       | `boolean` / `WikiLinkOptions`       | `false` |
| `attrs`           | `boolean` / `AttrsOptions`          | `false` |
| `cjkEmphasis`     | `boolean`                           | `false` |

## Emoji Shortcodes

Expand GitHub-style `:shortcode:` aliases to Unicode emoji:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      emojiShortcodes: true,
    }),
  ],
};
```

The built-in table covers hundreds of common aliases. Expansion happens outside
fenced and inline code, and unknown shortcodes are left unchanged:

```md
Ship it :rocket: :tada:

Status: :white_check_mark: passed, :warning: flaky, :x: failed

Unknown aliases like :no-such-emoji: stay untouched, and so does
inline code: `:rocket:`.
```

Rendered:

Ship it :rocket: :tada:

Status: :white_check_mark: passed, :warning: flaky, :x: failed

Unknown aliases like :no-such-emoji: stay untouched, and so does
inline code: `:rocket:`.

### Custom shortcodes

Custom values are merged into the built-in table and override it on conflict.
Keys are written without colons:

```ts
oxContent({
  emojiShortcodes: {
    custom: {
      shipit: "🚢",
      oxc: "🦀",
    },
  },
});
```

## Wiki Links

Resolve Obsidian-style `[[target]]` links into normal site links:

```ts
oxContent({
  wikiLinks: {
    // Defaults to the top-level `base` option.
    baseUrl: "/docs/",
  },
});
```

The expansion runs before Markdown parsing, and fenced code blocks and inline
code spans are protected. Given this source:

```md
See [[getting-started|Getting started]] and [[api/transform#options]].
```

the transform emits:

```html
<p>
  See <a href="/docs/getting-started">Getting started</a> and
  <a href="/docs/api/transform#options">api/transform#options</a>.
</p>
```

`[[target]]` uses the target as the label, `[[target|label]]` overrides it, and
`#fragment` parts are slugified. Site-relative targets are prefixed with
`baseUrl`.

Wiki links also run before raw HTML is parsed, so `[[...]]` inside literal
`<code>` tags in embedded HTML is expanded too — keep literal examples inside
Markdown code spans or fences instead.

## Attribute Syntax

Add IDs, classes, and attributes with `markdown-it-attrs` syntax:

```ts
oxContent({
  attrs: true,
});
```

Supported tokens are `#id`, `.class`, and `key=value`. A trailing `{...}` block
attaches to the element rendered from that line:

```md
A lead paragraph. {.lead}

## Install {.section data-section=install}
```

produces:

```html
<p class="lead">A lead paragraph.</p>

<h2 id="install" class="section" data-section="install">Install</h2>
```

The transform runs as a post-render HTML pass over the full document — raw
HTML embedded in Markdown is affected as well, so literal `{...}` examples
belong in code spans or fences.

## CJK Emphasis

The native parser already recognizes `**emphasis**` adjacent to CJK characters
without requiring ASCII spaces — a case where many CommonMark parsers need
manual workarounds. The `cjkEmphasis` option keeps that behavior explicit in
the public API for compatibility contracts:

```ts
oxContent({
  cjkEmphasis: true,
});
```

```md
これは**重要**です。句読点の前でも*強調*できます。
```

Rendered:

これは**重要**です。句読点の前でも*強調*できます。

## Related

- [Markdown Baseline](./markdown.md) — the default syntax these extensions
  build on.
- [Code Blocks](./code-blocks.md) — annotation and import syntax for fences.
