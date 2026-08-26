---
title: Code Play example
description: Minimal Vite site that runs documentation samples on demand.
---

# Code Play example

This site enables **JavaScript** and **TypeScript** only. Mark a fence with
`play`. **Typecheck** shows during `vite dev`, or on a published page if you
set `endpoints.typecheck`.

See the [plain page](./plain.md) for a route with no Code Play runtime.

```ts play typecheck play-title="Strict TypeScript" play-target=ESNext
const message: string = "hello from examples/code-play";
console.log(message);
console.warn("stderr from console.warn");
```

```ts play typecheck play-title="Loose TypeScript" play-strict=false play-compact
const label = "per-sample config";
console.log(label);
```

```js play play-title="JavaScript sum"
console.log(2 + 40);
```

See [@ox-content/code-play](https://github.com/ubugeeei-prod/ox-content/blob/main/docs/content/packages/code-play.md)
for the full language list and the headless API.
