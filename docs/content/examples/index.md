# Examples

Ox Content provides several examples demonstrating different use cases.

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

### [markdown-it Plugin](./plugin-markdown-it.md)

Use Ox Content as a markdown-it plugin for existing markdown-it projects.

### [rehype Plugin](./plugin-rehype.md)

Use Ox Content as a rehype plugin in the unified ecosystem.

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

### [Playground](./playground.md)

Interactive web playground for testing Markdown parsing.

### [Vite SSG](./ssg-vite.md)

Static Site Generation example using Vite.

## Running Examples

```bash
# Clone the repository
git clone https://github.com/ubugeeei/ox-content.git
cd ox-content

# Install dependencies
npm install

# Run an example
cd examples/integ-vue
npm run dev
```
