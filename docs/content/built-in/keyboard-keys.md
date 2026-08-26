---
title: Keyboard Keys
description: Opt-in `{kbd:...}` shortcuts rendered as semantic key markup.
---

# Keyboard Keys

Product docs and editor guides often need shortcuts such as Ctrl K or
Command Shift P. `{kbd:...}` markup is opt-in and off by default.

| Option         | Type                              | Default |
| -------------- | --------------------------------- | ------- |
| `keyboardKeys` | `boolean` / `KeyboardKeysOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      keyboardKeys: true,
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the transform. There is no client JavaScript and no runtime platform sniffing.

## Authoring

Write `{kbd:Ctrl+K}` or `{kbd:Cmd Shift P}`. `+` and whitespace both split
keys. The renderer emits nested `<kbd>` elements with stable classes:

{kbd:Ctrl+K} {kbd:Cmd Shift P} {kbd:Esc}

```md
Press {kbd:Ctrl+K} or {kbd:Cmd Shift P}.
```

Press {kbd:Ctrl+K} or {kbd:Cmd Shift P}.

Single keys, punctuation, and combinations are all valid. Built-in aliases
such as `cmd`, `ctrl`, `shift`, and `esc` normalize only when the feature is
on. Unknown tokens stay as written.

```md
`{kbd:Ctrl+K}`
```

`{kbd:Ctrl+K}`

Escape a literal with a backslash: `\{kbd:Ctrl+K}` stays `{kbd:Ctrl+K}`.
Empty, unclosed, or newline-spanning markup stays visible. Fenced, indented,
and inline code, HTML comments, and raw `code` / `pre` / `script` / `style`
are not rewritten.

## Options

```ts
oxContent({
  keyboardKeys: {
    style: "symbols",
    aliases: {
      cmd: "Cmd",
    },
  },
});
```

| Field     | Type                     | Default   |
| --------- | ------------------------ | --------- |
| `enabled` | `boolean`                | `true`    |
| `style`   | `"words"` / `"symbols"`  | `"words"` |
| `aliases` | `Record<string, string>` | `{}`      |

`style: "words"` turns `cmd` into `Command`. `style: "symbols"` turns it into
`⌘`. Custom `aliases` are matched case-insensitively and override the
built-in table. Labels are chosen at build time.

## Related

- [Inline Badges](./badges.md)
- [Syntax Extensions](./syntax-extensions.md)
- [Built-in Features overview](../built-in-features.md)
