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
app demonstrate the browser UI. The docs site keeps JavaScript and TypeScript
enabled; the standalone app also enables Rust and Go, and renders Python with
an explicit Piston-compatible endpoint when one is configured. Rust, Go, and
remote languages stay off unless you opt in.

## Install

<pm>npm install @ox-content/code-play@alpha</pm>

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
        rust: true,
        go: true,
        python: { endpoint: "https://piston.example/api/v2/piston" },
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
Pages without a matching Code Play block do not receive the hydration script,
and builds that never use Code Play do not emit `ox-code-play.js`.

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
```ts play typecheck play-title="Strict TypeScript" play-strict=false play-target=ESNext
const n: number = 1;
console.log(n);
```

```rust play typecheck play-title="Release-mode Rust" play-mode=release
fn main() {
    println!("ok");
}
```

```go play typecheck play-title="Go vet off" play-withVet=false
package main

import "fmt"

func main() {
    fmt.Println("ok")
}
```

```python play play-title="Python via Piston"
print("ok")
```
````

`play-title` labels the widget. `play-compact` / `play-headless` override
the UI preset for one sample, `play-timeout=2500` overrides the timeout, and
`play-viewers=stdio,stderr,-timing` toggles viewers. `play-<config-key>=...`
sets one language config value for that sample, so TypeScript can use
`play-strict=false`, Rust can use `play-edition=2021`, and Go can use
`play-withVet=false`. Python and other remote languages need their
`languages.<id>.endpoint` set at plugin configuration time.

HTML / MDX form:

```html
<CodePlay lang="ts" title="Loose TS" typecheck ui="compact" config-strict="false">
  const n = 1;
</CodePlay>
```

Project-level examples opt in per sample with `play-project` or `project`.
The current fence stays the primary executable snippet, while project metadata
adds file names, provider choice, and an external fallback link:

````md
```ts play play-project=stackblitz play-file=src/main.ts play-entry=src/main.ts play-files=package.json,src/App.tsx play-project-url=https://stackblitz.com/edit/example
console.log("project");
```
````

`play-file` names the current fence inside the project. `play-files` is a
comma-separated list of extra files, resolved relative to the Markdown source
file and confined to `srcDir`. Supported provider metadata adapters are
`stackblitz`, `codesandbox`, `webcontainer`, and `external`. Code Play does
not load provider scripts; the generated widget renders project metadata and
an **Open** fallback link when a safe `http(s)` URL is supplied.

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
viewer edits. `session.cancel()` aborts an in-flight run or typecheck and
returns `status: "cancelled"`. The default toolbar shows **Cancel** while a
run is busy. Inject `transport` (for example `createMemoryTransport`) in
tests so CI never hits a live playground.

| Field             | Meaning                                                              |
| ----------------- | -------------------------------------------------------------------- |
| `run.status`      | `ok` / `error` / `offline` / `timeout` / `cancelled` / `unsupported` |
| `run.stdio`       | Timestamped `stdin` / `stdout` / `stderr` events                     |
| `run.stdout`      | Concatenated stdout text                                             |
| `run.stderr`      | Concatenated stderr text                                             |
| `run.diagnostics` | Compiler / runtime messages with optional line/col                   |
| `run.provenance`  | Where it compiled and where it ran                                   |
| `run.timing`      | Phase durations and `totalMs`                                        |
| `run.preview`     | Framework iframe `srcdoc` when the backend is UI                     |
| `session.stdout`  | Same as `lastResult.stdout`                                          |
| `session.stderr`  | Same as `lastResult.stderr`                                          |

Custom UIs can use the exported `RunActionState` helpers:
`idleRunActionState()`, `runningRunActionState(action)`, and
`resultRunActionState(action, result)`. Transport and CORS failures use
`status: "offline"` so they can be styled separately from compiler/runtime
errors.

## UI

| Preset     | Behavior                                                        |
| ---------- | --------------------------------------------------------------- |
| `default`  | Toolbar plus stdio / stderr / config / provenance / timing tabs |
| `compact`  | Run / type-check plus stdio and stderr                          |
| `headless` | No DOM chrome; use the session API                              |

Viewers can be toggled independently through `viewers`. The hydrated widget
exposes a polite status region, `aria-busy`, tab panels, and arrow-key tab
navigation.

## Languages

| Languages                 | Execute | Type-check | Backend                                     |
| ------------------------- | ------- | ---------- | ------------------------------------------- |
| TypeScript                | yes     | yes        | local strip-types + `tsgo` + `node:vm`      |
| Rust                      | yes     | yes        | `play.rust-lang.org` (or `endpoints.rust`)  |
| Go                        | yes     | yes        | `play.golang.org` (or `endpoints.go`)       |
| JavaScript                | yes     | no         | `node:vm` / sandbox iframe                  |
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

Static hosts do not serve `POST /__ox-code-play/typecheck`. TypeScript
**Run** still works in the browser (strip types, then a sandboxed iframe).
The **Typecheck** button is omitted from published widgets unless you set a
reachable `endpoints.typecheck`. The Vite proxy path is used only during
`vite dev`.

Rust and Go on a published page call `endpoints.rust` / `endpoints.go`
directly from the browser. Official playgrounds may reject that as CORS;
keep the Vite proxy for local docs, or point `endpoints` at an executor you
control.

During `vite dev`, Code Play payloads use `/__ox-code-play/rust` and
`/__ox-code-play/go` by default when `proxy` is enabled. Explicit
`endpoints.rust` and `endpoints.go` values are preserved. Production builds
embed the configured endpoints instead, so static hosts do not depend on the
dev middleware.

Python has no bundled public executor. Configure a Piston-compatible
`languages.python.endpoint` that you operate or trust. If Python is enabled
without an endpoint, the widget still renders, but **Run** returns
`status: "unsupported"` with an endpoint diagnostic instead of silently doing
nothing.

## Security

`play` fences are **trusted site content**, same as any other script you
ship. Do not mark visitor-supplied or unreviewed snippets as `play`.

- Samples are not executed during Markdown transform or SSG.
- **JavaScript / TypeScript execute** in `node:vm` on Node, or in
  `<iframe sandbox="allow-scripts">` in the browser (no `allow-same-origin`).
  They are never run with page-origin `Function`. The sample cannot read the
  host page's DOM or storage.
- **Vue / React / Svelte / Solid** previews use the same iframe flags and
  `srcdoc`. Preview runtimes load from `esm.sh`.
- `sh` never spawns a local shell on the docs host.
- **Rust / Go** POST source to `play.rust-lang.org` / `play.golang.org`
  (or your `endpoints` override). Those hosts see the sample and their
  privacy policy applies.
- A Piston-compatible `languages.<id>.endpoint` receives source for that
  language. Only set HTTPS endpoints you trust, without embedded
  credentials.
- Project sandbox payloads embed trusted source for the current fence and any
  `play-files` entries. Extra files must be relative paths under the Markdown
  source root; symlink real paths are checked before embedding, missing or
  oversized files become widget warnings, and provider URLs are limited to
  `http(s)` without credentials.

## First publish

`@ox-content/code-play` is new on npm. Trusted publishing cannot create the
package, so a maintainer publishes **once** from a laptop, then registers the
trusted publisher with `npm trust`. Commands live in
[Release Operations](../release.md#first-time-npm-publishing).

See the [Code Play roadmap](../code-play-roadmap.md) for follow-up PRs.
