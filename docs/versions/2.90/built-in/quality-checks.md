---
title: Quality Checks
description: Lint code fences, type-check TypeScript snippets, run docs as tests, and sanitize HTML during the build.
---

# Quality Checks

Documentation rots when its code samples do. These opt-in checks run during
the Markdown transform, so a broken snippet fails the build instead of
shipping to readers.

| Option               | Default | Checks                                             |
| -------------------- | ------- | -------------------------------------------------- |
| `codeBlockLint`      | `false` | Fence hygiene: missing languages, trailing spaces. |
| `codeBlockTypecheck` | `false` | TypeScript fences compile, via `tsgo`.             |
| `docsTests`          | `false` | Runnable fences pass under Vitest.                 |
| `sanitize`           | `false` | Rendered HTML against an allow list.               |

All checks are static — no documentation code is executed during the
transform. `docsTests` executes code, but only inside the separate Vitest
harness you invoke from CI.

## Code Block Lint

The native scanner reports diagnostics with source positions, and only runs
when a file actually contains fences:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeBlockLint: {
        requireLanguage: true,
        trailingSpaces: true,
        mode: "error",
      },
    }),
  ],
};
```

| Option            | Default  | Purpose                                       |
| ----------------- | -------- | --------------------------------------------- |
| `languages`       | all      | Restrict which fence languages are linted.    |
| `requireLanguage` | `false`  | Report fences without a language identifier.  |
| `trailingSpaces`  | `true`   | Report trailing whitespace inside fences.     |
| `mode`            | `"warn"` | `"warn"` logs; `"error"` fails the transform. |

Given a page with an unlabeled fence and a trailing space, the transform
reports:

```
code-block-language at 3:1
  Code block is missing a language identifier.
code-block-trailing-spaces at 8:13
  Code block line has trailing whitespace.
```

With `mode: "warn"` (the default) diagnostics are logged and the build
continues; with `mode: "error"` the transform throws and the build fails.

## Code Block Type Checking

Type-check TypeScript fences with [tsgo](https://github.com/microsoft/typescript-go),
the native TypeScript compiler:

```ts
oxContent({
  codeBlockTypecheck: {
    languages: ["ts", "tsx"],
    requireMeta: true,
    tsgoCommand: "tsgo",
    mode: "error",
  },
});
```

| Option        | Default         | Purpose                                          |
| ------------- | --------------- | ------------------------------------------------ |
| `languages`   | `["ts", "tsx"]` | Fence languages sent to the compiler.            |
| `requireMeta` | `true`          | Only check fences tagged `typecheck`/`twoslash`. |
| `tsgoCommand` | `"tsgo"`        | Compiler binary to invoke.                       |
| `mode`        | `"warn"`        | `"warn"` logs; `"error"` fails the build.        |

With the default `requireMeta: true`, authors opt in per fence, so
intentionally incomplete snippets stay possible:

````md
```ts typecheck
const value: string = "ok";
```
````

Set `requireMeta: false` to check every TypeScript fence. Snippets are written
to a temporary directory and compiled with `tsgo --noEmit`; compiler errors
become build diagnostics pointing at the source fence.

`tsgo` ships with `@typescript/native-preview`:

<pm>npm install -D @typescript/native-preview</pm>

## Docs Tests

Extract runnable fences and run them under Vitest — the same "documentation
is tested" workflow as Rust doctests. Extraction is native and does not parse
the whole document in JavaScript:

```ts
import { runDocsTests } from "@ox-content/vite-plugin";

await runDocsTests({
  include: ["docs/content/**/*.md"],
  vitestCommand: "vitest",
  vitestArgs: ["run"],
});
```

A fence becomes a test when its meta contains `test`, `runnable`, `vitest`, or
`docs-test`:

````md
```ts docs-test
import { expect } from "vitest";

const result = 1 + 1;
expect(result).toBe(2);
```
````

Each fence is wrapped in a generated Vitest `test(...)`. Switch
`source: "jsdoc"` to run `@example` fences from your source code instead of
Markdown files — the same extractor that powers
[generated API docs](../jsdoc.md). See
[Vitest Docs Tests](../examples/vitest-docs-test.md) for the harness options,
including `executionMode: "module"` for fences that declare their own tests.

## HTML Sanitizer

Sanitize the final HTML when rendering untrusted or mixed content:

```ts
oxContent({
  sanitize: {
    allowedUrlSchemes: ["http", "https", "mailto"],
  },
});
```

Given this input:

```html
<p onclick="alert(1)">
  Hello
  <script>
    alert(2);
  </script>
  <a href="javascript:alert(3)">link</a>
</p>
```

the sanitizer emits:

```html
<p>Hello <a>link</a></p>
```

Script elements, event-handler attributes, and unsafe URL schemes are removed
in a single native pass. `sanitize: true` uses safe defaults that keep common
documentation HTML, including media elements (`video`, `audio`, `source`,
`track`, `picture`). Passing `allowedTags`, `allowedAttributes`, or
`allowedUrlSchemes` **replaces** the corresponding built-in list rather than
extending it.

The pass runs at the very end of the pipeline — after
[embeds](./embeds.md) expand — so allow lists apply to everything the page
actually ships.

## Related

- [Code Blocks](./code-blocks.md) — highlighting, annotations, and imports
  for the fences these checks guard.
- [HTML Sanitizer example](../examples/html-sanitizer.md)
