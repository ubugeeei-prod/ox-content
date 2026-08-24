# Svelte Integration Example

Demonstrates embedding Svelte 5 components in Markdown.

## Setup

```bash
cd examples/integ-svelte
npm install
npm run dev
```

## Configuration

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { oxContentSvelte } from "@ox-content/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte(),
    oxContentSvelte({
      srcDir: "docs",
      // Auto-discover all Svelte components
      components: "./src/components/*.svelte",
    }),
  ],
});
```

## Components (Svelte 5 Runes)

### Counter

```svelte
<script lang="ts">
  interface Props {
    initial?: number;
  }

  let { initial = 0 }: Props = $props();
  let count = $state(initial);
</script>

<div class="counter">
  <button onclick={() => count--}>-</button>
  <span>{count}</span>
  <button onclick={() => count++}>+</button>
</div>
```

### Alert

```svelte
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    type?: 'info' | 'warning' | 'error' | 'success';
    children: Snippet;
  }

  let { type = 'info', children }: Props = $props();
</script>

<div class="alert alert-{type}">
  {@render children()}
</div>
```

## Usage in Markdown

```markdown
# My Documentation

<Counter initial={10} />

<Alert type="warning">
  Be careful with this feature!
</Alert>
```

## Svelte 5 Features

This example uses Svelte 5's new features:

- **$state** - Reactive state declaration
- **$props** - Component props
- **$derived** - Computed values
- **Snippets** - Composable template fragments
- **New event syntax** - `onclick` instead of `on:click`

## File Structure

```
integ-svelte/
├── docs/
│   └── index.md
├── src/
│   ├── components/
│   │   ├── Counter.svelte
│   │   └── Alert.svelte
│   ├── App.svelte
│   └── main.ts
├── index.html
├── package.json
├── svelte.config.js
└── vite.config.ts
```
