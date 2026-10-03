---
title: Ox Content CLI
description: Native Markdown linting and content authoring tools.
---

# Ox Content CLI

Run the standalone CLI with `vpx oxct`, including outside an existing project.
The Vite plugin also includes the `oxct` binary for installed projects.

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
