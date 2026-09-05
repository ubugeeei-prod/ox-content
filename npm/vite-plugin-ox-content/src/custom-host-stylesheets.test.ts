import * as fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, createServer, type InlineConfig, type ViteDevServer } from "vite";
import { oxContentCustomHost } from ".";
import {
  resolveCustomHostStylesheets,
  type CustomHostDevModuleNode,
} from "./custom-host-stylesheets";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeHttpServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveCustomHostStylesheets", () => {
  it("collects build CSS in dependency order and dev CSS dependencies", () => {
    const manifest = {
      "src/Island.ts": { css: ["assets/island.css"], imports: ["_child.js"] },
      "_child.js": { css: ["assets/child.css"] },
    };
    expect(
      resolveCustomHostStylesheets({
        modules: ["/src/Island.ts"],
        manifest,
        base: "/docs/",
      }).stylesheets.map((style) => style.href),
    ).toEqual(["/docs/assets/child.css", "/docs/assets/island.css"]);

    const childCss = node("/src/child.module.css?used", [], "/repo/src/child.module.css");
    const island = node("/src/Island.ts", [childCss], "/repo/src/Island.ts");
    const dev = resolveCustomHostStylesheets({
      modules: ["/src/Island.ts", "/src/Missing.ts"],
      moduleGraph: { getModuleById: (id) => (id === "/src/Island.ts" ? island : undefined) },
      base: "/docs/",
      root: "/repo",
    });

    expect(dev.stylesheets).toEqual([
      { kind: "style", href: "/docs/src/child.module.css?used", moduleId: "/src/Island.ts" },
    ]);
    expect(dev.dependencies).toEqual(["/repo/src/Island.ts", "/repo/src/child.module.css"]);
    expect(dev.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-module", moduleId: "/src/Missing.ts" }),
    ]);
  });

  it("reports a missing resolver separately from a module with no CSS", () => {
    expect(resolveCustomHostStylesheets({ modules: ["src/Island.ts"] })).toEqual({
      stylesheets: [],
      dependencies: [],
      diagnostics: [
        {
          code: "missing-resolver",
          moduleId: "src/Island.ts",
          message:
            'No Vite manifest or development module graph was available for "src/Island.ts".',
        },
      ],
    });
  });
});

describe("custom host island stylesheets", () => {
  it("serves dev island CSS through assets context and invalidates CSS dependencies", async () => {
    const root = await createProject("ox-custom-host-style-dev-");
    const server = await trackDevServer(createServer(viteConfig(root, { reloadDebounceMs: 1 })));
    installFixtureModuleGraph(server, root);
    const listener = await listen(server);

    const home = await read(listener.port, "/");
    expect(home.status).toBe(200);
    expect(home.text).toContain('data-render="1"');
    expect(home.text).toContain('<link rel="stylesheet" href="/src/islands/child.css">');
    expect(home.text.match(/href="\/src\/islands\/island\.css"/g)).toHaveLength(1);
    expect(home.text.indexOf('href="/src/islands/child.css"')).toBeLessThan(
      home.text.indexOf('src="/src/main.ts"'),
    );
    expect(home.text).not.toContain("missing-module");

    const cached = await read(listener.port, "/");
    expect(cached.text).toContain('data-render="1"');

    const childCss = path.join(root, "src", "islands", "child.css");
    await fs.writeFile(childCss, ".child{color:navy}\n");
    server.watcher.emit("change", childCss);
    await wait(20);

    const updated = await read(listener.port, "/");
    expect(updated.text).toContain('data-render="2"');
  });

  it("writes build island CSS from the manifest without exposing moduleGraph", async () => {
    const root = await createProject("ox-custom-host-style-build-");

    await viteBuild(viteConfig(root));

    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");
    expect(html).toContain('data-render="1"');
    expect(html).toContain('href="/assets/');
    expect(html).toContain('<script type="module" src="/assets/main-');
    expect(html.indexOf('href="/assets/')).toBeLessThan(
      html.indexOf('<script type="module" src="/assets/main-'),
    );
    expect(html).not.toContain("missing-module");

    const css = await readDistCss(root);
    expect(css).toContain(".island");
    expect(css).toContain(".child");
  });
});

function viteConfig(root: string, dev: { reloadDebounceMs?: number } = {}): InlineConfig {
  return {
    root,
    configFile: false,
    appType: "custom",
    logLevel: "silent",
    plugins: [
      ...oxContentCustomHost({
        host: "./src/host.ts",
        oxContent: {
          srcDir: "content",
          outDir: "dist",
          resources: false,
          docs: false,
          search: false,
          ogViewer: false,
          feeds: false,
          siteMaps: false,
        },
        dev,
      }),
    ],
    build: {
      outDir: "dist",
      emptyOutDir: true,
      manifest: true,
      rollupOptions: { input: path.join(root, "src", "main.ts") },
    },
  };
}

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "src", "islands"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(
    path.join(root, "src", "main.ts"),
    'import("./islands/client.ts");\ndocument.documentElement.dataset.client = "ready";\n',
  );
  await fs.writeFile(
    path.join(root, "src", "islands", "Island.ts"),
    'import { child } from "./Child.ts";\nexport function renderIsland() { return `<section class="island ${child}">Island</section>`; }\n',
  );
  await fs.writeFile(
    path.join(root, "src", "islands", "client.ts"),
    'import "./island.css";\nimport "./child.css";\nexport const hydrate = true;\n',
  );
  await fs.writeFile(
    path.join(root, "src", "islands", "Child.ts"),
    'export const child = "child";\n',
  );
  await fs.writeFile(path.join(root, "src", "islands", "island.css"), ".island{color:teal}\n");
  await fs.writeFile(path.join(root, "src", "islands", "child.css"), ".child{color:maroon}\n");
  await fs.writeFile(path.join(root, "src", "host.ts"), hostModuleSource());
  return root;
}

function hostModuleSource(): string {
  return `
let renders = 0;

export default {
  routes: [
    {
      path: "/",
      async render(ctx) {
        renders += 1;
        const island = await ctx.loadModule("/src/islands/Island.ts");
        const styles = ctx.assets.stylesheets({
          modules: ["/src/islands/client.ts", "/src/islands/client.ts"],
        });
        const assets = ctx.assets.document({
          sharedStyles: ctx.mode === "serve" ? ["/src/islands/island.css"] : [],
          islandStyles: styles.stylesheets,
          clientEntries: ["src/main.ts"],
        });
        return {
          html: "<!doctype html><html><head>" + assets.headHtml + "</head><body data-render=\\"" + renders + "\\"><main>" + island.renderIsland() + "</main><p>" + styles.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "</p></body></html>",
          dependencies: styles.dependencies,
        };
      },
    },
  ],
};
`;
}

function installFixtureModuleGraph(server: ViteDevServer, root: string): void {
  const graph = server.moduleGraph as unknown as {
    getModuleById(id: string): CustomHostDevModuleNode | undefined;
    getModulesByFile?(file: string): Set<CustomHostDevModuleNode> | undefined;
  };
  const childCss = node(
    "/src/islands/child.css",
    [],
    path.join(root, "src", "islands", "child.css"),
  );
  const islandCss = node(
    "/src/islands/island.css",
    [],
    path.join(root, "src", "islands", "island.css"),
  );
  const client = node(
    "/src/islands/client.ts",
    [childCss, islandCss],
    path.join(root, "src", "islands", "client.ts"),
  );
  const originalGetModuleById = graph.getModuleById.bind(graph);
  const originalGetModulesByFile = graph.getModulesByFile?.bind(graph);

  graph.getModuleById = (id: string) =>
    id === "/src/islands/client.ts" ? client : originalGetModuleById(id);
  graph.getModulesByFile = (file: string) =>
    file === path.join(root, "src", "islands", "client.ts")
      ? new Set([client])
      : originalGetModulesByFile?.(file);
}

function node(
  url: string,
  imports: CustomHostDevModuleNode[] = [],
  file?: string,
): CustomHostDevModuleNode {
  return {
    id: url,
    url,
    file,
    importedModules: imports,
  };
}

async function trackDevServer(serverPromise: Promise<ViteDevServer>): Promise<ViteDevServer> {
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

async function readDistCss(root: string): Promise<string> {
  const assetDir = path.join(root, "dist", "assets");
  const files = (await fs.readdir(assetDir)).filter((file) => file.endsWith(".css"));
  return (
    await Promise.all(files.map((file) => fs.readFile(path.join(assetDir, file), "utf8")))
  ).join("\n");
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function closeHttpServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
