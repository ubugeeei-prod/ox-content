import * as fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  build as viteBuild,
  createServer,
  type InlineConfig,
  type Plugin,
  type ViteDevServer,
} from "vite";
import { oxContentCustomHost } from ".";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeHttpServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("oxContentCustomHost", () => {
  it("serves custom routes in dev without a consumer-authored Vite lifecycle", async () => {
    const root = await createProject("ox-custom-host-dev-");
    const server = await trackDevServer(createServer(viteConfig(root, { reloadDebounceMs: 1 })));
    const listener = await listen(server);

    const home = await read(listener.port, "/");
    const feed = await read(listener.port, "/feed.xml");
    const plain = await read(listener.port, "/plain.txt");
    const missing = await read(listener.port, "/custom-missing");
    const css = await read(listener.port, "/src/shared.css", { accept: "text/css" });

    expect(home.status).toBe(200);
    expect(home.headers.get("content-type")).toBe("text/html");
    expect(home.text).toContain("<h1>First</h1>");
    expect(home.text.match(/fixture-transform/g)).toHaveLength(1);
    expect(home.text).toContain('<link rel="stylesheet" href="/src/shared.css">');
    expect(home.text).toContain('<script type="module" src="/src/main.ts"></script>');
    expect(feed.headers.get("content-type")).toBe("application/xml; charset=utf-8");
    expect(feed.text).toBe("<feed><title>First</title></feed>");
    expect(feed.text).not.toContain("fixture-transform");
    expect(plain.headers.get("content-type")).toBe("text/plain");
    expect(plain.text).toBe("plain");
    expect(missing.status).toBe(404);
    expect(missing.text).toBe("custom missing");
    expect(css.status).toBe(200);
    expect(css.headers.get("content-type")).toContain("text/css");
    expect(css.text).toContain(".shared");

    const failed = await read(listener.port, "/flaky");
    expect(failed.status).toBe(500);
    const recovered = await read(listener.port, "/flaky");
    expect(recovered.status).toBe(200);
    expect(recovered.text).toBe("recovered");

    await fs.writeFile(path.join(root, "src", "data.ts"), 'export const title = "Second";\n');
    server.watcher.emit("change", path.join(root, "src", "data.ts"));
    await wait(10);

    const updated = await read(listener.port, "/");
    expect(updated.text).toContain("<h1>Second</h1>");
  });

  it("builds host routes after client assets and coordinates public outputs", async () => {
    const root = await createProject("ox-custom-host-build-");

    await viteBuild(viteConfig(root));

    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");
    const xml = await fs.readFile(path.join(root, "dist", "feed.xml"), "utf8");
    const themeTokens = await fs.readFile(
      path.join(root, "dist", "__ox_theme_tokens__", "theme-tokens.css"),
      "utf8",
    );

    expect(html).toContain("<h1>First</h1>");
    expect(html).toContain('href="/assets/main-');
    expect(html).toContain('href="/__ox_theme_tokens__/theme-tokens.css"');
    expect(html).toContain('<script type="module" src="/assets/main-');
    expect(html).toContain("crossorigin");
    expect(xml).toBe("<feed><title>First</title></feed>");
    expect(themeTokens).toContain("--octc-syntax-keyword");
    expect(await fs.readFile(path.join(root, "dist", "index.md"), "utf8")).toBe("# First\n");
    expect(await fs.readFile(path.join(root, "dist", "_redirects"), "utf8")).toContain(
      "/old-home / 301",
    );
  });

  it("skips custom host generation in test mode unless explicitly enabled", async () => {
    const root = await createProject("ox-custom-host-test-mode-");

    await viteBuild({ ...viteConfig(root), mode: "test" });

    await expect(fs.access(path.join(root, "dist", "index.html"))).rejects.toThrow();
  });

  it("rejects duplicate build output owners", async () => {
    const root = await createProject("ox-custom-host-duplicates-");
    await fs.writeFile(path.join(root, "src", "host.ts"), duplicateHostModuleSource());

    await expect(viteBuild(viteConfig(root))).rejects.toThrow(
      /conflicts with "\/" at .*index\.html/u,
    );
  });
});

function viteConfig(root: string, dev: { reloadDebounceMs?: number } = {}): InlineConfig {
  return {
    root,
    configFile: false,
    appType: "custom",
    logLevel: "silent",
    plugins: [
      htmlTransformMarker(),
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
          redirects: { provider: "netlify" },
          ssg: {
            markdownSource: true,
            siteUrl: "https://example.com",
            siteName: "Example",
          },
        },
        dev,
        themeTokens: {
          theme: {
            tokens: { "syntax-keyword": "#123456", other: "drop" },
            darkTokens: { "syntax-keyword": "#abcdef" },
          },
          include: (name) => name.startsWith("syntax-"),
        },
      }),
    ],
    build: {
      outDir: "dist",
      emptyOutDir: true,
      manifest: true,
      rollupOptions: {
        input: path.join(root, "src", "main.ts"),
      },
    },
  };
}

function htmlTransformMarker(): Plugin {
  return {
    name: "fixture-html-transform",
    transformIndexHtml(html) {
      return html.replace("</head>", '<meta name="fixture-transform" content="1"></head>');
    },
  };
}

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "content", "index.md"), "# First\n");
  await fs.writeFile(path.join(root, "src", "data.ts"), 'export const title = "First";\n');
  await fs.writeFile(path.join(root, "src", "shared.css"), ".shared{color:purple}\n");
  await fs.writeFile(path.join(root, "src", "entry.css"), ".entry{color:green}\n");
  await fs.writeFile(
    path.join(root, "src", "main.ts"),
    'import "./entry.css";\ndocument.documentElement.dataset.client = "ready";\n',
  );
  await fs.writeFile(path.join(root, "src", "host.ts"), hostModuleSource());
  return root;
}

function hostModuleSource(): string {
  return `
let flakyAttempts = 0;

export default {
  routes: [
    {
      path: "/",
      inputPath: "content/index.md",
      source: "# First\\n",
      title: "First",
      aliases: ["/old-home"],
      dependencies: ["src/data.ts"],
      async render(ctx) {
        const data = await ctx.loadModule("/src/data.ts");
        const styles = ctx.mode === "serve"
          ? ["/src/shared.css", ctx.assets.themeTokens?.href].filter(Boolean)
          : [ctx.assets.themeTokens?.href].filter(Boolean);
        const assets = ctx.assets.document({
          head: "<title>" + data.title + "</title>",
          sharedStyles: styles,
          clientEntries: ["src/main.ts"],
          crossorigin: true,
        });
        return {
          html: "<!doctype html><html><head>" + assets.headHtml + "</head><body><h1>" + data.title + "</h1></body></html>",
          inputPath: "content/index.md",
          source: "# " + data.title + "\\n",
          title: data.title,
          dependencies: ["src/data.ts"],
        };
      },
    },
    {
      path: "/feed.xml",
      async render(ctx) {
        const data = await ctx.loadModule("/src/data.ts");
        return { text: "<feed><title>" + data.title + "</title></feed>", contentType: "application/xml; charset=utf-8" };
      },
    },
    {
      path: "/plain.txt",
      render() {
        return new Response("plain", { headers: { "content-type": "text/plain" } });
      },
    },
    {
      path: "/flaky",
      render(ctx) {
        flakyAttempts += 1;
        if (ctx.mode === "serve" && flakyAttempts === 1) {
          throw new Error("flaky");
        }
        return { text: "recovered", contentType: "text/plain" };
      },
    },
  ],
  notFound(ctx) {
    if (new URL(ctx.request.url).pathname === "/custom-missing") {
      return { text: "custom missing", status: 404, contentType: "text/plain" };
    }
  },
};
`;
}

function duplicateHostModuleSource(): string {
  return `
export default {
  routes: [
    { path: "/", render() { return { html: "<h1>one</h1>" }; } },
    { path: "/index.html", render() { return { html: "<h1>two</h1>" }; } },
  ],
};
`;
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
  const address = listener.address() as AddressInfo;
  return { port: address.port };
}

async function read(port: number, requestPath: string, options: { accept?: string } = {}) {
  const headers = options.accept ? { accept: options.accept } : undefined;
  const response = await fetch(`http://127.0.0.1:${port}${requestPath}`, { headers });
  return {
    response,
    status: response.status,
    headers: response.headers,
    text: await response.text(),
  };
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function closeHttpServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
