---
title: Code Annotations
description: Opt-in line annotations for fenced code blocks.
---

# Code Annotations

This page demonstrates opt-in line annotations rendered by the Rust-side code block pipeline.

## Enable in `vite.config.ts`

```ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "src/content",
      highlight: true,
      codeAnnotations: {
        notation: "both",
      },
    }),
  ],
});
```

## Markdown source

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

## Rendered example

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

## VitePress-compatible syntax

````md
```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
console.warn("Token expires soon") // [!code warning]
throw new Error("Token is invalid") // [!code error]
```
````

```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
console.warn("Token expires soon") // [!code warning]
throw new Error("Token is invalid") // [!code error]
```

## Custom meta key

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
