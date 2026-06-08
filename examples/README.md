# Ox Content Examples

This directory contains example integrations and use cases for Ox Content.

## Framework Integrations

| Example                                    | Description     | Features                                   |
| ------------------------------------------ | --------------- | ------------------------------------------ |
| [vue-integration](./vue-integration)       | Vue 3 + Vite    | Composition API, reactive Markdown preview |
| [react-integration](./react-integration)   | React 18 + Vite | Hooks, useMemo optimization                |
| [svelte-integration](./svelte-integration) | Svelte 5 + Vite | Runes, reactive state                      |
| [vite-ssg](./vite-ssg)                     | Vite SSG        | Environment API, static generation         |

## Plugin Integrations

| Example                                                | Description                | Use Case                                                   |
| ------------------------------------------------------ | -------------------------- | ---------------------------------------------------------- |
| [markdown-it-plugin](./markdown-it-plugin)             | markdown-it integration    | Use Ox Content parser with markdown-it plugins             |
| [rehype-plugin](./rehype-plugin)                       | unified/rehype integration | Post-process HTML with rehype plugins                      |
| [unplugin-vite-ox-content](./unplugin-vite-ox-content) | mdast/unified bridge       | Run custom mdast and existing remark plugins on Ox Content |

## Tooling Examples

| Example                                      | Description               | Features                                     |
| -------------------------------------------- | ------------------------- | -------------------------------------------- |
| [source-docs](./source-docs)                 | Source code documentation | JSDoc extraction, Markdown generation        |
| [basic-playground](./basic-playground)       | Interactive playground    | Live preview, AST visualization              |
| [incremental-html-js](./incremental-html-js) | Incremental renderer demo | Plain HTML/CSS/JS, SSE, native Rust renderer |

## Running Examples

Each example is a standalone project. Navigate to the example directory and run:

```bash
# Install dependencies
npm install

# Start development server (for web examples)
npm run dev

# Or run directly (for Node.js examples)
npm start
```

## Example Structure

```
examples/
├── vue-integration/        # Vue 3 integration
├── react-integration/      # React 18 integration
├── svelte-integration/     # Svelte 5 integration
├── vite-ssg/              # Vite SSG with Environment API
├── markdown-it-plugin/    # markdown-it plugin
├── rehype-plugin/         # rehype/unified plugin
├── source-docs/           # Source code documentation
├── incremental-html-js/   # Plain HTML/CSS/JS incremental rendering
└── basic-playground/      # Interactive playground
```

## Creating New Examples

1. Create a new directory in `examples/`
2. Add a `package.json` with necessary dependencies
3. Include a `README.md` explaining the integration
4. Add the example to this README

## Integration Patterns

### Vite Environment API

```typescript
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      gfm: true,
    }),
  ],
});
```

### Direct NAPI Usage

```javascript
import { parseAndRender } from "@ox-content/napi";

const { html, frontmatter, toc } = parseAndRender(markdown, {
  gfm: true,
});
```

### markdown-it Plugin

```javascript
import MarkdownIt from "markdown-it";
import { oxContentPlugin } from "./plugin.js";

const md = new MarkdownIt();
md.use(oxContentPlugin, { gfm: true });
```

### rehype Pipeline

```javascript
import { unified } from "unified";
import rehypeParse from "rehype-parse";

const result = await unified()
  .use(rehypeParse, { fragment: true })
  .use(myRehypePlugin)
  .process(oxContentHtml);
```
