# Solid Integration Example

Demonstrates embedding Solid components in Markdown.

## Setup

The example depends on workspace packages, so install from the repository root:

<pm>npm install</pm>

Then start the example:

<tabs>
  <tab title="vp">
    <pre><code>vp run integ-solid</code></pre>
  </tab>
  <tab title="pnpm">
    <pre><code>pnpm run integ-solid</code></pre>
  </tab>
  <tab title="bun">
    <pre><code>bun run integ-solid</code></pre>
  </tab>
  <tab title="npm">
    <pre><code>npm run integ-solid</code></pre>
  </tab>
  <tab title="yarn">
    <pre><code>yarn integ-solid</code></pre>
  </tab>
</tabs>

## Configuration

```ts
// vite.config.ts
import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      srcDir: "docs",
      // Auto-discover all Solid components
      components: "./src/components/*.tsx",
    }),
    solid({ extensions: [".md", ".markdown", ".mdx"], compiler: "native" }),
  ],
});
```

Solid's JSX is compile-time only, so `oxContentSolid()` has to run first (it
produces the JSX) and `solid()` has to be told about the Markdown extensions
while using Solid 2's native compiler. See
[the package reference](../packages/vite-plugin-ox-content-solid.md#plugin-order-and-extensions).

## Components

### Counter

```tsx
import { createSignal } from "solid-js";

export default function Counter(props: { start?: number }) {
  const [count, setCount] = createSignal(props.start ?? 0);

  return (
    <div class="counter">
      <button onClick={() => setCount(count() - 1)}>-</button>
      <span>{count()}</span>
      <button onClick={() => setCount(count() + 1)}>+</button>
    </div>
  );
}
```

### Alert

```tsx
import type { JSX } from "solid-js";

export default function Alert(props: {
  type?: "info" | "success" | "warning";
  title?: string;
  children?: JSX.Element;
}) {
  return (
    <div class={`alert alert-${props.type ?? "info"}`}>
      {props.title ? <strong class="alert-title">{props.title}</strong> : null}
      <div class="alert-body">{props.children}</div>
    </div>
  );
}
```

## Usage in Markdown

```markdown
# My Documentation

<Counter start={10} />

<Alert type="warning">
  Be careful with this feature!
</Alert>
```

## Solid Notes

- **Signals** — `createSignal` returns a getter/setter pair; read state by
  calling the getter (`count()`), not by reading a value.
- **Props stay reactive** — do not destructure `props`. Destructuring reads
  every value once during setup and drops the reactivity.
- **No virtual DOM** — components run once and updates are applied directly to
  the DOM nodes that depend on the changed signal.

## File Structure

```text
integ-solid/
├── docs/
│   └── index.md
├── src/
│   ├── components/
│   │   ├── Counter.tsx
│   │   └── Alert.tsx
│   ├── App.tsx
│   ├── main.tsx
│   └── styles.css
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```
