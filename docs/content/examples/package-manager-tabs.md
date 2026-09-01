---
title: Package Manager Tabs
description: Author one npm command and render it as vp/pnpm/bun/npm/yarn install tabs.
---

# Package Manager Tabs

Ox Content can expand a single `<pm>` block into an accessible tab group with
one tab per package manager — **vp (Vite+), pnpm, bun, npm, and yarn**, in
that order. You write the command once using npm syntax and the renderer
converts it to each package manager's equivalent natively (in Rust, with no
client-side JavaScript required for the tabs themselves).

## Authoring package-manager tabs

Package-manager tabs are opt-in. Enable them before authoring `<pm>` blocks:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        pm: true,
      },
    }),
  ],
};
```

Write a `<pm>` element containing a single npm-style command:

```html
<pm>npm install -D @ox-content/vite-plugin</pm>
```

The renderer expands it into the same tab widget used by `<tabs>`, so styling
and keyboard navigation are consistent. See
`examples/builtin-features/content/tabs.md` for the compact tab source. Each tab
body is a code block with the command converted for that package manager.

## Rendered example

<pm>npm install -D @ox-content/vite-plugin</pm>

## Conversion rules

The command is written using npm syntax and converted to the others. Rendered
tabs use the same vp-first order:

| vp                    | pnpm                | bun                | npm                    | yarn                    |
| --------------------- | ------------------- | ------------------ | ---------------------- | ----------------------- |
| `vp install`          | `pnpm install`      | `bun install`      | `npm install`          | `yarn`                  |
| `vp install <pkg>`    | `pnpm add <pkg>`    | `bun add <pkg>`    | `npm install <pkg>`    | `yarn add <pkg>`        |
| `vp install <pkg>`    | `pnpm add <pkg>`    | `bun add <pkg>`    | `npm i <pkg>`          | `yarn add <pkg>`        |
| `vp install -D <pkg>` | `pnpm add -D <pkg>` | `bun add -D <pkg>` | `npm install -D <pkg>` | `yarn add -D <pkg>`     |
| `vp install -g <pkg>` | `pnpm add -g <pkg>` | `bun add -g <pkg>` | `npm install -g <pkg>` | `yarn global add <pkg>` |
| `vp uninstall <pkg>`  | `pnpm remove <pkg>` | `bun remove <pkg>` | `npm uninstall <pkg>`  | `yarn remove <pkg>`     |
| `vp run <script>`     | `pnpm run <script>` | `bun run <script>` | `npm run <script>`     | `yarn <script>`         |
| `vp exec -- <bin>`    | `pnpm dlx <bin>`    | `bunx <bin>`       | `npx <bin>`            | `yarn dlx <bin>`        |

Package versions, scopes (`@scope/pkg`), additional flags, and multiple packages
are preserved. Both `-D`/`--save-dev` and `-g`/`--global` are recognized.

## Synced tab groups (opt-in)

By default each package-manager tab group is independent and works with no
JavaScript. You can **opt in** to syncing so that selecting (for example) pnpm in
one block selects pnpm in every other package-manager block on the page, with the
choice persisted in `localStorage`.

Enable it through the `embeds.pm` option:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      // Synced tab groups are OFF by default.
      embeds: {
        pm: { sync: true },
      },
    }),
  ],
};
```

When syncing is enabled, the group element gains a
`data-ox-tab-group="pkg-manager"` attribute and a tiny dependency-free runtime
keeps the groups in sync. When syncing is **disabled** (the default) the
attribute is not emitted and the widget behaves exactly like a standalone tab
group.

## Notes

- The tab widget itself requires no client-side JavaScript; selection uses CSS
  `:has()`, with a `<details>` fallback for readers without CSS or JS.
- Syncing is the only behavior that uses JavaScript, and it is opt-in.
- Unrecognized commands are passed through unchanged, so authoring never breaks
  silently.
