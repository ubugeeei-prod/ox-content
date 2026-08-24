# @ox-content/vite-plugin-react

React integration for Ox Content - embed React components in Markdown.

## Installation

```bash
vp install @ox-content/vite-plugin-react react react-dom @vitejs/plugin-react
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { oxContentReact } from "@ox-content/vite-plugin-react";

export default defineConfig({
  plugins: [
    react(),
    oxContentReact({
      srcDir: "docs",
      // Auto-discover components with glob pattern
      components: "./src/components/*.tsx",
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
components: "./src/components/*.tsx";

// Multiple patterns
components: ["./src/components/*.tsx", "./src/ui/*.tsx"];
```

Component names are derived from file names in PascalCase:

- `counter.tsx` → `Counter`
- `my-button.tsx` → `MyButton`

#### Explicit Map

```ts
components: {
  Counter: './src/components/Counter.tsx',
  Alert: './src/components/Alert.tsx',
}
```

### jsxRuntime

- Type: `'automatic' | 'classic'`
- Default: `'automatic'`

JSX runtime mode for React 17+.

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

## Example Component

```tsx
// src/components/Counter.tsx
import { useState } from "react";

interface CounterProps {
  initial?: number;
}

export default function Counter({ initial = 0 }: CounterProps) {
  const [count, setCount] = useState(initial);

  return (
    <div className="counter">
      <button onClick={() => setCount((c) => c - 1)}>-</button>
      <span>{count}</span>
      <button onClick={() => setCount((c) => c + 1)}>+</button>
    </div>
  );
}
```

## Virtual Modules

- `virtual:ox-content-react/runtime` - React-specific runtime
- `virtual:ox-content-react/components` - Registered components

```ts
import { OxContentRenderer, useOxContent } from "virtual:ox-content-react/runtime";
import components from "virtual:ox-content-react/components";
```

## HMR

Components are hot-reloaded when modified. Fast Refresh is supported.
