# @ox-content/vite-plugin-svelte

Svelte integration for Ox Content - embed Svelte 5 components in Markdown.

## Installation

```bash
vp install @ox-content/vite-plugin-svelte svelte @sveltejs/vite-plugin-svelte
```

## Usage

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
      // Auto-discover components with glob pattern
      components: "./src/components/*.svelte",
    }),
  ],
});
```

## Options

### components

- Type: `string | string[] | Record<string, string>`

Components to register for use in Markdown. Supports:

#### Glob Pattern (Recommended)

```ts
// Single pattern
components: "./src/components/*.svelte";

// Multiple patterns
components: ["./src/components/*.svelte", "./src/ui/*.svelte"];
```

Component names are derived from file names in PascalCase:

- `counter.svelte` → `Counter`
- `my-button.svelte` → `MyButton`

#### Explicit Map

```ts
components: {
  Counter: './src/components/Counter.svelte',
  Alert: './src/components/Alert.svelte',
}
```

### runes

- Type: `boolean`
- Default: `true`

Enable Svelte 5 Runes mode.

## Using Components in Markdown

```markdown
# My Page

Here's an interactive counter:

<Counter initial={5} />

And an alert:

<Alert type="warning">
  This is a warning message!
</Alert>
```

## Example Component (Svelte 5 Runes)

```svelte
<!-- src/components/Counter.svelte -->
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

<style>
  .counter {
    display: inline-flex;
    gap: 8px;
    align-items: center;
  }
</style>
```

## Virtual Modules

- `virtual:ox-content-svelte/runtime` - Svelte-specific runtime
- `virtual:ox-content-svelte/components` - Registered components

```ts
import { mount, unmount } from "virtual:ox-content-svelte/runtime";
import components from "virtual:ox-content-svelte/components";
```

## HMR

Components are hot-reloaded when modified. Svelte's built-in HMR is supported.

## Svelte 5 Features

This plugin is designed for Svelte 5 and supports:

- **Runes**: `$state`, `$derived`, `$effect`, `$props`
- **Snippets**: `{#snippet}` and `{@render}`
- **New event syntax**: `onclick` instead of `on:click`
