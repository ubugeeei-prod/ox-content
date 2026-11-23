# @ox-content/vite-plugin-hono

Embed Hono JSX components in Markdown and MDX. The adapter renders components on the server and mounts interactive islands in the browser with `hono/jsx/dom`.

## Installation

```sh
npm install @ox-content/vite-plugin-hono hono
npm install -D @ox-content/vite-plugin vite
```

## Content sites

Add `oxContentHono()` to your Vite configuration. It configures Hono's JSX transform, content routes, the island registry and browser entry, and stylesheet discovery.

```ts
import { defineConfig } from "vite";
import { oxContentHono } from "@ox-content/vite-plugin-hono/vite";

export default defineConfig({
  plugins: [
    oxContentHono({
      oxContent: { srcDir: "content" },
      frontmatter: ({ title }) => ({ title: title ?? "" }),
    }),
  ],
});
```

For example, `content/index.mdx` renders at `/` and `content/guide.md` renders at `/guide/`. Use the usual Vite dev, build, and preview commands.

| Option          | Behavior                                                                                                                                                                                   |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `oxContent`     | Ox Content options for document discovery and rendering. Highlighting, resources, docs, search, feeds, and sitemaps default to off.                                                        |
| `components`    | A glob, an array of globs, or a name-to-module map for shared components. MDX can also import document-local components.                                                                   |
| `frontmatter`   | Selects the JSON values passed to Hono components on the server and in the browser. The default is an empty object.                                                                        |
| `layout`        | Path to a Hono JSX layout with a default export. Render `children` and `head` to include the document body and its CSS and script tags. The layout also receives `frontmatter` and `path`. |
| `routes: false` | Lets an existing app own routing and build inputs. Import `virtual:ox-content-hono/client` from the app's browser entry to mount islands. The app handles asset delivery.                  |

Import a component in MDX and choose when it mounts:

```mdx
import Counter from "./Counter.tsx";

<Counter initial={2} client:visible>

Slot **Markdown**.

</Counter>
```

Islands support one loading directive per component. Use `client:load` (the default), `client:idle`, `client:visible`, or `client:media="(max-width: 600px)"`. Use synchronous components for browser islands. Components can import CSS and read the selected frontmatter with `useFrontmatter()`. Select only fields you intend to include in publicly served HTML.

## Markdown as a Hono component

Import Markdown from a server entry or a Hono layout. Its default export is an async Hono component, with `frontmatter` and `toc` available as named exports. Add the `/virtual` type reference in a declaration file or where you import the document.

```tsx
/// <reference types="@ox-content/vite-plugin-hono/virtual" />
import Page, { frontmatter } from "../content/page.mdx";

export function Article() {
  return (
    <article>
      <h1>{String(frontmatter.title ?? "")}</h1>
      <Page />
    </article>
  );
}
```

Static Markdown imports in the layout are included when building the browser island registry. With a custom multi-stage build, use `honoJsx({ documents })` to supply documents outside the content collection and layout import graph.

## Custom HTML hosts

Use `honoJsx({ oxContent })` alongside `oxContentCustomHost()` to configure JSX and the island registry. Call `renderHonoMarkdown()` from the host's `ctx.markdown.render()` callback. Pass the returned page's asset tags into the document head and return its dependencies so the development server can invalidate the route.

In the browser entry, load the generated island registry and initialize the adapter:

```ts
/// <reference types="@ox-content/vite-plugin-hono/virtual" />
import { initHonoIslands } from "@ox-content/vite-plugin-hono/client";
import modules from "virtual:ox-content-hono/modules";

const controller = initHonoIslands({ modules });
if (import.meta.hot) import.meta.hot.dispose(() => controller.destroy());
```

| Entry point                    | Exports                                                                             |
| ------------------------------ | ----------------------------------------------------------------------------------- |
| `@ox-content/vite-plugin-hono` | `FrontmatterContext`, `useFrontmatter()`, and `Frontmatter`.                        |
| `/server`                      | `createHonoServerAdapter()` for the HTML-host renderer.                             |
| `/client`                      | `createHonoHydrate()` and `initHonoIslands()` for mounting and disposing islands.   |
| `/vite`                        | `oxContentHono()`, `honoJsx()`, `createHonoRenderer()`, and `renderHonoMarkdown()`. |
| `/virtual`                     | Type declarations for the generated modules and Markdown imports.                   |

The root, `/server`, and `/client` entry points can be imported without Vite or the Markdown compiler. When an island mounts, Hono replaces the island's server-rendered children. Until then, the server-rendered HTML remains visible.

See the [Hono JSX examples](../examples/integ-hono.md) for both setups.
