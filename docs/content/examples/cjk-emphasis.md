---
title: CJK Emphasis
description: Recognize emphasis adjacent to CJK text and CJK punctuation.
---

# CJK Emphasis

Emphasis between CJK characters works out of the box — CommonMark's delimiter
rules already allow it, and no ASCII spaces are needed:

```md
これは**重要**です。
これは*強調*です。
```

## Emphasis Against CJK Punctuation

What plain CommonMark rejects is a `*`/`_` run sitting directly against
punctuation on its outer side. Its flanking rules read Unicode punctuation as a
whole, so East Asian punctuation blocks a run the same way ASCII punctuation
does:

```md
A**強調。**B
中文**加粗，**测试
```

Both render as literal text in any spec-conformant engine. Latin prose rarely
runs into it because a space usually separates the punctuation from the
delimiter, but CJK sets punctuation directly against the words it follows, so it
comes up constantly.

`cjkEmphasis` classifies East Asian punctuation as an ordinary character for
that decision, which lets those runs pair:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      cjkEmphasis: true,
    }),
  ],
};
```

| Source                 | Default      | `cjkEmphasis: true`      |
| ---------------------- | ------------ | ------------------------ |
| `これは**重要**です。` | **重要**     | **重要**                 |
| `A**強調。**B`         | literal text | **強調。**               |
| `中文**加粗，**测试`   | literal text | **加粗，**               |
| `a**bold.**c`          | literal text | literal text (unchanged) |

Only the fullwidth and CJK-specific punctuation blocks are reclassified:
`U+3000`–`U+303F`, the vertical and compatibility forms (`U+FE10`–`U+FE19`,
`U+FE30`–`U+FE4F`), the small form variants (`U+FE50`–`U+FE6F`), and fullwidth
ASCII (`U+FF01`–`U+FF65`, skipping the fullwidth digits and letters).
Halfwidth ASCII punctuation is written the same way in every script, so
enabling the option never changes how a Latin document parses.

The option relaxes the punctuation rule only. Whitespace still blocks a run, so
`A** 強調。 **B` stays literal either way.

## Conformance

This is a deliberate deviation from the specification, which is why it is
opt-in: with the option off the parser renders every CommonMark 0.31.2 example
per spec. See [CommonMark Conformance](../performance.md#commonmark-conformance).
