---
title: Math
description: Opt-in inline `$…$` and display `$$…$$` math with escaped static markup.
---

# Math

Math authoring is opt-in. Ordinary `$` characters stay literal until a site
enables the transform:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      math: true,
    }),
  ],
};
```

`true` and `{}` both enable the defaults. Omit the option or pass `false` to
leave `$` and `$$` unchanged.

## Delimiters

| Form    | Source               | Result                                   |
| ------- | -------------------- | ---------------------------------------- |
| Inline  | `$E=mc^2$`           | `<span class="ox-math ox-math-inline">…` |
| Display | `$$E = mc^2$$`       | `<div class="ox-math ox-math-block">…`   |
| Inline  | `Before $$x$$ after` | `<span class="ox-math ox-math-inline">…` |

Display delimiters produce a block only when they occupy the whole paragraph.
Surrounded `$$…$$` stays inline so Markdown does not emit a `<div>` inside a
`<p>`.

The transform emits accessible static MathML. TeX is placed in `<mtext>` after
the same HTML escaping used elsewhere in the pipeline, so `<script>`, quotes,
and attribute-like fragments cannot become raw HTML.

## Rendered Example

Inline: the identity is $E=mc^2$.

Display:

$$E = mc^2$$

Fenced code, indented code, and inline code are not rewritten:

````md
```
$E=mc^2$
```

Use `$E=mc^2$` in prose. Currency stays literal: `$5`, `$5.00`, `$5-$10`, `US$`.
````

Unclosed `$` or `$$` stays literal and does not consume the rest of the file.
Write `\$` when math is on and you need a literal dollar sign.

GitHub-style `> [!NOTE]` callouts and other existing syntax are unchanged.

## Related

- [Syntax Extensions](./syntax-extensions.md) — other opt-in Markdown syntax.
- [Built-in Features overview](../built-in-features.md)
