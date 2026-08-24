# Vue Integration Example

Demonstrates embedding Vue 3 components in Markdown.

## Setup

```bash
cd examples/integ-vue
npm install
npm run dev
```

## Configuration

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
      // Auto-discover all Vue components
      components: "./src/components/*.vue",
    }),
  ],
});
```

## Components

### Counter

Interactive counter with increment/decrement buttons.

```vue
<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{
  initial?: number;
}>();

const count = ref(props.initial ?? 0);
</script>

<template>
  <button @click="count--">-</button>
  <span>{{ count }}</span>
  <button @click="count++">+</button>
</template>
```

### Alert

Styled alert box with different types.

```vue
<script setup lang="ts">
defineProps<{
  type?: "info" | "warning" | "error" | "success";
}>();
</script>

<template>
  <div :class="['alert', type ?? 'info']">
    <slot />
  </div>
</template>
```

### CodeDemo

Live code demonstration with preview.

## Usage in Markdown

```markdown
# My Documentation

<Counter :initial="10" />

<Alert type="warning">
  Be careful with this feature!
</Alert>

<CodeDemo language="vue">
  <!-- Your code here -->
</CodeDemo>
```

## File Structure

```
integ-vue/
├── docs/
│   └── index.md
├── src/
│   ├── components/
│   │   ├── Counter.vue
│   │   ├── Alert.vue
│   │   └── CodeDemo.vue
│   ├── App.vue
│   └── main.ts
├── index.html
├── package.json
└── vite.config.ts
```
