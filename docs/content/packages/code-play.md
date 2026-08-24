---
title: "@ox-content/code-play"
description: Opt-in API and UI for on-demand documentation sample execution.
---

# @ox-content/code-play

Code Play runs documentation samples on demand. It is a separate plugin:
`@ox-content/vite-plugin` does not enable it, and installing this package does
nothing until you list languages.

The [docs example](../examples/code-play.md) on this site and the standalone
[`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play)
app enable JavaScript and TypeScript only. Rust, Go, and remote languages stay
off unless you opt in.

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

The plugin is a second opt-in layer on top of the package install. A fence
without `play`, or a language that is not listed, stays an ordinary highlighted
block.

## Plugin options

| Option      | Type                                            | Default   | Role                                     |
| ----------- | ----------------------------------------------- | --------- | ---------------------------------------- |
| `languages` | `Record<string, true \| LanguageEnableOptions>` | `{}`      | Enable execute / typecheck / `endpoint`  |
| `ui`        | `"default" \| "compact" \| "headless"`          | `default` | Chrome around the sample                 |
| `viewers`   | `Partial<ViewerFlags>`                          | all on    | Show or hide stdio / stderr / config / … |
| `timeoutMs` | `number`                                        | `10000`   | Per-run timeout                          |
| `endpoints` | `{ rust?, go?, typecheck? }`                    | official  | Playground / typecheck URLs              |
| `proxy`     | `boolean`                                       | `true`    | Mount Vite **dev** `/__ox-code-play/*`   |
| `srcDir`    | `string`                                        | `"docs"`  | Markdown root used to match play fences  |
| `outDir`    | `string`                                        | Vite out  | Written HTML to enhance after SSG        |
| `base`      | `string`                                        | `"/"`     | Public path for `ox-code-play.js`        |

`LanguageEnableOptions` accepts `execute`, `typecheck`, `endpoint`, and
`config` overrides for that language's schema (TypeScript `strict`, Rust
`crateType`, Go `withVet`, …).

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
viewer edits. Inject `transport` (for example `createMemoryTransport`) in tests
so CI never hits a live playground.

| Field             | Meaning                                                  |
| ----------------- | -------------------------------------------------------- |
| `run.status`      | `ok` / `error` / `timeout` / `cancelled` / `unsupported` |
| `run.stdio`       | Timestamped `stdin` / `stdout` / `stderr` events         |
| `run.stdout`      | Concatenated stdout text                                 |
| `run.stderr`      | Concatenated stderr text                                 |
| `run.diagnostics` | Compiler / runtime messages with optional line/col       |
| `run.provenance`  | Where it compiled and where it ran                       |
| `run.timing`      | Phase durations and `totalMs`                            |
| `run.preview`     | Framework iframe `srcdoc` when the backend is UI         |
| `session.stdout`  | Same as `lastResult.stdout`                              |
| `session.stderr`  | Same as `lastResult.stderr`                              |

## UI

| Preset     | Behavior                                                        |
| ---------- | --------------------------------------------------------------- |
| `default`  | Toolbar plus stdio / stderr / config / provenance / timing tabs |
| `compact`  | Run / type-check plus stdio and stderr                          |
| `headless` | No DOM chrome; use the session API                              |

Viewers can be toggled independently through `viewers`.

## Languages

| Languages                 | Execute | Type-check | Backend                                     |
| ------------------------- | ------- | ---------- | ------------------------------------------- |
| TypeScript                | yes     | yes        | local strip-types + `tsgo` + `node:vm`      |
| Rust                      | yes     | yes        | `play.rust-lang.org` (or `endpoints.rust`)  |
| Go                        | yes     | yes        | `play.golang.org` (or `endpoints.go`)       |
| JavaScript                | yes     | no         | `node:vm` / `Function`                      |
| Vue, React, Svelte, Solid | yes     | no         | iframe `srcdoc` + esm.sh import map         |
| Python, PHP, Ruby, sh, …  | yes     | no         | Piston-compatible `languages.<id>.endpoint` |

The full catalog is the same list as the [roadmap](../code-play-roadmap.md).
Aliases such as `ts`, `c++`, `bash`, and `coq` resolve to the canonical id.

## Playground proxies

Vite **dev server** only. `codePlay({ proxy: true })` (the default) mounts:

| Path                             | Forwards to                                                     |
| -------------------------------- | --------------------------------------------------------------- |
| `POST /__ox-code-play/rust`      | `endpoints.rust` (default `https://play.rust-lang.org/execute`) |
| `POST /__ox-code-play/go`        | `endpoints.go` (default `https://play.golang.org/compile`)      |
| `POST /__ox-code-play/typecheck` | local `tsgo` (no remote compiler)                               |

These routes accept **POST** only, cap the body at 256 KiB, and refuse
non-`http(s)` destinations or URLs with embedded credentials. Upstream
failures return generic JSON `{ "error": "..." }` and do not leak fetch
details.

The proxy is not installed in production SSG output. Set `endpoints` to the
official playgrounds (or your own HTTPS executor) for published pages, or
`proxy: false` if you do not want the dev middleware.

## Security

- Samples are not executed during Markdown transform or SSG.
- JavaScript and TypeScript execute in `node:vm` on Node, or in
  `<iframe sandbox="allow-scripts">` in the browser (no `allow-same-origin`).
- `sh` never spawns a local shell.
- Enabling Rust or Go sends source to `play.rust-lang.org` /
  `play.golang.org` (or your `endpoints` override).
- A configured remote endpoint receives source for that language.

## First publish

`@ox-content/code-play` is new on npm. Trusted publishing cannot create the
package, so a maintainer publishes **once** from a laptop, then adds the
trusted publisher. Commands and the exact npmjs.com fields live in
[Release Operations](../release.md#first-time-npm-publishing).

See the [Code Play roadmap](../code-play-roadmap.md) for follow-up PRs.
