# Source Documentation Generator Example

Demonstrates auto-generating API documentation from JSDoc/TSDoc comments, then layering a richer viewer on top of the generated `docs.json` payload.

## Setup

```bash
cd examples/gen-source-docs
npm install
npm run dev
```

## Configuration

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { oxContent } from "@ox-content/vite-plugin";
import { oxContentVue } from "@ox-content/vite-plugin-vue";

export default defineConfig({
  plugins: [
    vue(),

    // Base plugin with docs generation
    oxContent({
      srcDir: "docs",
      docs: {
        enabled: true,
        src: ["./src"],
        out: "docs/api",
        include: ["**/*.ts"],
        exclude: ["**/*.test.*"],
        format: "markdown",
        toc: true,
        groupBy: "file",
      },
    }),

    // Vue integration for interactive docs
    oxContentVue({
      srcDir: "docs",
      components: "./src/components/*.vue",
    }),
  ],
});
```

## Source Files

### utils.ts

````ts
/**
 * Capitalizes the first letter of a string.
 *
 * @param str - The input string
 * @returns The capitalized string
 *
 * @example
 * ```ts
 * capitalize('hello') // => 'Hello'
 * ```
 */
export function capitalize(str: string): string {
  if (!str) return str;
  return str.charAt(0).toUpperCase() + str.slice(1);
}
````

### math.ts

```ts
/**
 * Adds two numbers together.
 *
 * @param a - First number
 * @param b - Second number
 * @returns The sum
 */
export function add(a: number, b: number): number {
  return a + b;
}
```

## Generated Output

The plugin generates Markdown files plus a machine-readable JSON manifest in `docs/api/`:

```
docs/api/
├── docs.json      # Structured data for custom viewers
├── index.md       # API index
├── utils.md       # utils.ts documentation
└── math.md        # math.ts documentation
```

The example viewer uses that JSON to provide:

- one-line symbol overviews
- expandable accordion details
- scoped search such as `@api clamp`

## Dogfooding

This example demonstrates dogfooding - using Ox Content's tools to document example source files with Vue components for interactive documentation viewing.
