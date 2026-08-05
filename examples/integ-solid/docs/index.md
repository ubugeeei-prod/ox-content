---
title: Solid Integration Example
description: Embedding Solid components in Markdown
---

# Solid Integration Example

Embed Solid components directly in Markdown using `@ox-content/vite-plugin-solid`.

## Interactive Counter

<Counter start={3} />

Backed by Solid signals (`createSignal`) — no virtual DOM.

## Alert Components

<Alert type="info" title="Information">
Powered by Solid's fine-grained reactivity.
</Alert>

<Alert type="success" title="Success">
Components work seamlessly!
</Alert>

## Configuration

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import solid from "vite-plugin-solid";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      components: {
        Counter: "./src/components/Counter.tsx",
        Alert: "./src/components/Alert.tsx",
      },
    }),
    // Solid's JSX is compile-time only, so Markdown extensions must be listed
    // here, and this plugin must come after oxContentSolid().
    solid({ extensions: [".md", ".markdown", ".mdx"] }),
  ],
});
```

## Features

- Solid signals and fine-grained reactivity
- Hot Module Replacement
- Island hydration through `@ox-content/islands`
