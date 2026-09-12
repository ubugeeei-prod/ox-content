import fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createServer, type InlineConfig, type ViteDevServer } from "vite";

const activeListeners: http.Server[] = [];
const activeServers: ViteDevServer[] = [];
const tempDirs: string[] = [];
const PACKAGE_ROOT = path.dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const ISLANDS_ROOT = path.resolve(PACKAGE_ROOT, "../ox-content-islands");
const ISLANDS_ENTRY = path.join(ISLANDS_ROOT, "src", "index.ts");
const ISLANDS_HTML_HOST_ENTRY = path.join(ISLANDS_ROOT, "src", "html-host.ts");

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("Svelte HTML host development runtime", () => {
  it.each([
    ["official Svelte", "@sveltejs/vite-plugin-svelte"],
    ["rsvelte", "@rsvelte/vite-plugin-svelte"],
  ] as const)(
    "renders islands inside the Vite SSR runtime with %s",
    async (_name, plugin) => {
      const root = await createProject("ox-svelte-html-host-runtime-");
      await writeFile(root, "vite.config.mjs", viteConfigSource(plugin));
      await writeFile(root, "src/host.ts", hostModuleSource());
      const server = await trackServer(createServer(viteConfig(root)));
      const listener = await listen(server);

      const response = await read(listener.port, "/docs/probe");

      expect(response.status, response.text).toBe(200);
      expect(response.text).toContain('data-diagnostics=""');
      expect(response.text).toContain('data-host-runtime-loaded="false"');
      expect(response.text).toContain('data-role="probe"');
      expect(response.text).toContain(">ready</button>");
      expect(response.text).toContain("<pre>/src/Probe.svelte</pre>");
    },
    30_000,
  );
});

function viteConfig(root: string): InlineConfig {
  return {
    root,
    configFile: path.join(root, "vite.config.mjs"),
  };
}

function viteConfigSource(sveltePackage: string): string {
  return [
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
    "      dev: { transformHtml: false },",
    "    }),",
    "  ],",
    "  resolve: {",
    "    alias: [",
    `      { find: "@ox-content/islands/html-host", replacement: ${JSON.stringify(ISLANDS_HTML_HOST_ENTRY)} },`,
    `      { find: "@ox-content/islands", replacement: ${JSON.stringify(ISLANDS_ENTRY)} },`,
    "    ],",
    "  },",
    `  server: { fs: { allow: [root, ${JSON.stringify(PACKAGE_ROOT)}, ${JSON.stringify(ISLANDS_ROOT)}] } },`,
    "  build: { rollupOptions: { input: path.join(root, 'src', 'entry.js') } },",
    "};",
    "",
  ].join("\n");
}

function hostModuleSource(): string {
  const packageEntry = `/@fs/${path.join(PACKAGE_ROOT, "src", "index.ts").replace(/\\/g, "/")}`;
  return `
import path from "node:path";
import { createSvelteHtmlHostRenderer } from ${JSON.stringify(packageEntry)};

const loaded = [];

export default {
  routes: [
    {
      path: "/probe",
      async render(ctx) {
        const renderIslands = createSvelteHtmlHostRenderer({
          root: ctx.root,
          loadModule: async (moduleId) => {
            const viteModuleId = toViteModuleId(moduleId, ctx.root);
            loaded.push(viteModuleId);
            return ctx.loadModule(viteModuleId);
          },
        });
        const result = await renderIslands(
          [
            '<div data-ox-island="Probe">',
            '<script type="application/json">{"props":{"label":"ready"},"expressions":{},"spreads":[]}</script>',
            "</div>",
          ].join(""),
          {
            documentPath: path.join(ctx.root, "content", "probe.mdx"),
            components: { Probe: "./src/Probe.svelte" },
          },
        );
        const diagnostics = result.diagnostics.map((diagnostic) => diagnostic.code).join(",");
        const hostRuntimeLoaded = loaded.includes("svelte/server") && loaded.includes("svelte");
        return {
          html:
            '<!doctype html><html><body data-diagnostics="' +
            diagnostics +
            '" data-host-runtime-loaded="' +
            String(hostRuntimeLoaded) +
            '"><pre>' +
            loaded.slice(-2).join("\\n") +
            "</pre>" +
            result.html +
            "</body></html>",
          dependencies: [path.join(ctx.root, "src", "Probe.svelte")],
        };
      },
    },
  ],
};

function toViteModuleId(moduleId, root) {
  if (!path.isAbsolute(moduleId) || !moduleId.startsWith(root)) {
    return moduleId;
  }
  return "/" + path.relative(root, moduleId).replace(/\\\\/g, "/");
}
`;
}

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(PACKAGE_ROOT, `.tmp-${prefix}`));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await writeFile(root, "package.json", '{"type":"module"}\n');
  await writeFile(root, "src/entry.js", "document.body.dataset.client = 'unused';\n");
  await writeFile(
    root,
    "src/Probe.svelte",
    [
      "<script>",
      "  export let label = '';",
      "</script>",
      '<button data-role="probe">{label}</button>',
      "",
    ].join("\n"),
  );
  return root;
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
  return { status: response.status, text: await response.text() };
}

function closeServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
