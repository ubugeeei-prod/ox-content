# Code Annotations

This example shows the opt-in code annotation model for fenced code blocks. Ox Content supports both the configurable attribute syntax and VitePress-compatible notation.

## Enable the feature

```ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
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

Supported annotation kinds are `highlight`, `warning`, and `error`.

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

## Rendered example

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

## Custom meta key

You can swap `annotate` for another attribute name when you want a more domain-specific notation.

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
