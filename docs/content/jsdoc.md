---
title: API Docs from JSDoc
description: Generate API documentation from JSDoc and TypeScript types, like cargo doc for JavaScript.
---

# API Docs from JSDoc

Ox Content can generate API documentation directly from your JSDoc comments and
TypeScript types — "cargo doc for JavaScript". Source files are parsed with the
[OXC](https://oxc.rs) parser, so extraction is fast and understands real
TypeScript (generics, overloads, interfaces, enums) without a separate
type-checker pass.

It produces, in one run:

- **Markdown pages** for each module, with syntax-highlighted signatures,
  parameter tables, return types, examples, and (optionally) source links.
- **`docs.json`** — a machine-readable payload of the same data for runtime
  tooling.
- **`nav.ts`** — a typed navigation tree you can feed into the sidebar.

## Enable

Docs generation is part of the Vite plugin and is **on by default** (opt-out).
Point it at your source and pick an output directory:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      docs: {
        src: ["./src"],
        out: "docs/api",
        githubUrl: "https://github.com/your/repo",
      },
    }),
  ],
});
```

To turn it off entirely, set `docs: { enabled: false }`.

## Options

| Option                  | Default                                          | Description                                                                         |
| ----------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `enabled`               | `true`                                           | Enable/disable docs generation.                                                     |
| `src`                   | `['./src']`                                      | Source directories to scan.                                                         |
| `out`                   | `'docs/api'`                                     | Output directory for generated docs.                                                |
| `include`               | `['**/*.ts', '**/*.tsx', …]`                     | Glob patterns of files to include.                                                  |
| `exclude`               | `['**/*.test.*', '**/*.spec.*', 'node_modules']` | Glob patterns to exclude.                                                           |
| `entryPoints`           | —                                                | Public API entry points used to group re-exported docs (see below).                 |
| `format`                | `'markdown'`                                     | `'markdown'`, `'json'`, or `'html'`.                                                |
| `private`               | `false`                                          | Include `@private` members.                                                         |
| `internal`              | `false`                                          | Include `@internal` members.                                                        |
| `toc`                   | `true`                                           | Emit a table of contents per file.                                                  |
| `groupBy`               | `'file'`                                         | Group output by `'file'` or `'category'`.                                           |
| `githubUrl`             | —                                                | Repository URL; when set, signatures link to their source lines.                    |
| `linkStyle`             | `'markdown'`                                     | Internal link style: `'markdown'` (`.md` links) or `'clean'` (extensionless).       |
| `basePath`              | `'/api'`                                         | Route prefix for generated links and nav metadata.                                  |
| `pathStrategy`          | `'flat'`                                         | Output layout: `'flat'` or `'typedoc'` (see below).                                 |
| `renderStyle`           | `'html'`                                         | Output renderer: themed HTML-in-Markdown or plain Markdown.                         |
| `propertyMembersFormat` | `'none'`                                         | Display nested object-literal members owned by properties as `'list'` or `'table'`. |
| `typeDeclarationFormat` | `'none'`                                         | Display return type declaration members as `'list'` or `'table'`.                   |
| `generateNav`           | `true`                                           | Emit the `nav.ts` navigation file.                                                  |

## What gets extracted

Each documented declaration becomes an entry with its `kind` (`function`,
`class`, `interface`, `type`, `enum`, `variable`, or `module`), description,
signature, and members. The following JSDoc tags are recognised:

| Tag                    | Effect                                                  |
| ---------------------- | ------------------------------------------------------- |
| `@param name desc`     | Adds a row to the parameter table (type comes from TS). |
| `@returns desc`        | Documents the return value.                             |
| `@example`             | Rendered as a fenced code block.                        |
| `@default value`       | Shown alongside the parameter/property.                 |
| `@deprecated [reason]` | Flags the entry as deprecated.                          |
| `@private`             | Hidden unless `private: true`.                          |
| `@internal`            | Hidden unless `internal: true`.                         |

Types are read from the TypeScript annotations themselves, so `@param {Type}`
JSDoc type syntax is not required.

## Entry points and re-exports

By default, docs are grouped by source file. If your package re-exports its
public API through a barrel (`index.ts`), pass it as an `entryPoint` to group
the docs by what's actually exported, following the re-export graph:

```ts
docs: {
  entryPoints: [
    "./src/index.ts",
    { path: "./src/cli.ts", name: "CLI" },
  ],
}
```

A symbol re-exported under a new name is documented under the name it is
exported as.

## Output layout

`pathStrategy` controls the directory shape of the generated Markdown:

- **`flat`** (default) — one file per module under `out` (e.g.
  `docs/api/index.md`, `docs/api/parser.md`).
- **`typedoc`** — a nested, TypeDoc-style tree that splits modules into
  per-kind subdirectories (`functions/`, `classes/`, `interfaces/`, …), which
  scales better for large APIs.

Alongside the Markdown, `writeDocs` emits `docs.json` (the structured payload)
and, when `generateNav` is on, `nav.ts`. A manifest tracks generated files so
stale pages are cleaned up on the next run.

## Display formats

`renderStyle` controls the generated page body:

- **`html`** (default) — emits ox-content themed HTML inside Markdown files.
- **`markdown`** — emits plain Markdown without raw HTML scaffolding.

Display-format options accept `'none'`, `'list'`, or `'table'`. With
`renderStyle: 'html'`, `propertyMembersFormat` and `typeDeclarationFormat`
render the requested HTML list/table for nested property object literals and
return type declaration members.

## Wire the nav into your sidebar

`nav.ts` exports a typed `NavItem[]` you can splice into your theme's sidebar:

```ts
import { apiNav } from "./docs/api/nav";

// in your ssg theme config
sidebar: [
  ...apiNav,
  // …your hand-written sections
];
```

## See also

- [`examples/gen-source-docs`](./examples/gen-source-docs.md) — a runnable setup.
- The generated [API Reference](./api/index.md) for this project is itself
  produced by this feature.
