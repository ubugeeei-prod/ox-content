# Examples

Ox Content provides runnable examples and small source snippets demonstrating
different use cases.

Small built-in feature snippets live in the repository under
`examples/builtin-features/`.

## Integration Examples

### [Vue Integration](./integ-vue.md)

Embed Vue 3 components in Markdown using `@ox-content/vite-plugin-vue`.

```ts
import { oxContentVue } from "@ox-content/vite-plugin-vue";

export default defineConfig({
  plugins: [
    vue(),
    oxContentVue({
      components: "./src/components/*.vue",
    }),
  ],
});
```

### [React Integration](./integ-react.md)

Embed React components in Markdown using `@ox-content/vite-plugin-react`.

```ts
import { oxContentReact } from "@ox-content/vite-plugin-react";

export default defineConfig({
  plugins: [
    react(),
    oxContentReact({
      components: "./src/components/*.tsx",
    }),
  ],
});
```

### [Svelte Integration](./integ-svelte.md)

Embed Svelte 5 components in Markdown using `@ox-content/vite-plugin-svelte`.

```ts
import { oxContentSvelte } from "@ox-content/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte(),
    oxContentSvelte({
      components: "./src/components/*.svelte",
    }),
  ],
});
```

### [Solid Integration](./integ-solid.md)

Embed Solid components in Markdown using `@ox-content/vite-plugin-solid`.

```ts
import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      components: "./src/components/*.tsx",
    }),
    // Solid's JSX is compile-time only, so this plugin runs after
    // oxContentSolid() and needs the Markdown extensions.
    solid({ extensions: [".md", ".markdown", ".mdx"], compiler: "native" }),
  ],
});
```

## Plugin Examples

### [Code Annotations](./code-annotations.md)

Opt-in code block annotations with both custom attributes and VitePress-compatible notation.

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    notation: "both",
  },
});
```

### [Package Manager Tabs](./package-manager-tabs.md)

Opt in to package-manager tabs, then author one npm command and render it as vp/pnpm/bun/npm/yarn install tabs.

```md
<pm>npm install -D vite</pm>
```

### [unplugin mdast Bridge](./unplugin-mdast-bridge.md)

Run custom mdast plugins and existing remark/unified plugins on top of Ox Content's native parser,
with documented compatibility boundaries and bridge performance notes.

### [unplugin markdown-it Token Bridge](./unplugin-markdown-it-token-bridge.md)

Run `markdown-it` plugins first and then read the resulting token stream from downstream unified plugins.

## Generator Examples

### [Source Docs Generation](./gen-source-docs.md)

Generate API documentation from JSDoc/TSDoc comments automatically.

```ts
oxContent({
  docs: {
    src: ["./src"],
    out: "docs/api",
    include: ["**/*.ts"],
  },
});
```

## OG Image Examples

### [OG Viewer](./og-viewer.md)

Dev tool for previewing Open Graph metadata of all pages. Accessible at `/__og-viewer` during development.

### [Custom OG Image Templates](./og-image-custom.md)

Generate per-page Open Graph images with a custom template. Pass arbitrary frontmatter data as props.

```ts
oxContent({
  ogImage: true,
  ogImageOptions: {
    template: "./og-template.ts",
  },
});
```

## Other Examples

### [Code Play](./code-play.md)

On-demand sample execution through `@ox-content/code-play`, with stdio, stderr,
config, provenance, and timing viewers. Live fences on the docs page; a
standalone Vite app is [`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play).

### [Playground](./playground.md)

Interactive web playground for testing Markdown parsing.

### [Vite SSG](./ssg-vite.md)

Static Site Generation example using Vite.

### [Built-in MDX](./mdx.md)

Default-on MDX for `.mdx` files: static HTML, island placeholders, ESM that
is not executed, and `{expression}` that is not evaluated. A sibling `.md`
file stays GFM. Runnable app:
[`examples/mdx`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/mdx).

## Running Examples

Clone the repository and install dependencies:

```bash
git clone https://github.com/ubugeeei-prod/ox-content.git
cd ox-content
```

<pm>npm install</pm>

Run examples from the repository root:

<tabs>
  <tab title="vp">
    <pre><code>vp run integ-vue
vp run ssg-vite
vp run mdx
vp run plugin-markdown-it
vp run --filter ./examples/code-play dev</code></pre>
  </tab>
  <tab title="pnpm">
    <pre><code>pnpm run integ-vue
pnpm run ssg-vite
pnpm run mdx
pnpm run plugin-markdown-it
pnpm --filter ./examples/code-play dev</code></pre>
  </tab>
  <tab title="bun">
    <pre><code>bun run integ-vue
bun run ssg-vite
bun run mdx
bun run plugin-markdown-it
bun --filter ./examples/code-play dev</code></pre>
  </tab>
  <tab title="npm">
    <pre><code>npm run integ-vue
npm run ssg-vite
npm run mdx
npm run plugin-markdown-it
npm --workspace ./examples/code-play run dev</code></pre>
  </tab>
  <tab title="yarn">
    <pre><code>yarn integ-vue
yarn ssg-vite
yarn mdx
yarn plugin-markdown-it
yarn workspace ox-content-code-play-example dev</code></pre>
  </tab>
</tabs>
