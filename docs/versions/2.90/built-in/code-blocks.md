---
title: Code Blocks
description: Syntax highlighting, code annotations, and code imports for fenced code blocks.
---

# Code Blocks

Three opt-in features extend fenced code blocks: Shiki-based syntax
highlighting, annotation syntax for highlighting and diff markers, and
importing snippets from real source files. This site enables all three, so
every example below is rendered live.

| Option            | Type                                 | Default         |
| ----------------- | ------------------------------------ | --------------- |
| `highlight`       | `boolean`                            | `false`         |
| `highlightTheme`  | `string` / `ThemeRegistration`       | `"github-dark"` |
| `highlightLangs`  | `LanguageRegistration[]`             | `[]`            |
| `codeAnnotations` | `boolean` / `CodeAnnotationsOptions` | `false`         |
| `codeImports`     | `boolean` / `CodeImportOptions`      | `false`         |

## Syntax Highlighting

Highlighting is opt-in because it adds Shiki to the build:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      highlight: true,
      // Optional: a Shiki theme name or a full theme registration object.
      highlightTheme: "github-dark",
      // Optional: extra TextMate grammars for custom languages.
      highlightLangs: [],
    }),
  ],
};
```

Every code block on this site is highlighted through this pipeline — this page
uses a custom `highlightTheme` registration to match the site design. After
highlighting, code block metadata (annotations, line numbers) is merged back
into Shiki's output natively.

## Code Annotations

Annotations are opt-in so ordinary fences stay literal unless a site chooses an
annotation syntax:

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    // "attribute" (default) | "vitepress" | "both"
    notation: "both",
    // Attribute name used by the attribute syntax. Default: "annotate".
    metaKey: "annotate",
    // Render line numbers for every block. Default: false.
    defaultLineNumbers: false,
  },
});
```

Supported annotation kinds are `highlight`, `warning`, and `error`.

### Attribute notation

The default notation is a single fence attribute with `kind:lines` groups
separated by `;`. Line selectors accept single lines (`5`) and ranges (`3-4`):

````md
```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```
````

Rendered:

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

### VitePress notation

`notation: "vitepress"` (or `"both"`) enables VitePress-compatible fence
metadata and inline comment directives. The fence meta pieces compose
independently:

- `{1,3}` — highlighted lines.
- `[config.ts]` — a filename label rendered above the block.
- `:line-numbers` / `:line-numbers=7` / `:no-line-numbers` — line numbers per
  block, with an optional start.

````md
```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
const expires = readExpiry(token);
refreshBefore(expires);
```
````

Rendered:

```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
const expires = readExpiry(token);
refreshBefore(expires);
```

Inline comment directives annotate the line they sit on and are removed from
the output. This block is authored with `// [!code warning]` on the second
line and `// [!code error]` on the third:

```ts
const token = readToken();
console.warn("Token expires soon"); // [!code warning]
throw new Error("Token is invalid"); // [!code error]
```

Diff notation uses `// [!code --]` for removed and `// [!code ++]` for added
lines — this block carries them on the two `return` lines:

```ts
export function resolve(id: string) {
  return legacyResolve(id); // [!code --]
  return nativeResolve(id); // [!code ++]
}
```

`// [!code focus]` (or `// [!code focus:3]` for a range) dims everything but
the focused lines.

Inline directives are consumed wherever they appear inside a code block —
including fence examples nested in an outer fence — so use the escape
directive below when a line needs to show annotation-looking text.

### Escaping

A standalone `// [!code escape]` comment is removed from the output and makes
the next line render literally. This block is authored with an escape comment
above the first `console.warn` line, so its `// [!code warning]` survives as
text while the second one becomes an annotation:

```ts
// [!code escape]
console.warn("literal"); // [!code warning]
console.warn("annotated"); // [!code warning]
```

### Custom meta key

Swap `annotate` for a more domain-specific attribute name:

```ts
oxContent({
  codeAnnotations: {
    metaKey: "markers",
  },
});
```

````md
```ts markers="highlight:2;warning:3"
const token = readToken();
refreshToken(token);
console.warn("Token expires soon");
```
````

## Code Imports

Import checked source files into Markdown instead of copy-pasting them:

```ts
oxContent({
  codeImports: {
    // Root for `@/` imports. Defaults to the Vite project root.
    rootDir: process.cwd(),
  },
});
```

The fence language is inferred from the file extension, and imported snippets
go through the same highlighting and annotation pipeline as inline fences.

Writing `<<< @/snippets/greet.ts` on its own line imports the whole file:

<<< @/snippets/greet.ts

A `{1-4}` suffix — `<<< @/snippets/greet.ts{1-4}` — imports a line range:

<<< @/snippets/greet.ts{1-4}

A named suffix — `<<< @/snippets/greet.ts{greet}` — imports the region
delimited by `#region greet` / `#endregion greet` comments, with the markers
themselves stripped:

<<< @/snippets/greet.ts{greet}

Because imports resolve at transform time, editing the source file updates
every page that imports it, and stale docs snippets stop being possible.

`<<<` references are resolved inside fenced code blocks too, so quote the
syntax with inline code (as this page does) when you need to show it
literally.

## Related

- [Quality Checks](./quality-checks.md) — lint, type-check, and test the code
  blocks themselves.
- [Code Annotations example](../examples/code-annotations.md)
- [Code Imports example](../examples/code-imports.md)
