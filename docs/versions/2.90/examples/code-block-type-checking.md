---
title: Code Block Type Checking
description: Type-check TypeScript fences with tsgo.
---

# Code Block Type Checking

Type-checking is opt-in because it shells out to `tsgo`. By default, only
TypeScript fences marked with `typecheck` or `twoslash` are checked.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeBlockTypecheck: {
        languages: ["ts", "tsx"],
        requireMeta: true,
        tsgoCommand: "tsgo",
        mode: "error",
      },
    }),
  ],
};
```

````md
```ts typecheck
const value: string = "ok";
```
````

Set `requireMeta: false` to check every TypeScript fence.
