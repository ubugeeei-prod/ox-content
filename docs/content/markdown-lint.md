---
title: Markdown lint
description: Configure native structure checks, opt-in prose rules, safe fixes, and parallel linting.
---

# Markdown lint

The Rust engine checks Markdown structure and visible prose using Ox Content's
AST. Setext headings, nested lists, block quotes, and MDX are supported. Code,
frontmatter, HTML blocks, link destinations, autolinks, and MDX expressions are
excluded from prose checks. Literal unresolved reference links remain prose.

```sh
vpx oxct lint
vpx oxct lint --strict --threads 8
vpx oxct lint --strict --fix
vpx oxct lint --config lint.json --format json
```

The CLI discovers `.oxlint.json`, then `oxlint.json`, in the current directory.
`--config` overrides discovery. Explicit paths override `include`.
`--strict` enables optional structural/style rules; explicit rule switches win.
Spelling requires `--spellcheck` or `rules.spellcheck: true` in the CLI.

## Structure rules

| Option                     | Rule ID                       | Default                          |
| -------------------------- | ----------------------------- | -------------------------------- |
| `duplicateHeadings`        | `duplicate-heading`           | on                               |
| `headingIncrement`         | `heading-increment`           | on                               |
| `emptyHeadings`            | `empty-heading`               | on                               |
| `codeFenceClosed`          | `code-fence-closed`           | on                               |
| `emptyLinks`               | `empty-link`                  | on                               |
| `firstHeadingH1`           | `first-heading-h1`            | off; enabled by `--strict`       |
| `singleH1`                 | `single-h1`                   | off; enabled by `--strict`       |
| `codeFenceLanguage`        | `code-fence-language`         | off; enabled by `--strict`       |
| `imageAlt`                 | `image-alt`                   | off; enabled by `--strict`       |
| `finalNewline`             | `final-newline`               | off; enabled by `--strict`       |
| `maxConsecutiveBlankLines` | `max-consecutive-blank-lines` | 1                                |
| `trailingSpaces`           | `trailing-spaces`             | on; preserves hard breaks        |
| `repeatedWords`            | `repeated-word`               | on                               |
| `repeatedPunctuation`      | `repeated-punctuation`        | on                               |
| `spellcheck`               | `spellcheck`                  | CLI/editor: off; content API: on |

## Opt-in prose rules

Every `textRules` field is disabled when omitted. These native rules follow
ideas from textlint's [sentence-length](https://github.com/textlint-rule/textlint-rule-sentence-length)
and [no-exclamation-question-mark](https://github.com/textlint-rule/textlint-rule-no-exclamation-question-mark)
rules. They have their own configuration and do not load textlint plugins or
`.textlintrc`. Existing editor textlint integration remains available for plugins.

Create `.oxlint.json`:

```json
{
  "include": ["content/**/*.{md,mdx}"],
  "ignore": ["content/generated/**"],
  "rules": { "finalNewline": true, "imageAlt": true },
  "textRules": {
    "sentenceLength": 100,
    "maxTen": 3,
    "noExclamationQuestionMark": true,
    "noTodo": true,
    "terminology": [
      { "term": "Javascript", "replacement": "JavaScript" },
      { "term": "表記ゆれ", "replacement": "表記揺れ" }
    ]
  },
  "severities": {
    "sentence-length": "error",
    "no-todo": "error",
    "terminology": "warning"
  }
}
```

`sentenceLength` counts Unicode code points in each sentence, including spaces
and soft line breaks, while excluding syntax and code. It splits on `.`, `。`,
`!`, `?`, `！`, and `？` outside Japanese quotation marks and parentheses;
decimal dots do not split a sentence. This is a lightweight splitter, without
morphological analysis or textlint's full sentence-splitter behavior.
`maxTen` counts `、` per sentence. `noTodo` checks uppercase `TODO` at word
boundaries. Terminology matches literal, case-sensitive phrases; ASCII terms
respect word boundaries. Rules also inspect headings, tables, and quotations.

## Safe fixes and suppression

`--fix` removes unnecessary trailing whitespace and excess blank lines,
removes adjacent repeated words, inserts a configured final newline, and applies
literal terminology replacements that cannot introduce Markdown syntax.
Two spaces that produce a hard break and existing CRLF line endings survive.
Spelling suggestions and structural rewrites require review and are not applied.

Files are replaced atomically with their permissions preserved. Symlinks and
files changed since reading are refused. `--fix` cannot be combined with stdin.
The report describes the resulting document and includes `fixedCount` edits.
Remaining errors or warnings above `--max-warnings` return exit code 1.

Use HTML comments to suppress all rules or selected rule IDs:

```text
<!-- oxlint-disable-next-line sentence-length -->
This deliberately long sentence is exempt.

<!-- oxlint-disable terminology, no-todo -->

TODO: Preserve this quoted terminology.
<!-- oxlint-enable terminology, no-todo -->
```

Empty rule lists apply to all rules. Directives inside code are ignored.
Suppressed diagnostics do not contribute fixes. `severities` accepts `off`,
`info`, `warning`, or `error`; errors always fail the CLI, while info does not.

## JavaScript and editors

```ts
import { lintMarkdown, fixMarkdown } from "@ox-content/vite-plugin";

const options = {
  rules: { spellcheck: false, finalNewline: true },
  textRules: { terminology: [{ term: "Javascript", replacement: "JavaScript" }] },
};
const diagnostics = lintMarkdown("Javascript", options);
const fixed = fixMarkdown("Javascript", options);
console.log(fixed.output); // "JavaScript\n"
```

Diagnostic lines and UTF-16 columns are one-based. Fix ranges use UTF-8 byte
offsets; use `fixMarkdown` to apply them to JavaScript strings safely.
`lintMarkdownAsync` and file APIs retain opt-in CSpell standard dictionaries;
`fixMarkdown` uses native rules and rejects external dictionaries.

For the LSP, add `markdownLint` options to `.ox-content.json` (without CLI
`include`/`ignore` fields). Its presence enables native diagnostics and quick
fixes. `initializationOptions.markdownLint` accepts the same native options;
`markdownLintEnabled` can explicitly enable or disable them. Native LSP spelling
is off by default. VS Code exposes `oxContent.markdownLint.enabled`; leave it
unset to follow workspace configuration. Other LSP editors use the same options.
Rust options express severities as `{ruleId, severity}` entries in an array;
all JSON configurations also accept the rule-ID object shown above.

## Performance verification

CLI workers read and lint files in parallel using Rayon, with batches of at most
128 files. `--threads` sets the worker count (1–256); the default uses available
CPUs. Workers share prepared dictionaries and terminology matchers. No worker
writes stdout: the coordinator sorts diagnostics and serializes borrowed data
through a 64 KiB buffer. Short word tokens use inline string storage.
Line masks and token vectors reuse per-document storage, and fully visible lines
borrow source text directly. UTF-16 position indexes are built only when a
diagnostic needs columns. Single-file and one-thread runs skip worker-pool startup;
JSON diagnostics serialize directly without an intermediate reference array.

The Native CLI workflow measures base/head medians on identical input, allocation
counts and allocated bytes, plus serial/eight-thread CLI wall time including
file discovery, reads, process startup, and clean/8192-diagnostic JSON output.
Base/head result hashes and serial/parallel reports must match. Raw measurements are
published as the `markdown-lint-performance` artifact. Timing and allocation
instrumentation run separately; results include seven warm repetitions.
