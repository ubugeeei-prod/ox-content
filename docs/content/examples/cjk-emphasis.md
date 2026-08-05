---
title: CJK Emphasis
description: Recognize emphasis adjacent to CJK text.
---

# CJK Emphasis

The native parser recognizes emphasis next to CJK characters without requiring
ASCII spaces.

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

```md
これは**重要**です。
これは*強調*です。
```

The option is explicit for compatibility, while the parser keeps the fast common
inline path. It does not change parsing: the cases above are handled with or
without it, because CommonMark's delimiter rules already allow emphasis between
CJK characters.

## What Is Not Covered

CommonMark does not recognize `**` placed immediately **inside CJK
punctuation**:

```md
A**強調。**B
A**、強調**B
```

Both render as literal text rather than bold. This is a property of the
specification's left/right-flanking delimiter rules, so it affects every
spec-conformant engine — `markdown-it` in `commonmark` mode, `micromark`,
`pulldown-cmark`, and Ox Content all behave the same way here.

Ox Content does not currently deviate from the specification for this case. If
you need it, keep the punctuation outside the emphasis:

```md
A**強調**。B
```

See [CommonMark Conformance](../performance.md#commonmark-conformance) for where
Ox Content does and does not follow the specification.
