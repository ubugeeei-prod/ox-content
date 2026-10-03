---
title: Ox Content CLI
description: Native Markdown linting and content authoring tools.
---

# Ox Content CLI

The Vite plugin includes the `oxct` binary. In a project that already has
`@ox-content/vite-plugin` installed, run:

```bash
vpx oxct lint
vpx oxct lint 'content/**/*.{md,mdx}'
vpx oxct lint --format json
cat content/index.md | vpx oxct lint --stdin
```

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
1. JSON output includes file counts, diagnostics, and elapsed milliseconds.

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
