# Ox Content Examples

This directory contains runnable projects and small source examples.

## Built-in Features

| Example                                | Description                     | Run style            |
| -------------------------------------- | ------------------------------- | -------------------- |
| [builtin-features](./builtin-features) | Small Markdown and config input | Copy into a Vite app |

## Framework Integrations

| Example                        | Description     | Shows                                |
| ------------------------------ | --------------- | ------------------------------------ |
| [integ-vue](./integ-vue)       | Vue 3 + Vite    | Vue component islands in Markdown    |
| [integ-react](./integ-react)   | React + Vite    | React component islands in Markdown  |
| [integ-svelte](./integ-svelte) | Svelte 5 + Vite | Svelte component islands in Markdown |

## Site and Tooling

| Example                                      | Description               | Shows                                |
| -------------------------------------------- | ------------------------- | ------------------------------------ |
| [playground](./playground)                   | Browser playground        | Live Markdown preview                |
| [ssg-vite](./ssg-vite)                       | Vite static site          | SSG output and generated routes      |
| [gen-source-docs](./gen-source-docs)         | Source documentation      | API docs generated from TypeScript   |
| [og-image-custom](./og-image-custom)         | Custom OG image templates | React, Svelte, Vue, and TS templates |
| [incremental-html-js](./incremental-html-js) | Incremental rendering     | Plain Node.js, HTML, CSS, and SSE    |

## Parser and Pipeline Plugins

| Example                                                  | Description        | Shows                                     |
| -------------------------------------------------------- | ------------------ | ----------------------------------------- |
| [plugin-markdown-it](./plugin-markdown-it)               | markdown-it plugin | Parser reuse in markdown-it               |
| [plugin-rehype](./plugin-rehype)                         | rehype plugin      | HTML post-processing with unified         |
| [unplugin-vite-ox-content](./unplugin-vite-ox-content)   | Vite bridge        | Native parser plus unified-compatible AST |
| [unplugin-vite-markdown-it](./unplugin-vite-markdown-it) | Vite bridge        | markdown-it token bridge                  |
| [unplugin-vite-rehype](./unplugin-vite-rehype)           | Vite bridge        | rehype transform pipeline                 |
| [unplugin-vite-remark](./unplugin-vite-remark)           | Vite bridge        | remark transform pipeline                 |
| [unplugin-esbuild](./unplugin-esbuild)                   | esbuild plugin     | Markdown imports in esbuild               |
| [unplugin-rollup](./unplugin-rollup)                     | Rollup plugin      | Markdown imports in Rollup                |
| [unplugin-webpack](./unplugin-webpack)                   | webpack plugin     | Markdown imports in webpack               |

## Running

Most examples are workspace packages. Run them from the repository root:

```bash
corepack pnpm --filter ./examples/integ-vue dev
corepack pnpm --filter ./examples/ssg-vite build
corepack pnpm --filter ./examples/plugin-markdown-it start
```

`builtin-features` is a source catalog instead of a standalone app. Copy the
Markdown or config file you want into `ssg-vite` or a framework integration.

## Adding Examples

1. Prefer a small, focused directory under `examples/`.
2. Add a short `README.md` when the example is not self-explanatory.
3. Add runnable examples to the pnpm workspace with a `package.json`.
4. Add copyable snippets without a `package.json` when a full app would obscure the feature.
5. Add the new entry to this README.
