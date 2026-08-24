---
title: Code Play
description: Opt-in on-demand sample execution with stdio, stderr, config, provenance, and timing viewers.
---

# Code Play

This page uses `@ox-content/code-play` with **JavaScript** and **TypeScript**
enabled. Other languages stay ordinary fences until a site opts them in.

A copy-paste Vite app lives at
[`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play)
in the repository.

## Enable the plugin

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        javascript: true,
        typescript: { execute: true, typecheck: true },
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
    }),
  ],
};
```

## Live TypeScript sample

The fence below is marked `play`. Use **Run** to execute it. **Typecheck**
appears during `vite dev` (the `/__ox-code-play/typecheck` proxy) or when
the site sets a reachable `endpoints.typecheck`. Published pages still run
TypeScript by stripping types into the sandbox iframe. The stdio, stderr,
config, provenance, and timing tabs are the same objects the headless API
returns. `console.warn` lands in `run.stderr`.

```ts play typecheck
const message: string = "hello from Code Play";
console.log(message);
console.warn("this warning is a stderr chunk");
```

## Live JavaScript sample

```js play
function add(left, right) {
  return left + right;
}

console.log(add(2, 40));
```

## Typecheck failure

During `vite dev`, **Typecheck** should fail on this sample. On a published
page the button is omitted unless `endpoints.typecheck` is set. **Run** still
executes after types are stripped, so execute and type-check stay separate.

```ts play typecheck
const n: number = "not a number";
console.log(n);
```

## Runtime error

`throw` becomes a diagnostic and a stderr chunk. The stderr tab opens when the
run produces stderr or an error diagnostic.

```js play
console.log("before");
throw new Error("boom from the example");
```

## Headless usage

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({
  language: "ts",
  code: "const n: number = 1;",
});

const result = await session.run();
result.stdio;
result.stdout;
result.stderr;
result.provenance.compile;
result.provenance.execute;
result.timing.phases;
```

`ui: "compact"` hides the tab list and keeps stdio plus stderr. `ui: "headless"`
renders no chrome — use `createCodePlay()` from your own UI.

## Remote languages

Python, Rust, Go, and the rest of the catalog are registered but not enabled
on this page. Give Rust/Go a playground endpoint, or give Python a
Piston-compatible `endpoint`, before marking those fences `play`.

See [@ox-content/code-play](../packages/code-play.md) and the
[roadmap](../code-play-roadmap.md).
