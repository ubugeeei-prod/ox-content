# React Integration Example

Demonstrates embedding React components in Markdown.

## Setup

```bash
cd examples/integ-react
npm install
npm run dev
```

## Configuration

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
      // Auto-discover all React components
      components: "./src/components/*.tsx",
    }),
  ],
});
```

## Components

### Counter

```tsx
import { useState } from "react";

interface Props {
  initial?: number;
}

export default function Counter({ initial = 0 }: Props) {
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

### Alert

```tsx
interface Props {
  type?: "info" | "warning" | "error" | "success";
  children: React.ReactNode;
}

export default function Alert({ type = "info", children }: Props) {
  return <div className={`alert alert-${type}`}>{children}</div>;
}
```

## Usage in Markdown

```markdown
# My Documentation

<Counter initial={10} />

<Alert type="warning">
  Be careful with this feature!
</Alert>
```

## File Structure

```
integ-react/
├── docs/
│   └── index.md
├── src/
│   ├── components/
│   │   ├── Counter.tsx
│   │   └── Alert.tsx
│   ├── App.tsx
│   └── main.tsx
├── index.html
├── package.json
└── vite.config.ts
```
