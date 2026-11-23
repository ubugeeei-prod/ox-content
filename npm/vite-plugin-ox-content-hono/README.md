# @ox-content/vite-plugin-hono

Embed Hono JSX components in Markdown with Ox Content, render them on the server, and mount interactive islands in the browser.

```sh
npm install @ox-content/vite-plugin-hono hono
npm install -D @ox-content/vite-plugin vite
```

Add the adapter to your Vite config:

```ts
import { defineConfig } from "vite";
import { oxContentHono } from "@ox-content/vite-plugin-hono/vite";

export default defineConfig({
  plugins: [oxContentHono({ oxContent: { srcDir: "content" } })],
});
```

See the [package reference](https://ox-content.dev/packages/vite-plugin-ox-content-hono.md) for Markdown imports, layouts, and the custom HTML-host API.

This adapter originated in [ts-76/ox-content-hono-jsx](https://github.com/ts-76/ox-content-hono-jsx) and is distributed under the MIT license.
