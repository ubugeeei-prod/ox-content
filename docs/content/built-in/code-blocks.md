---
title: Code Blocks
description: Syntax highlighting, code annotations, and code imports for fenced code blocks.
---

# Code Blocks

Three opt-in features extend fenced code blocks: tree-sitter syntax
highlighting, annotation syntax for highlighting and diff markers, and
importing snippets from real source files. This site enables all three, so
every example below is rendered live.

On-demand **Run** / **Typecheck** for samples is a separate package,
[`@ox-content/code-play`](../packages/code-play.md). It is not part of
`@ox-content/vite-plugin`. See the [Code Play example](../examples/code-play.md).

| Option            | Type                                 | Default |
| ----------------- | ------------------------------------ | ------- |
| `highlight`       | `boolean`                            | `false` |
| `codeAnnotations` | `boolean` / `CodeAnnotationsOptions` | `false` |
| `codeImports`     | `boolean` / `CodeImportOptions`      | `false` |
| `codeGroups`      | `boolean` / `CodeGroupOptions`       | `false` |

## Syntax Highlighting

Highlighting is opt-in. When enabled, fenced blocks and language-tagged inline
code go through the native tree-sitter engine. Languages with no native
grammar stay as ordinary `<pre><code>` — they are not highlighted.
Adjacent formats without their own bundled grammar use best-effort aliases
when the existing grammar keeps source text safe: `jsonc` / `json5` /
`webmanifest` use JSON, `vue` / `svelte` / `astro` / `angular` use HTML,
`flow` / `javascriptreact` use JavaScript, and `typescriptreact` uses TSX.
Dotfile and config tags such as `dotenv`, `.env`, `gitignore`, `npmrc`,
`ini`, and `conf` render as escaped plain text.

### Supported languages

Native grammars tokenize the fence tags below. Aliases in a cell resolve to
the same grammar. Vue, Svelte, Astro, and Angular still use the HTML grammar
— crates.io does not currently ship maintained dedicated grammars that match
this tree-sitter line.

| Language   | Fence tags                                                                |
| ---------- | ------------------------------------------------------------------------- |
| TypeScript | `typescript`, `ts`, `cts`, `mts`                                          |
| TSX        | `tsx`, `typescriptreact`                                                  |
| JavaScript | `javascript`, `js`, `cjs`, `mjs`, `jsx`, `javascriptreact`, `flow`, `mdx` |
| Rust       | `rust`, `rs`                                                              |
| JSON       | `json`, `jsonc`, `json5`, `webmanifest`                                   |
| CSS        | `css`                                                                     |
| Less       | `less`                                                                    |
| HTML       | `html`, `vue`, `svelte`, `astro`, `angular`                               |
| XML        | `xml`, `svg`, `xsl`, `xslt`, `rss`, `atom`, `plist`, `xsd`                |
| Python     | `python`, `py`                                                            |
| Go         | `go`, `golang`                                                            |
| Java       | `java`                                                                    |
| C          | `c`, `h`                                                                  |
| C++        | `cpp`, `c++`, `cc`, `hpp`, `cxx`                                          |
| YAML       | `yaml`, `yml`                                                             |
| Markdown   | `markdown`, `md`                                                          |
| Bash       | `bash`, `sh`, `shell`, `zsh`, `shellscript`                               |
| TOML       | `toml`                                                                    |
| WGSL       | `wgsl`                                                                    |
| SQL        | `sql`                                                                     |
| GraphQL    | `graphql`, `gql`                                                          |
| Dockerfile | `dockerfile`, `docker`, `containerfile`                                   |
| Ruby       | `ruby`, `rb`                                                              |
| PHP        | `php`                                                                     |
| Nix        | `nix`                                                                     |
| C#         | `csharp`, `cs`                                                            |
| Swift      | `swift`                                                                   |
| Kotlin     | `kotlin`, `kt`                                                            |
| GLSL       | `glsl`                                                                    |
| Lua        | `lua`                                                                     |
| HCL        | `hcl`, `terraform`, `tf`, `tfvars`                                        |
| Make       | `make`, `makefile`, `mk`                                                  |
| Diff       | `diff`, `patch`, `udiff`                                                  |
| PowerShell | `powershell`, `pwsh`, `ps1`, `psm1`                                       |
| Zig        | `zig`, `zon`                                                              |
| Haskell    | `haskell`, `hs`                                                           |
| Elixir     | `elixir`, `ex`, `exs`                                                     |
| Scala      | `scala`, `sc`, `sbt`                                                      |
| R          | `r`, `rscript`                                                            |

Unknown tags stay ordinary `<pre><code>` — for example `perl`, `elm`,
`assembly`, `asm`, `llvm`, `clojure`, and `brainfuck`. Do not alias those onto
an unrelated grammar. Plain tags such as
`text`, `dotenv`, and `ini` are escaped but not tokenized.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      highlight: true,
    }),
  ],
};
```

Token colors are `--octc-syntax-*` CSS custom properties on
`<pre class="ox-highlight css-variables">`. Highlighting is tree-sitter only,
and `@ox-content/theme-color-*` packages resolve those variables. Without a
color scheme the properties fall back to GitHub Dark. After highlighting, code
block metadata (annotations, line numbers) is merged back into the native
output.

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

### Dense blocks

Dense API pages can opt a block into stable line fragments and wrapping
without custom components:

````md
```ts:line-numbers=27 :line-links=auth-loader :wrap [src/auth/load-user.ts]
export async function loadUserSession(request: Request) {
  const token = request.headers.get("authorization") ?? request.headers.get("x-legacy-auth-token");
  return fetchSession(token);
}
```
````

Rendered:

```ts:line-numbers=27 :line-links=auth-loader :wrap [src/auth/load-user.ts]
export async function loadUserSession(request: Request) {
  const token = request.headers.get("authorization") ?? request.headers.get("x-legacy-auth-token");
  return fetchSession(token);
}
```

`:line-links` adds `id` targets to rendered lines. With an explicit prefix,
line 27 above can be linked as `#auth-loader-L27`; without one, the prefix is
derived from the filename caption when present. `:wrap` wraps long lines inside
the code frame for mobile-heavy docs, while `:no-wrap` keeps the default
horizontal scrolling.

When `ssg.readerChrome.copy` is enabled, blocks whose VitePress inline
directives changed the visible code expose the original fence source to the
copy button. Plain and purely metadata-decorated blocks still copy their
visible code text.

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

## Code Groups

For adjacent JS/TS/shell alternatives, opt in to `codeGroups` and wrap the
fences in `::: code-group` instead of hand-writing `<tabs>`. Titles come
from ` ```ts [label] ` or fence meta. See [Code Groups](./code-groups.md).

## Related

- [Code Groups](./code-groups.md) — VitePress-style grouped fences.
- [Quality Checks](./quality-checks.md) — lint, type-check, and test the code
  blocks themselves.
- [Typed Hover](./typed-hover.md) — build-time TypeScript hover overlays on
  `twoslash` fences.
- [Code Annotations example](../examples/code-annotations.md)
- [Code Imports example](../examples/code-imports.md)
