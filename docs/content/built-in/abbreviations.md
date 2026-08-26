---
title: Abbreviations
description: Opt-in glossary terms rendered as accessible `<abbr>` markup.
---

# Abbreviations

Long-lived docs accumulate acronyms and product names. Define a term once and
ox-content expands matching text into `<abbr class="ox-abbr">` with a `title`.
The feature is opt-in and off by default. There is no client JavaScript.

| Option          | Type                               | Default |
| --------------- | ---------------------------------- | ------- |
| `abbreviations` | `boolean` / `AbbreviationsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      abbreviations: true,
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the transform.

## Authoring

Write a Markdown Extra definition on its own line:

*[LSP]: Language Server Protocol

```md
*[LSP]: Language Server Protocol

Use LSP in the editor.
```

Use LSP in the editor.

The definition line is consumed. Matching uses Unicode word boundaries, so
`XLSPY` and `myLSP` stay literal. Config `terms` work the same way and can
live in a shared glossary. A page-local definition overrides a config term
with the same key.

Malformed lines such as `*[LSP]` or `*[LSP]:` stay visible. Fenced, indented,
and inline code, HTML comments, raw `code` / `pre` / `script` / `style`, and
existing links are not rewritten.

## Options

```ts
oxContent({
  abbreviations: {
    terms: {
      LSP: "Language Server Protocol",
    },
    firstUseOnly: false,
  },
});
```

| Field          | Type                     | Default |
| -------------- | ------------------------ | ------- |
| `enabled`      | `boolean`                | `true`  |
| `terms`        | `Record<string, string>` | `{}`    |
| `firstUseOnly` | `boolean`                | `false` |

`firstUseOnly: true` wraps only the first occurrence of each term. The default
expands every match. Titles and term text are HTML-escaped.

## Related

- [Keyboard Keys](./keyboard-keys.md)
- [Syntax Extensions](./syntax-extensions.md)
- [Built-in Features overview](../built-in-features.md)
