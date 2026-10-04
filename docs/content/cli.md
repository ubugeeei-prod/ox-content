---
title: Ox Content CLI
description: Native Markdown linting and content authoring tools.
---

# Ox Content CLI

Run the standalone CLI with `vpx oxct`, including outside an existing project.
The Vite plugin also includes the `oxct` binary for installed projects.

Project creation, IDE setup, Markdown linting, and the terminal viewer are
implemented in Rust. npm launchers call the same native implementation through
the bundled NAPI package, so no separate Rust installation is required.
Vite configuration loading and JavaScript validation hooks run in the Node.js
integration layer.

From a source checkout, the authoring commands can also run directly:

```sh
cargo run -p ox_content_cli --bin oxct -- lint
cargo run -p ox_content_cli --bin oxct -- new my-docs --yes --no-install
```

```bash
vpx oxct new
vpx oxct ide install
vpx oxct lint
vpx oxct lint 'content/**/*.{md,mdx}'
vpx oxct lint --format json
cat content/index.md | vpx oxct lint --stdin
```

## Create a project

`vpx oxct new` guides you through a directory, documentation / blog / minimal
template, theme skin, color palette, package manager, and dependency installation.
The generated project includes Vite configuration, Markdown examples, scripts,
and a TypeScript configuration. Existing nonempty directories are preserved.

For automation:

```bash
vpx oxct new my-docs --yes --template docs --skin editorial --palette nord --no-install
vpx oxct new my-blog --yes --template blog --package-manager pnpm --install
```

Without a terminal, setup uses defaults and does not install dependencies unless
you pass `--install`. Installation failures preserve the project so you can retry.

## Set up an IDE

`vpx oxct ide install` lets you select VS Code, Cursor, Windsurf, VSCodium, Zed,
or Neovim, then choose extension installation, workspace configuration, or both.
It shows the affected files and commands before applying the plan.
Existing JSONC comments and unrelated settings are preserved; changed files get
a backup alongside the original.

```bash
vpx oxct ide install --ide vscode --config-only --yes
vpx oxct ide install --ide cursor --ide zed --dry-run
vpx oxct ide install --ide neovim --yes
```

VS Code family installations use the IDE's CLI, which must be on PATH. Zed setup
enables `auto_install_extensions` in user settings; Zed installs the extension
on its next launch when available in the registry. Neovim setup installs the
bundled plugin into its native pack directory and generates
`.ox-content/neovim.lua`. Source it with `:luafile .ox-content/neovim.lua` on
Neovim 0.11+. Existing plugin installations are preserved for your plugin manager
to update. `--extensions-only` and `--config-only` keep the two actions separate.

## Markdown lint

See [Markdown lint](./markdown-lint.md) for structural rules, opt-in prose rules,
safe fixes, suppression comments, and parallel performance measurements.

`oxct lint` discovers Markdown, MDC, and MDX files in the current directory. The
native Rust engine checks headings, repeated words, punctuation, blank lines,
and trailing whitespace in bounded batches. MDX syntax is masked when checking
`.mdx` files. Dependencies, Git metadata, build output, and Rust build artifacts
are excluded by default.

Spelling is opt-in with `--spellcheck`. This uses the native built-in dictionary;
it does not load external dictionary datasets. Use the JavaScript lint API when
you need CSpell dictionary packages.

The default warning budget is zero: diagnostics return exit code 1. Set
`--max-warnings 10` to allow up to ten warnings. A clean run returns 0; invalid
arguments, unreadable files, invalid configuration, and unmatched inputs return
exit code 1. JSON output includes file counts, diagnostics, and elapsed milliseconds.

Pass a JSON configuration with `--config lint.json`:

```json
{
  "include": ["content/**/*.md"],
  "ignore": ["content/generated/**"],
  "rules": {
    "spellcheck": false,
    "repeatedWords": true,
    "maxConsecutiveBlankLines": 1
  }
}
```

Explicit file arguments override `include`; `--ignore` adds to configured ignore
patterns. Native custom dictionaries accept `words`, `ignoredWords`, and a
`byLanguage` array with `{ "language": "en", "words": ["OxContent"] }` entries.
For stdin, `--stdin-filepath article.mdx` selects MDX checking and sets the
filename in diagnostics.

## Terminal Markdown reader

`vpx oxct tui` opens a full-screen reader with a file sidebar, heading outline,
local Markdown links, styled GFM tables and task lists, code blocks, and a search
bar. It wraps Japanese text and emoji by display width and follows editor saves.

```bash
vpx oxct tui content
vpx oxct tui README.md --theme light
vpx oxct tui 'content/**/*.md' --no-watch
cat README.md | vpx oxct tui --stdin --width 80
```

Use Tab to switch between reader, files, outline, and links. Arrow keys or `j` / `k`
move, Enter opens the selection, `/` searches, and `n` / `N` finds the next or
previous match. `b` toggles zen mode, `t` changes theme, `r` reloads, and `?` shows
help. Quit with `q` or Ctrl-C; the previous terminal screen and input mode are restored.
Narrow terminals show navigation as a full-width pane.

`--print` renders a document to stdout; non-interactive output uses this mode
automatically. `--no-color` removes colors. The viewer strips control characters
from Markdown, displays external links as text, and reads documents up to 4 MiB.

## Frontmatter types and diagnostics

Define ordered globs relative to `srcDir` in your Vite configuration. Each validator
implements [Standard Schema](https://standardschema.dev/), so Zod, Valibot, ArkType,
or a custom implementation can provide validation and inferred TypeScript types.
The first matching glob wins; unmatched documents keep their existing behavior.

```ts
import { defineConfig } from "vite";
import {
  oxContent,
  defineFrontmatterSchemas,
  type InferFrontmatter,
} from "@ox-content/vite-plugin";
import { z } from "zod";

export const schemas = defineFrontmatterSchemas({
  "posts/**/*.md": z.object({
    title: z.string(),
    category: z.enum(["guide", "news"]),
    draft: z.boolean().default(false),
  }),
  "guides/**/*.{md,mdc}": z.object({ title: z.string(), order: z.number() }),
});

export type PostFrontmatter = InferFrontmatter<typeof schemas, "posts/**/*.md">;
// { title: string; category: "guide" | "news"; draft: boolean }

export default defineConfig({
  plugins: [oxContent({ srcDir: "content", frontmatterSchemas: schemas })],
});
```

Run `vpx oxct typecheck` to check all matching Markdown/MDC/MDX documents without
building the site, or `vpx oxct typecheck 'content/posts/**/*.md' --format json`.
Use `--config path/to/vite.config.ts` for an alternate configuration. Schema errors
report YAML line/column positions and fail both typecheck and the Vite build.
Async refinements execute in both paths. Defaults and transforms become the
frontmatter returned by the Vite transform; they do not rewrite source files.
`InferFrontmatterInput` describes the YAML input before those defaults/transforms.

`vpx oxct ide install` configures project schema support. The VS Code extension
automatically uses the installed workspace plugin after Workspace Trust is granted;
`oxContent.frontmatter.projectValidation` can disable it. Explicit server paths
continue to take precedence. Other LSP clients can run `vpx oxct lsp --project`.
This mode executes Vite configuration and validators, and should be enabled only
for a trusted project. Plain `vpx oxct lsp` uses the bundled Rust server without
evaluating project configuration.

IDE diagnostics run the same async validators as builds and update on document or
configuration changes. Completion and hover use the validator's **input** Standard
JSON Schema conversion (supported by Zod 4). When a library has no converter or a
schema cannot be converted, provide a shape explicitly:

```ts
defineFrontmatterSchemas({
  "posts/**/*.md": {
    schema: myStandardSchemaValidator,
    jsonSchema: {
      type: "object",
      required: ["title"],
      properties: { title: { type: "string", description: "Display title" } },
    },
  },
});
```

The shape supplies key, enum, boolean, nested-object and array-item completions,
descriptions and local JSON Schema references. Full validation always uses the
Standard Schema validator, including constraints not expressible in JSON Schema.
Without a completion shape, diagnostics and TypeScript inference still work.
