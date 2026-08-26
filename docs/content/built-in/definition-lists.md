---
title: Definition Lists
description: Opt-in Term / : definition markup rendered as semantic glossary lists.
---

# Definition Lists

Glossaries, option references, and protocol fields often need a term next to
one or more definitions. Tables are too heavy for that. The compact
PHP Markdown Extra / mdBook form is opt-in and off by default, so existing
pandoc-style source stays ordinary paragraphs and lists until a site turns
this on.

| Option            | Type                                | Default |
| ----------------- | ----------------------------------- | ------- |
| `definitionLists` | `boolean` / `DefinitionListOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      definitionLists: true,
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the transform. There is no client JavaScript.

## Authoring

Write a term on its own line, then one or more definitions that start with
`: ` (colon and space):

HTTP
: Hypertext Transfer Protocol
: Also the name of the request/response protocol used by the web.

TLS
: Transport Layer Security

```md
HTTP
: Hypertext Transfer Protocol
: Also the name of the request/response protocol used by the web.

TLS
: Transport Layer Security
```

The renderer emits a semantic list with a stable class for themes:

```html
<dl class="ox-definition-list">
  <dt>…</dt>
  <dd>…</dd>
</dl>
```

Inline Markdown inside terms and definitions is parsed:

**Status**
: A `2xx` response means **success**.

```md
**Status**
: A `2xx` response means **success**.
```

A blank line between the term and the first definition is also accepted.
Several terms may share the following definitions. Invalid or ambiguous
forms — a lone `: definition`, a list item followed by `: `, or a wrapped
paragraph — stay ordinary paragraphs or lists.

Fenced, indented, and inline code, HTML comments, and raw `code` / `pre` /
`script` / `style` are not rewritten.

## Options

```ts
oxContent({
  definitionLists: {
    enabled: true,
  },
});
```

| Field     | Type      | Default |
| --------- | --------- | ------- |
| `enabled` | `boolean` | `true`  |

## Related

- [Syntax Extensions](./syntax-extensions.md)
- [Keyboard Keys](./keyboard-keys.md)
- [Built-in Features overview](../built-in-features.md)
