# @ox-content/code-play

Opt-in Code Play plugin for Ox Content. It runs documentation samples on demand
and exposes a headless API plus Headless / preset UI.

Nothing runs until you install this package **and** enable specific languages.

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
```ts play
const n: number = 1;
console.log(n);
```

```rust play typecheck
fn main() {
    println!("ok");
}
```
````

```html
<CodePlay lang="python"> print("hello") </CodePlay>
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

## Security

`play` fences are trusted site content. Do not mark unreviewed or
visitor-supplied snippets as `play`.

- No sample is executed during Markdown transform or SSG.
- JavaScript and TypeScript run in `node:vm` on Node, or in
  `<iframe sandbox="allow-scripts">` in the browser (no `allow-same-origin`).
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
