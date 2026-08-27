---
title: Syntax Extensions
description: Opt-in authoring syntax - emoji shortcodes, wiki links, attribute syntax, and CJK emphasis.
---

# Syntax Extensions

Non-standard Markdown syntax is opt-in, so ordinary documents render the same
everywhere until a site explicitly enables an extension.

| Option            | Type                                 | Default |
| ----------------- | ------------------------------------ | ------- |
| `emojiShortcodes` | `boolean` / `EmojiShortcodeOptions`  | `false` |
| `wikiLinks`       | `boolean` / `WikiLinkOptions`        | `false` |
| `attrs`           | `boolean` / `AttrsOptions`           | `false` |
| `crossReferences` | `boolean` / `CrossReferencesOptions` | `false` |
| `xrefs`           | Alias for `crossReferences`          | `false` |
| `citations`       | `boolean` / `CitationsOptions`       | `false` |
| `budoux`          | `boolean` / `BudouxOptions`          | `false` |
| `cjkEmphasis`     | `boolean`                            | `false` |
| `magicLinks`      | `boolean` / `MagicLinkOptions`       | `false` |
| `notByAi`         | `boolean` / `NotByAiOptions`         | `false` |
| `keyboardKeys`    | `boolean` / `KeyboardKeysOptions`    | `false` |
| `abbreviations`   | `boolean` / `AbbreviationsOptions`   | `false` |
| `definitionLists` | `boolean` / `DefinitionListOptions`  | `false` |

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

[Slides](https://example.com/slides){.deck-link data-kind=deck}

![Architecture](./architecture.png){.w-1/2 .mx-auto width=480}
```

produces:

```html
<p class="lead">A lead paragraph.</p>

<h2 id="install" class="section" data-section="install">Install</h2>

<p><a href="https://example.com/slides" class="deck-link" data-kind="deck">Slides</a></p>

<p><img src="./architecture.png" alt="Architecture" class="w-1/2 mx-auto" width="480" /></p>
```

The transform runs as a post-render HTML pass over the full document — raw
HTML embedded in Markdown is affected as well, so literal `{...}` examples
belong in code spans or fences.

## CJK Emphasis

Emphasis adjacent to CJK characters needs no configuration — CommonMark's
delimiter rules already allow it, and no ASCII spaces are required:

```md
これは**重要**です。次の文でも*強調*できます。
```

Rendered:

これは**重要**です。次の文でも*強調*できます。

What plain CommonMark rejects is a delimiter run sitting directly against
punctuation on its outer side. Its flanking rules read Unicode punctuation as a
whole, so East Asian punctuation blocks a run just like ASCII punctuation does,
and `A**強調。**B` stays literal text. Latin prose rarely hits this because a
space usually separates the two; CJK sets punctuation against the preceding
word, so it comes up constantly.

`cjkEmphasis` classifies East Asian punctuation as an ordinary character for
that decision only:

```ts
oxContent({
  cjkEmphasis: true,
});
```

```md
A**強調。**B
```

renders as `A<strong>強調。</strong>B` with the option on, and as literal text
with it off. Halfwidth ASCII punctuation is deliberately untouched, so a Latin
document parses identically either way.

This is a deliberate deviation from the specification, which is why it is
opt-in. See [CJK Emphasis](../examples/cjk-emphasis.md) for the exact boundary
and the reclassified character ranges.

## Magic Links

Opt-in `{link:@user}`, `{link:alias}`, and `{link:label|url}` rich links with
optional avatars. Off by default. See [Magic Links](./magic-links.md).

## Cross References

Opt-in `@sec-*`, `@fig-*`, and `@tbl-*` references link to labeled headings,
figures, images, and tables. Off by default. See
[Cross References](./cross-references.md).

## Citations

Opt-in `[@key]` and `[@key; -@other]` references link to generated bibliography
entries loaded from local CSL JSON. Off by default. See
[Citations](./citations.md).

## BudouX

Opt-in `budoux` inserts zero-width spaces into visible prose at build time for
better Japanese line breaking. It leaves tags, attributes, URLs, entities,
code, raw HTML blocks, and island JSON payloads unchanged. Off by default. See
[BudouX](./budoux.md).

## NotByAI Badge

Opt-in `<NotByAI />` emits a static human-authorship disclosure. It is not a
status badge. Off by default. See [NotByAI Badge](./not-by-ai.md).

## Keyboard Keys

Opt-in `{kbd:Ctrl+K}` and `{kbd:Cmd Shift P}` semantic shortcuts. Off by
default. See [Keyboard Keys](./keyboard-keys.md).

## Abbreviations

Opt-in `*[LSP]: Language Server Protocol` glossary expansion. Off by
default. See [Abbreviations](./abbreviations.md).

## Definition Lists

Opt-in `Term` / `: definition` glossary lists. Off by default. See
[Definition Lists](./definition-lists.md).

## Related

- [Markdown Baseline](./markdown.md) — the default syntax these extensions
  build on.
- [Code Blocks](./code-blocks.md) — annotation and import syntax for fences.
- [Magic Links](./magic-links.md) — configurable GitHub, alias, and URL links.
- [Cross References](./cross-references.md) — generated labels for sections,
  figures, and tables.
- [Citations](./citations.md) — generated bibliography-backed references.
- [BudouX](./budoux.md) — build-time Japanese phrase segmentation.
- [Keyboard Keys](./keyboard-keys.md) — `{kbd:Ctrl+K}` shortcut markup.
- [Abbreviations](./abbreviations.md) — `*[TERM]:` glossary expansion.
- [Definition Lists](./definition-lists.md) — opt-in `Term` / `: definition`
  glossaries.
