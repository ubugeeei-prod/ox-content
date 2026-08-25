# Built-in MDX example

Vite + `@ox-content/vite-plugin` SSG app with real `.mdx` pages.

`mdx` is omitted in `vite.config.ts` on purpose:

- `.mdx` files parse JSX, module-level `import` / `export`, and `{expression}`
- sibling `.md` files stay CommonMark + GFM
- there is no framework plugin, so PascalCase tags stay static
  `data-ox-island` placeholders

From the repository root:

```bash
corepack pnpm --filter ./examples/mdx dev
```

```bash
corepack pnpm --filter ./examples/mdx typecheck
corepack pnpm --filter ./examples/mdx build
corepack pnpm --filter ./examples/mdx preview
```

## Pages

| File                      | Shows                                                                   |
| ------------------------- | ----------------------------------------------------------------------- |
| `src/content/index.mdx`   | Static HTML, island placeholder, ESM omitted, expressions not evaluated |
| `src/content/islands.mdx` | Lowercase tags as HTML vs PascalCase `data-ox-island`                   |
| `src/content/plain.md`    | Sibling `.md` that does **not** parse JSX                               |

After `build`, inspect `dist` for `data-ox-island="NoteCard"`, lowercase
`<note>` markup, and the absence of `import` / `export` from the `.mdx` pages.
