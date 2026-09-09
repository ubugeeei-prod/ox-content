import fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser } from "playwright";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, createServer, type InlineConfig, type ViteDevServer } from "vite";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];
const PACKAGE_ROOT = path.dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("custom host Svelte SSR stylesheets", () => {
  it.each([
    ["official Svelte", "@sveltejs/vite-plugin-svelte"],
    ["rsvelte", "@rsvelte/vite-plugin-svelte"],
  ] as const)("discovers scoped Svelte CSS in production with %s", async (_name, plugin) => {
    const root = await createProject("ox-custom-host-svelte-build-");
    await writeViteConfig(root, plugin);
    await viteBuild(viteConfig(root));

    const html = await fs.readFile(path.join(root, "dist", "probe", "index.html"), "utf8");
    expect(html).toContain('data-diagnostics=""');
    expect(html).toContain('data-style-content-diagnostics=""');
    expect(html).toContain('data-critical="/docs/assets/');
    expect(html).toContain("Hello");
    expect(html).not.toContain('<script type="module"');

    const css = await readLinkedCss(root, html);
    expect(css).toMatch(/color:(?:red|rgb\(255,0,0\))/u);
    expect(css).toMatch(/color:(?:#00f|blue|rgb\(0,0,255\))/u);
  });

  it("serves blocking scoped Svelte styles before client JavaScript", async () => {
    const root = await createProject("ox-custom-host-svelte-browser-");
    await writeViteConfig(root, "@sveltejs/vite-plugin-svelte");
    await viteBuild(viteConfig(root));
    const staticServer = await serveDist(root);
    let browser: Browser | undefined;

    try {
      browser = await chromium.launch({
        channel: process.env.CI ? "chrome" : undefined,
        headless: true,
      });
      const page = await browser.newPage();
      const response = await page.goto(`http://127.0.0.1:${staticServer.port}/docs/probe/`);
      expect(response?.status()).toBe(200);

      const state = await page.evaluate(() => {
        const probe = document.querySelector(".probe");
        const child = document.querySelector(".child");
        return {
          scripts: document.scripts.length,
          probeColor: probe ? getComputedStyle(probe).color : "",
          childColor: child ? getComputedStyle(child).color : "",
        };
      });

      expect(state.scripts).toBe(0);
      expect(state.probeColor).toBe("rgb(255, 0, 0)");
      expect(state.childColor).toBe("rgb(0, 0, 255)");
    } finally {
      await browser?.close();
    }
  });

  it.each([
    ["official Svelte", "@sveltejs/vite-plugin-svelte"],
    ["rsvelte", "@rsvelte/vite-plugin-svelte"],
  ] as const)("serves and invalidates scoped Svelte CSS in dev with %s", async (_name, plugin) => {
    const root = await createProject("ox-custom-host-svelte-dev-");
    await writeViteConfig(root, plugin, { reloadDebounceMs: 1 });
    const server = await trackServer(createServer(viteConfig(root)));
    const listener = await listen(server);

    const first = await read(listener.port, "/docs/probe");
    expect(first.status, first.text).toBe(200);
    expect(first.text).toContain('data-diagnostics=""');
    expect(first.text).toContain("Page.svelte");
    expect(first.text).toContain("Child.svelte");
    expect(first.text).toContain("type=style");

    expect(first.text).toMatch(/color:\s*rgb\(0,\s*0,\s*255\)/u);

    await writeFile(
      root,
      "src/Child.svelte",
      '<p class="child">child</p>\n<style>.child{color:rgb(0,128,0)}</style>\n',
    );
    server.watcher.emit("change", path.join(root, "src", "Child.svelte"));
    await wait(50);

    const updated = await read(listener.port, "/docs/probe");
    expect(updated.text).toContain('data-render="2"');
    expect(updated.text).toMatch(/color:\s*rgb\(0,\s*128,\s*0\)/u);
  });
});

function viteConfig(root: string): InlineConfig {
  return {
    root,
    configFile: path.join(root, "vite.config.mjs"),
  };
}

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(PACKAGE_ROOT, `.tmp-${prefix}`));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await writeFile(root, "package.json", '{"type":"module"}\n');
  await writeFile(root, "src/main.js", "document.body.dataset.client = 'unused';\n");
  await writeFile(
    root,
    "src/Page.svelte",
    [
      "<script>",
      '  import Child from "./Child.svelte";',
      "</script>",
      '<h1 class="probe">Hello</h1>',
      "<Child />",
      "<style>.probe{color:rgb(255,0,0)}</style>",
    ].join("\n"),
  );
  await writeFile(
    root,
    "src/Child.svelte",
    '<p class="child">child</p>\n<style>.child{color:rgb(0,0,255)}</style>\n',
  );
  await writeFile(root, "src/host.ts", hostModuleSource());
  return root;
}

async function writeViteConfig(
  root: string,
  sveltePackage: string,
  dev: { reloadDebounceMs?: number } = {},
): Promise<void> {
  const devOptions = { transformHtml: false, ...dev };
  await writeFile(
    root,
    "vite.config.mjs",
    [
      'import path from "node:path";',
      'import { fileURLToPath } from "node:url";',
      'import { oxContentCustomHost } from "@ox-content/vite-plugin";',
      `import { svelte } from ${JSON.stringify(sveltePackage)};`,
      'const root = fileURLToPath(new URL(".", import.meta.url));',
      "export default {",
      '  base: "/docs/",',
      "  appType: 'custom',",
      "  logLevel: 'silent',",
      "  plugins: [",
      "    noopSsrSvelteStyleImports(),",
      "    svelte(),",
      "    ...oxContentCustomHost({",
      '      host: "./src/host.ts",',
      "      oxContent: {",
      '        base: "/docs/",',
      '        srcDir: "content",',
      '        outDir: "dist",',
      "        resources: false,",
      "        docs: false,",
      "        search: false,",
      "        ogViewer: false,",
      "        feeds: false,",
      "        siteMaps: false,",
      "      },",
      '      ssrStylesheets: { modules: ["/src/Page.svelte"] },',
      "      build: { transformHtml: false, runInTest: true },",
      `      dev: ${JSON.stringify(devOptions)},`,
      "    }),",
      "  ],",
      "  server: { fs: { allow: [root] } },",
      "  build: {",
      '    outDir: "dist",',
      "    emptyOutDir: true,",
      "    cssMinify: true,",
      "    cssCodeSplit: true,",
      "    manifest: true,",
      '    rollupOptions: { input: path.join(root, "src", "main.js") },',
      "  },",
      "};",
      "function noopSsrSvelteStyleImports() {",
      "  const prefix = '\\0ox-content-svelte-ssr-style:';",
      "  const modules = new Map();",
      "  let command = 'build';",
      "  return {",
      "    name: 'ox-content-svelte-test:ssr-style-noop',",
      "    enforce: 'pre',",
      "    config(_config, env) {",
      "      command = env.command;",
      "    },",
      "    resolveId(id, importer, options) {",
      "      const loadingHost = process.env.OX_CONTENT_CUSTOM_HOST_LOAD === '1';",
      "      const serving = command === 'serve';",
      "      if ((loadingHost || serving || options?.ssr) && importer?.includes('.svelte') && isSvelteStyleModuleId(id)) {",
      "        const resolved = `${prefix}${modules.size}`;",
      "        modules.set(resolved, id);",
      "        return resolved;",
      "      }",
      "      return null;",
      "    },",
      "    load(id) {",
      "      if (modules.has(id)) return \"export default '';\";",
      "      return null;",
      "    },",
      "  };",
      "}",
      "function isSvelteStyleModuleId(id) {",
      "  return id.includes('.svelte') && /[?&]type=style(?:&|$)/u.test(id);",
      "}",
      "",
    ].join("\n"),
  );
}

function hostModuleSource(): string {
  return `
let renders = 0;

export default {
  routes: [
    {
      path: "/probe",
      async render(ctx) {
        renders += 1;
        const { default: Page } = await ctx.loadModule("/src/Page.svelte");
        const { render } = await ctx.loadModule("svelte/server");
        const rendered = render(Page);
        const ssr = ctx.assets.ssrStylesheets({ modules: ["/src/Page.svelte"] });
        const content = await ctx.assets.stylesheetContent({ stylesheets: ssr.stylesheets });
        const assets = ctx.assets.document({
          islandStyles: ssr.stylesheets,
          inlineStyles: content.stylesheets.map((stylesheet) => ({
            key: "critical:" + stylesheet.href,
            content: stylesheet.content,
            attrs: { "data-critical": stylesheet.href },
          })),
        });
        return {
          html: "<!doctype html><html><head>" + assets.headHtml + "</head><body data-render=\\"" + renders + "\\" data-diagnostics=\\"" + ssr.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "\\" data-style-content-diagnostics=\\"" + content.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "\\">" + (rendered.html || rendered.body || "") + "</body></html>",
          dependencies: ssr.dependencies,
        };
      },
    },
  ],
};
`;
}

async function writeFile(root: string, file: string, content: string): Promise<void> {
  await fs.writeFile(path.join(root, ...file.split("/")), content);
}

async function trackServer(serverPromise: Promise<ViteDevServer>): Promise<ViteDevServer> {
  const server = await serverPromise;
  activeServers.push(server);
  return server;
}

async function listen(server: ViteDevServer): Promise<{ port: number }> {
  const listener = http.createServer(server.middlewares);
  activeListeners.push(listener);
  await new Promise<void>((resolve) => listener.listen(0, "127.0.0.1", resolve));
  return { port: (listener.address() as AddressInfo).port };
}

async function read(port: number, requestPath: string) {
  const response = await fetch(`http://127.0.0.1:${port}${requestPath}`);
  return { status: response.status, headers: response.headers, text: await response.text() };
}

async function serveDist(root: string): Promise<{ port: number }> {
  const dist = path.join(root, "dist");
  const listener = http.createServer(async (req, res) => {
    const requestPath = new URL(req.url ?? "/", "http://localhost").pathname;
    const relative = requestPath.startsWith("/docs/")
      ? requestPath.slice("/docs/".length)
      : requestPath.replace(/^\/+/u, "");
    const decoded = decodeURIComponent(relative || "index.html");
    const file = path.resolve(dist, decoded);
    const candidate = requestPath.endsWith("/") ? path.join(file, "index.html") : file;
    if (!candidate.startsWith(`${dist}${path.sep}`)) {
      res.statusCode = 403;
      res.end("Forbidden");
      return;
    }
    try {
      res.statusCode = 200;
      res.end(await fs.readFile(candidate));
    } catch {
      res.statusCode = 404;
      res.end("Not found");
    }
  });
  activeListeners.push(listener);
  await new Promise<void>((resolve) => listener.listen(0, "127.0.0.1", resolve));
  return { port: (listener.address() as AddressInfo).port };
}

async function readLinkedCss(root: string, html: string): Promise<string> {
  const files = linkedCssPaths(html).map((href) =>
    path.join(root, "dist", href.replace(/^\/docs\//u, "")),
  );
  return (await Promise.all(files.map((file) => fs.readFile(file, "utf8")))).join("\n");
}

function linkedCssPaths(html: string): string[] {
  return [...html.matchAll(/href="(\/docs\/[^"]+)"/gu)].map((match) =>
    match[1].replaceAll("&amp;", "&"),
  );
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function closeServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
