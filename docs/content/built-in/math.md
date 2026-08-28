---
title: Math
description: Opt-in `$…$` / `$$…$$` math, typeset at build time with optional KaTeX.
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

The plugin does not depend on KaTeX. Sites that never turn math on do not
install it. When math is on, the native transform finds `$…$` / `$$…$$` and
KaTeX — if present — turns that TeX into static HTML at build time. Readers
get typeset math with no client-side JavaScript.

## Delimiters

| Form    | Source               | Result                                   |
| ------- | -------------------- | ---------------------------------------- |
| Inline  | `$E=mc^2$`           | `<span class="ox-math ox-math-inline">…` |
| Display | `$$E = mc^2$$`       | `<div class="ox-math ox-math-block">…`   |
| Inline  | `Before $$x$$ after` | `<span class="ox-math ox-math-inline">…` |

Display delimiters produce a block only when they occupy the whole paragraph.
Surrounded `$$…$$` stays inline so Markdown does not emit a `<div>` inside a
`<p>`.

Unclosed `$` or `$$` stays literal and does not consume the rest of the file.
Write `\$` when math is on and you need a literal dollar sign. Fenced code,
indented code, and inline code are not rewritten. Currency such as `$5` or
`$5.00` stays literal.

## Rendered Example

Inline: the identity is $E=mc^2$.

Display:

$$
\int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}
$$

A Gaussian density:

$$
\frac{1}{\sqrt{2\pi\sigma^2}}
\exp\left(-\frac{(x-\mu)^2}{2\sigma^2}\right)
$$

```md
Inline: the identity is $E=mc^2$.

$$
\int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}
$$
```

## When TeX Does Not Parse

The `$…$` scan is a heuristic. It leaves `${score}` in prose, `$vuetify`, and
`$10 から $20` alone, but a page _about_ math syntax — one that quotes `$…$` to
explain it — is exactly the page it picks up by mistake. KaTeX then refuses
what it was handed, and `onError` decides what the reader sees:

```ts
oxContent({
  math: { onError: "literal" },
});
```

| `onError`   | Result                                                                |
| ----------- | --------------------------------------------------------------------- |
| `'literal'` | The source goes back as written, delimiters included, plus a warning. |
| `'error'`   | The build fails on the first run KaTeX refuses.                       |
| `'render'`  | KaTeX's own red error text is written into the page.                  |

`'literal'` is the default: a sentence that merely mentions `$` keeps reading
like a sentence, and the warning names the file and the TeX so a real mistake
in a formula is still visible. Use `'error'` on a site where every `$…$` is
meant to be math, and `'render'` to see KaTeX's message in place.

## Requirements

Typesetting uses KaTeX at build time, so add it only on sites that enable
math:

<pm>npm install -D katex</pm>

If `katex` cannot be found, the build does not fail: the escaped TeX
placeholder stays in the page and a warning is printed once. That keeps
sites that only want the delimiter scan — or CI images without the extra
package — working while you decide whether typesetting is worth the
dependency.

## Emitted Assets

KaTeX's stylesheet and fonts land in `__ox_katex__/` in the output directory,
and only the pages that rendered math link them. A site that turns `math` on
without writing any gets nothing: the assets are emitted when at least one
page asks for them.

Only the `woff2` fonts ship. `.ttf` and `.woff` are three quarters of the font
bytes and no browser that can run the rest of the site needs them —
`@font-face` lists `woff2` first and stops at the first format it supports.
Ask for the rest when a target genuinely needs them:

```ts
oxContent({
  math: { fontFormats: "all" },
});
```

## Related

- [Syntax Extensions](./syntax-extensions.md) — other opt-in Markdown syntax.
- [Built-in Features overview](../built-in-features.md)
