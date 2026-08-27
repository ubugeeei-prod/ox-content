---
title: BudouX
description: Opt-in build-time phrase segmentation for Japanese line breaking.
---

# BudouX

`budoux` inserts zero-width spaces between phrases during the Markdown
transform. It improves line-break opportunities for Japanese prose while
keeping the output static.

The feature is off by default. Install the optional `budoux` package only in
projects that enable it:

```sh
pnpm add -D budoux
```

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      budoux: true,
    }),
  ],
};
```

## Output

Given Japanese prose:

```md
今日はとても良い天気です。
```

the rendered HTML contains phrase separators:

```html
<p>今日は\u200bとても\u200b良い\u200b天気です。</p>
```

Only the HTML text content changes. The generated page module does not import
`budoux`, and no parser is included in browser bundles.

## Protected Markup

The transform copies HTML tags and attributes without changing them. It keeps
HTML entities as entities, skips `code`, `pre`, `script`, `style`, `textarea`,
`svg`, and `math`, and leaves island JSON payloads untouched. Link URLs stay
unchanged; link labels are visible text and are segmented.

Visible text inside normal Markdown blocks is segmented, including paragraphs,
headings, list items, blockquotes, table cells, and island slot HTML. This means
framework adapters that run local island SSR receive already-segmented slot
HTML, while component props and island payload data remain unchanged.

## Styling

BudouX creates break opportunities, but CSS decides how aggressively the
browser wraps. Ox Content does not inject inline styles. Add the rule to your
content container or theme when you want BudouX-style wrapping:

```css
.ox-content {
  word-break: keep-all;
  overflow-wrap: anywhere;
}
```

## Options

| Option      | Type                                        | Default        |
| ----------- | ------------------------------------------- | -------------- |
| `enabled`   | `boolean`                                   | `true`         |
| `language`  | `"ja"` / `"zh-hans"` / `"zh-hant"` / `"th"` | `"ja"`         |
| `separator` | `string`                                    | `"\u200b"`     |
| `parser`    | `{ parse(text: string): string[] }`         | default parser |

Use `language` to load another default BudouX parser:

```ts
oxContent({
  budoux: {
    language: "ja",
  },
});
```

Pass `parser` when a site owns a custom model. In that case Ox Content does not
import the optional `budoux` package:

```ts
oxContent({
  budoux: {
    parser: customParser,
  },
});
```

## Related

- [Syntax Extensions](./syntax-extensions.md) - other opt-in Markdown features.
- [CJK Emphasis](../examples/cjk-emphasis.md) - emphasis parsing near CJK punctuation.
