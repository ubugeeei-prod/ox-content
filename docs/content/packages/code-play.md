---
title: "@ox-content/code-play"
description: Opt-in API and UI for on-demand documentation sample execution.
---

# @ox-content/code-play

Code Play runs documentation samples on demand. It is a separate plugin:
`@ox-content/vite-plugin` does not enable it, and installing this package does
nothing until you list languages.

The [Code Play example](../examples/code-play.md) on this site enables
JavaScript and TypeScript only. Rust, Go, and remote languages stay off unless
you opt in.

## Install

<pm>npm install @ox-content/code-play</pm>

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        typescript: { execute: true, typecheck: true },
        javascript: true,
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
      srcDir: "content",
    }),
  ],
};
```

## Authoring

Mark a fence with `play`. Add `typecheck` when the language supports it.

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

HTML / MDX form:

```html
<CodePlay lang="python"> print("hello") </CodePlay>
```

## Headless API

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({
  language: "ts",
  code: "const n: number = 1;\nconsole.log(n);",
});

const check = await session.typecheck();
const run = await session.run();

run.stdio; // timestamped stdin / stdout / stderr events
run.stdout; // concatenated stdout text
run.stderr; // concatenated stderr text
run.provenance; // where it compiled, where it ran
run.timing; // phase durations and totalMs
session.config; // editable language config
```

`createCodePlay()` throws if you ask for a language that is not enabled.
`session.setConfig({ strict: false })` updates the same object the config
viewer edits.

## UI

| Preset     | Behavior                                                        |
| ---------- | --------------------------------------------------------------- |
| `default`  | Toolbar plus stdio / stderr / config / provenance / timing tabs |
| `compact`  | Run / type-check plus stdio and stderr                          |
| `headless` | No DOM chrome; use the session API                              |

Viewers can be toggled independently through `viewers`.

## Languages

**Execute and type-check:** TypeScript, Rust, Go.

**On-demand execute:** JavaScript, Vue, React, Svelte, Solid, MoonBit, Java,
Swift, Kotlin, C, C++, Zig, Haskell, OCaml, Python, PHP, Ruby, sh, C#, Elixir,
F#, Lean, Rocq, Clojure, Scheme.

Remote languages need `languages.<id>.endpoint` pointing at a
Piston-compatible executor. Rust and Go default to the official playgrounds.
JavaScript and TypeScript run locally in `node:vm` or a browser iframe.

## Security

- Samples are not executed during Markdown transform or SSG.
- `sh` never spawns a local shell.
- Enabling Rust or Go sends source to `play.rust-lang.org` /
  `play.golang.org` (or your `endpoints` override).
- A configured remote endpoint receives source for that language.

See the [Code Play roadmap](../code-play-roadmap.md) for follow-up PRs.
