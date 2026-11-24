# Hono JSX Integration Examples

The [Hono JSX adapter](../packages/vite-plugin-ox-content-hono.md) has two runnable examples. Both render an interactive counter on the server, load its CSS, and mount the counter in the browser.

From the repository root, install and build the workspace packages:

```sh
vp install
vp run build:npm
```

## Content site

Run `vp run integ-hono`. The example under `examples/integ-hono` uses `oxContentHono()` to build content routes and the browser entry. Its Hono layout imports standalone Markdown and MDX snippets, including an island outside the content directory.

Build static HTML and assets with `vp run --filter ./examples/integ-hono build`.

## Custom host

Run `vp run integ-hono-custom-host`. The example under `examples/integ-hono-custom-host` uses `honoJsx()` to register islands. Its custom host selects documents and calls `renderHonoMarkdown()`. Its browser entry initializes the islands explicitly.

Build static HTML and assets with `vp run --filter ./examples/integ-hono-custom-host build`.
