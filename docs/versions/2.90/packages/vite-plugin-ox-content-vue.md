# @ox-content/vite-plugin-vue

Vue integration for Ox Content - embed Vue components in Markdown.

## Installation

```bash
vp install @ox-content/vite-plugin-vue vue @vitejs/plugin-vue
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { oxContentVue } from "@ox-content/vite-plugin-vue";

export default defineConfig({
  plugins: [
    vue(),
    oxContentVue({
      srcDir: "docs",
      // Auto-discover components with glob pattern
      components: "./src/components/*.vue",
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
components: "./src/components/*.vue";

// Multiple patterns
components: ["./src/components/*.vue", "./src/ui/*.vue"];
```

Component names are derived from file names in PascalCase:

- `counter.vue` → `Counter`
- `my-button.vue` → `MyButton`
- `AlertBox.vue` → `AlertBox`

#### Explicit Map

```ts
components: {
  Counter: './src/components/Counter.vue',
  MyAlert: './src/components/AlertBox.vue',
}
```

### reactivityTransform

- Type: `boolean`
- Default: `false`

Enable Vue Reactivity Transform.

### customBlocks

- Type: `boolean`
- Default: `true`

Enable custom blocks in Markdown (e.g., `:::tip`).

## Using Components in Markdown

```markdown
# My Page

Here's an interactive counter:

<Counter :initial="5" />

And an alert:

<Alert type="warning">
  This is a warning message!
</Alert>
```

## Example Component

```vue
<!-- src/components/Counter.vue -->
<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{
  initial?: number;
}>();

const count = ref(props.initial ?? 0);
</script>

<template>
  <div class="counter">
    <button @click="count--">-</button>
    <span>{{ count }}</span>
    <button @click="count++">+</button>
  </div>
</template>

<style scoped>
.counter {
  display: inline-flex;
  gap: 8px;
  align-items: center;
}
</style>
```

## Virtual Modules

- `virtual:ox-content-vue/runtime` - Vue-specific runtime
- `virtual:ox-content-vue/components` - Registered components

```ts
import { OxContentRenderer, useOxContent } from "virtual:ox-content-vue/runtime";
import components from "virtual:ox-content-vue/components";
```

## HMR

Components are hot-reloaded when modified. Markdown files using those components are also refreshed.
