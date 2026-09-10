import fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Page } from "playwright";
import type { InlineConfig, ViteDevServer } from "vite";
import { expect } from "vite-plus/test";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];
const PACKAGE_ROOT = path.dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

export async function cleanupSvelteStylesheetFixture(): Promise<void> {
  await Promise.all(activeListeners.splice(0).map((server) => closeServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
}

export function viteConfig(root: string): InlineConfig {
  return { root, configFile: path.join(root, "vite.config.mjs") };
}

export async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(PACKAGE_ROOT, `.tmp-${prefix}`));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "src", "components"), { recursive: true });
  await fs.mkdir(path.join(root, "src", "config"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await writeFile(root, "package.json", '{"type":"module"}\n');
  await writeFile(
    root,
    "tsconfig.json",
    JSON.stringify({ compilerOptions: { paths: { "@/*": ["./src/*"] } } }, null, 2),
  );
  await writeFile(root, "src/main.js", "document.body.dataset.client = 'unused';\n");
  await writeFile(root, "src/Page.svelte", pageSource());
  await writeFile(root, "src/components/Child.svelte", childSource("rgb(0,0,255)"));
  await writeFile(root, "src/SiteLayout.svelte", siteLayoutSource());
  await writeFile(root, "src/components/SiteHeader.svelte", siteHeaderSource());
  await writeFile(root, "src/config/site.ts", 'export const siteName = "site";\n');
  await writeFile(root, "src/config/site-owner.ts", 'export const ownerName = "owner";\n');
  await writeFile(root, "src/host.ts", hostModuleSource());
  return root;
}

export async function writeViteConfig(
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
      "  resolve: {",
      '    alias: { "@": path.join(root, "src") },',
      "    tsconfigPaths: true,",
      "  },",
      "  plugins: [",
      "    noopSsrSvelteStyleImports(),",
      "    svelte({ configFile: false }),",
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
      '      ssrStylesheets: { modules: ["/src/Page.svelte", "/src/SiteLayout.svelte"] },',
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

export async function expectRenderedStyles(page: Page, childColor: string): Promise<void> {
  const state = await page.evaluate(() => {
    const probe = document.querySelector(".probe");
    const child = document.querySelector(".child");
    const header = document.querySelector(".siteHeader");
    return {
      client: document.body.dataset.client ?? "",
      probeColor: probe ? getComputedStyle(probe).color : "",
      childColor: child ? getComputedStyle(child).color : "",
      headerColor: header ? getComputedStyle(header).color : "",
    };
  });
  expect(state).toMatchObject({
    client: "",
    probeColor: "rgb(255, 0, 0)",
    childColor,
    headerColor: "rgb(0, 128, 0)",
  });
}

export async function writeFile(root: string, file: string, content: string): Promise<void> {
  await fs.writeFile(path.join(root, ...file.split("/")), content);
}

export async function trackServer(serverPromise: Promise<ViteDevServer>): Promise<ViteDevServer> {
  const server = await serverPromise;
  activeServers.push(server);
  return server;
}

export async function listen(server: ViteDevServer): Promise<{ port: number }> {
  const listener = http.createServer(server.middlewares);
  activeListeners.push(listener);
  await new Promise<void>((resolve) => listener.listen(0, "127.0.0.1", resolve));
  return { port: (listener.address() as AddressInfo).port };
}

export async function serveDist(root: string): Promise<{ port: number }> {
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

export async function readLinkedCss(root: string, html: string): Promise<string> {
  const files = linkedCssPaths(html).map((href) =>
    path.join(root, "dist", href.replace(/^\/docs\//u, "")),
  );
  return (await Promise.all(files.map((file) => fs.readFile(file, "utf8")))).join("\n");
}

export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function pageSource(): string {
  return '<script>\n  import Child from "@/components/Child.svelte";\n  import { siteName } from "@/config/site.ts";\n  import { ownerName } from "@/config/site-owner.ts";\n</script>\n<h1 class="probe">Hello {siteName} {ownerName}</h1>\n<Child />\n<style>.probe{color:rgb(255,0,0)}</style>\n';
}

function childSource(color: string): string {
  return `<p class="child">child</p>\n<style>.child{color:${color}}</style>\n`;
}

function siteLayoutSource(): string {
  return '<script>\n  import SiteHeader from "@/components/SiteHeader.svelte";\n</script>\n<SiteHeader />\n';
}

function siteHeaderSource(): string {
  return '<header class="siteHeader">header</header>\n<style>.siteHeader{color:rgb(0,128,0)}</style>\n';
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
        const { default: SiteLayout } = await ctx.loadModule("/src/SiteLayout.svelte");
        const { render } = await ctx.loadModule("svelte/server");
        const rendered = render(Page);
        const layout = render(SiteLayout);
        const ssr = ctx.assets.ssrStylesheets({
          modules: ["/src/Page.svelte", "/src/SiteLayout.svelte"],
        });
        const content = await ctx.assets.stylesheetContent({ stylesheets: ssr.stylesheets });
        const assets = ctx.assets.document({
          islandStyles: ctx.mode === "build" ? ssr.stylesheets : [],
          inlineStyles: content.stylesheets.map((stylesheet) => ({
            key: "critical:" + stylesheet.href,
            content: stylesheet.content,
            attrs: { "data-critical": stylesheet.href },
          })),
        });
        return {
          html: "<!doctype html><html><head>" + assets.headHtml + "</head><body data-render=\\"" + renders + "\\" data-diagnostics=\\"" + ssr.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "\\" data-style-content-diagnostics=\\"" + content.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "\\">" + (layout.html || layout.body || "") + (rendered.html || rendered.body || "") + "</body></html>",
          dependencies: ssr.dependencies,
        };
      },
    },
  ],
};
`;
}

function linkedCssPaths(html: string): string[] {
  return [...html.matchAll(/href="(\/docs\/[^"]+)"/gu)].map((match) =>
    match[1].replaceAll("&amp;", "&"),
  );
}

function closeServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => server.close(() => resolve()));
}
