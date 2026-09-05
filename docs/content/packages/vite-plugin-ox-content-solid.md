# @ox-content/vite-plugin-solid

Solid integration for Ox Content - embed Solid components in Markdown.

## Installation

```bash
vp install @ox-content/vite-plugin-solid solid-js@next @solidjs/web@next @solidjs/vite-plugin
```

This 3.x beta adapter targets Solid 2 and `@solidjs/vite-plugin`. It does not
preserve the Solid 1 + `vite-plugin-solid` peer dependency path; keep the older
adapter release if your app has to stay on Solid 1.

## Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      srcDir: "docs",
      // Auto-discover components with glob pattern
      components: "./src/components/*.tsx",
    }),
    solid({ extensions: [".md", ".markdown", ".mdx"], compiler: "native" }),
  ],
});
```

## Plugin Order and `extensions`

Unlike the Vue, React, and Svelte integrations, this plugin has two setup rules
that are not optional. Both follow from the same fact: **Solid's JSX is
compile-time only.** There is no runtime element factory like React's
`createElement` or Vue's `h` to fall back on, so Markdown is emitted as Solid
JSX and `@solidjs/vite-plugin` is what turns it into DOM or SSR instructions.

1. `oxContentSolid()` must come **before** `solid()` in the `plugins` array.
   Both plugins are `enforce: "pre"`, so array order decides which one sees the
   Markdown file first. If `solid()` runs first, the Solid compiler sees raw
   Markdown instead of the generated JSX.
2. `solid()` must be given the Markdown extensions. Use
   `compiler: "native"` for the Solid 2 native OXC compiler path. Without the
   extensions, generated Markdown modules would be handed to the browser as
   uncompiled JSX.

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
`@solidjs/vite-plugin`.

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
does not import `@solidjs/web` for SSR.

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
integration's: Solid mounts through `render` from `@solidjs/web`, which the
generated modules import directly.

## Islands

Markdown that uses a registered or document-imported component is emitted with
island markers and hydrated through `@ox-content/islands`, the same runtime the
other framework integrations use. Each island is mounted with `render` from
`@solidjs/web` and disposed when the Markdown component unmounts.

Markdown without any registered or document-imported component skips the island
runtime entirely and compiles to a single `innerHTML` binding.

## HTML-string custom hosts

Hosts that call `renderMarkdown()` and then place the returned HTML inside their
own Solid page shell can use the Solid adapter without importing each Markdown
document as a Vite module. `renderSolidHtmlHost()` resolves document-local MDX
imports, loads the matching server modules through a host-supplied loader, and
replaces island bodies with Solid SSR HTML.

```ts
import { renderSolidHtmlHost, type MdxImport } from "@ox-content/vite-plugin-solid";

const imports: MdxImport[] = [
  { source: "./Chart.tsx", specifiers: [{ imported: "default", local: "Chart", kind: "default" }] },
];

const rendered = await renderSolidHtmlHost({
  html: markdown.html,
  documentPath: "/repo/docs/report.mdx",
  root: "/repo",
  srcDir: "docs",
  imports,
  components: { Badge: "./src/components/Badge.tsx" },
  loadModule: (moduleId) => viteDevServer.ssrLoadModule(moduleId),
  resolveClientModule: (module) => `/assets/islands/${module.name}.js`,
});
```

The module cache is scoped to one render call, so development edits should
trigger a fresh call after the host invalidates its page state. `modules` is
server-side metadata for this render; serialize only `clientModules` or your own
resolver output when sending island module identities to the browser.

Diagnostics report missing components, module load failures, missing exports,
SSR errors, and unsupported document-local import forms with document/component
context. The supported document-local forms are the same ones the Markdown-module
adapter understands: default imports and named imports with local bindings.

The client helper is a fresh-mount bridge, not Solid hydration. It reads the
existing Ox Content island payload and slot HTML, clears the target, and lets the
caller mount with `@solidjs/web`. Use it with `initIslands()` so load strategies,
disposal, and cancellation stay owned by the shared island runtime.

```tsx
import { initIslands } from "@ox-content/islands";
import { render } from "@solidjs/web";
import { createSolidHtmlHostHydrate } from "@ox-content/vite-plugin-solid";
import * as components from "./generated-island-client-modules";

const hydrate = createSolidHtmlHostHydrate({
  components,
  render(Component, props, element, slotHtml) {
    const dispose = render(
      () =>
        slotHtml ? (
          <Component {...props}>
            <div innerHTML={slotHtml} />
          </Component>
        ) : (
          <Component {...props} />
        ),
      element,
    );
    return dispose;
  },
});

initIslands(hydrate, { selector: ".ox-content [data-ox-island]" });
```

## Island stylesheets for custom hosts

Server-rendered islands often need their CSS before the client module mounts.
`resolveSolidIslandStylesheets()` maps island module identities to stylesheet
URLs from either a Vite build manifest or a development module graph.

```ts
import { resolveSolidIslandStylesheets } from "@ox-content/vite-plugin-solid";

const styles = resolveSolidIslandStylesheets({
  modules: rendered.modules.map((module) => module.serverModuleId),
  manifest: viteManifest,
  base: "/docs/",
});

for (const stylesheet of styles.stylesheets) {
  head.push(`<link rel="stylesheet" href="${escapeHtml(stylesheet.href)}">`);
}
```

Build resolution walks static `imports`, deduplicates CSS, keeps imported chunk
CSS before the importing island, and respects `base` plus emitted hashed file
names. Development resolution accepts a Vite-like module graph with
`getModuleById()` and optional `getModulesByFile()`, preserving CSS query
parameters so HMR URLs remain loadable. A valid island with no CSS returns no
stylesheet and no diagnostic; missing manifest/module-graph entries report a
`missing-module` diagnostic. If neither resolver is supplied, each requested
module reports `missing-resolver`.

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
