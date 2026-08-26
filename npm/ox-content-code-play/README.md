# @ox-content/code-play

Opt-in Code Play plugin for Ox Content. It runs documentation samples on demand
and exposes a headless API plus Headless / preset UI.

Nothing runs until you install this package **and** enable specific languages.
Pages without matching `play` blocks do not load `ox-code-play.js`.

## Install

```bash
npm install @ox-content/code-play
```

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        typescript: { execute: true, typecheck: true },
        rust: true,
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
    }),
  ],
};
```

## Authoring

````md
```ts play typecheck play-title="Strict TypeScript" play-strict=false play-target=ESNext
const n: number = 1;
console.log(n);
```

```rust play typecheck play-title="Release-mode Rust" play-mode=release
fn main() {
    println!("ok");
}
```
````

Use `play-title`, `play-compact` / `play-headless`, `play-timeout=2500`,
`play-viewers=stdio,stderr,-timing`, and `play-<config-key>=...` for one
sample. The MDX-style `<CodePlay>` tag accepts `title`, `ui`, `timeout`,
`viewers`, and `config-*` attributes.

```html
<CodePlay lang="ts" title="Loose TS" typecheck ui="compact" config-strict="false">
  const n = 1;
</CodePlay>
```

## Headless API

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({ language: "ts", code: "const n: number = 1;" });
const check = await session.typecheck();
const run = await session.run();

run.stdio;
run.stdout;
run.stderr;
run.provenance;
run.timing;
```

`run.status` is `ok`, `error`, `offline`, `timeout`, `cancelled`, or
`unsupported`. Custom UIs can use the exported `RunActionState` helpers to
model idle, running, result, error, and offline states.

## Security

`play` fences are trusted site content. Do not mark unreviewed or
visitor-supplied snippets as `play`.

- No sample is executed during Markdown transform or SSG.
- JavaScript and TypeScript run in `node:vm` on Node, or in a Web Worker in the
  browser. Browsers without Worker support fall back to
  `<iframe sandbox="allow-scripts">` with no `allow-same-origin`. Snippets are
  never run with page-origin `Function`.
- Published widgets hide **Typecheck** unless `endpoints.typecheck` is set.
  **Cancel** appears while a run is in flight.
- Framework previews use the same iframe flags and load runtimes from esm.sh.
- Rust and Go POST source to the official playgrounds (or your `endpoints`).
  The Vite `/__ox-code-play/*` proxy is **dev-only**.
- Other languages need a Piston-compatible `endpoint` you trust.
- `sh` never spawns a local shell.

## Example

A runnable Vite site lives in [`examples/code-play`](../../examples/code-play):

```bash
corepack pnpm --filter ./examples/code-play dev
```

The docs site also dogfoods JavaScript and TypeScript widgets on
[Code Play](../../docs/content/examples/code-play.md).

See [Code Play](../../docs/content/packages/code-play.md) in the docs site.
