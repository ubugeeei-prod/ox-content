# @ox-content/vite-plugin-solid

Solid integration for Ox Content - embed Solid components in Markdown.

## Installation

```bash
vp install @ox-content/vite-plugin-solid solid-js vite-plugin-solid
```

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      srcDir: "docs",
      // Auto-discover components with glob pattern
      components: "./src/components/*.tsx",
    }),
    solid({ extensions: [".md", ".markdown", ".mdx"] }),
  ],
});
```

## Plugin Order and `extensions`

Unlike the Vue, React, and Svelte integrations, this plugin has two setup rules
that are not optional. Both follow from the same fact: **Solid's JSX is
compile-time only.** There is no runtime element factory like React's
`createElement` or Vue's `h` to fall back on, so Markdown is emitted as Solid
JSX and `vite-plugin-solid` is what turns it into DOM or SSR instructions.

1. `oxContentSolid()` must come **before** `solid()` in the `plugins` array.
   Both plugins are `enforce: "pre"`, so array order decides which one sees the
   Markdown file first. If `solid()` runs first, Babel tries to parse raw
   Markdown as JSX.
2. `solid()` must be given the Markdown extensions. By default it only looks at
   `.jsx` and `.tsx` files, so the generated modules would be handed to the
   browser as uncompiled JSX.

Both mistakes are checked for you and reported with the fix — see
[`verifySolidPlugin`](#verifysolidplugin).

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

### verifySolidPlugin

- Type: `boolean`
- Default: `true`

Fail fast on the two setup mistakes described above instead of letting them
surface as an unrelated syntax error.

The plugin check runs when the config resolves; the `extensions` check runs the
first time a Markdown module comes out of the pipeline still uncompiled. Both
throw a message naming the fix.

Turn it off when Solid's JSX is compiled by something other than
`vite-plugin-solid`.

## Using Components in Markdown

Register shared components in the global `components` map, or import a
component from the document that uses it.

```markdown
# My Page

Here's an interactive counter:

<Counter start={5} />

And an alert:

<Alert type="warning">
  This is a warning message!
</Alert>
```

On `.mdx`, a relative import is local to that file and overrides the global
map when the names match:

```md
import GtvChart from './gtv-chart/GtvChart.tsx'

<GtvChart title="ok" />
```

Optional `renderIsland(name, props, filePath)` can replace island inner HTML
at transform time. The hook belongs on the Solid adapter; `@ox-content/vite-plugin`
does not import `solid-js/web` for SSR.

## Example Component

```tsx
// src/components/Counter.tsx
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

Note that `props` is not destructured. Solid props are a reactive proxy, and
destructuring them reads every value once at setup time, which is what breaks
reactivity in components ported from React.

## Virtual Modules

- `virtual:ox-content-solid/components` - Registered components

```ts
import components from "virtual:ox-content-solid/components";
```

There is no `virtual:ox-content-solid/runtime` counterpart to the Svelte
integration's: Solid mounts through `render` from `solid-js/web`, which the
generated modules import directly.

## Islands

Markdown that uses a registered or document-imported component is emitted with
island markers and hydrated through `@ox-content/islands`, the same runtime the
other framework integrations use. Each island is mounted with `render` from
`solid-js/web` and disposed when the Markdown component unmounts.

Markdown without any registered or document-imported component skips the island
runtime entirely and compiles to a single `innerHTML` binding.

## HMR

Components are hot-reloaded when modified. Markdown modules that use a changed
component are invalidated alongside it.

## Rust and N-API Codegen

The Rust renderer can also emit Solid code directly from rendered Markdown HTML,
without the Vite pipeline:

```ts
import { renderFrameworkComponentCode } from "@ox-content/napi";

renderFrameworkComponentCode("<p>Hello</p>", "solid", [], "component");
```

That path targets `solid-js/h`, Solid's hyperscript entry point, because it has
to produce code that runs without the JSX compiler. The Vite plugin emits JSX
instead, which compiles to faster, finer-grained output.
