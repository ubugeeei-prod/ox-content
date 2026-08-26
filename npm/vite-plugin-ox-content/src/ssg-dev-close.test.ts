import * as fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, createServer, type InlineConfig, type ViteDevServer } from "vite";
import { oxContent } from ".";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeHttpServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("SSG dev server close", () => {
  it("does not write SSG output when middleware servers close", async () => {
    const root = await createProject("ox-content-ssg-middleware-close-");
    const first = await trackDevServer(createSsgDevServer(root));
    const second = await trackDevServer(createSsgDevServer(root));

    await first.close();
    await second.close();

    await expectMissing(path.join(root, "site"));
  });

  it("keeps the development SSG middleware available", async () => {
    const root = await createProject("ox-content-ssg-middleware-serve-");
    const server = await trackDevServer(createSsgDevServer(root));
    const listener = await listen(server);

    const response = await fetch(`http://127.0.0.1:${listener.port}/`);
    const html = await response.text();
    await server.close();

    expect(response.status).toBe(200);
    expect(html).toContain("<h1");
    expect(html).toContain("Hello from SSG");
    await expectMissing(path.join(root, "site"));
  });

  it("still writes SSG output during a production build", async () => {
    const root = await createProject("ox-content-ssg-production-build-");

    await viteBuild({
      ...viteConfig(root),
      build: {
        outDir: "client",
        emptyOutDir: false,
      },
    });

    const html = await fs.readFile(path.join(root, "site", "index.html"), "utf8");
    expect(html).toContain("Hello from SSG");
  });
});

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);

  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.writeFile(path.join(root, "content", "index.md"), "# Hello from SSG\n\nDev page.\n");
  await fs.writeFile(
    path.join(root, "index.html"),
    '<div id="app"></div><script type="module" src="/src/main.js"></script>\n',
  );
  await fs.writeFile(
    path.join(root, "src", "main.js"),
    'document.querySelector("#app").textContent = "client";\n',
  );
  return root;
}

function viteConfig(root: string): InlineConfig {
  return {
    root,
    configFile: false,
    logLevel: "silent",
    appType: "custom",
    plugins: oxContent({
      srcDir: "content",
      outDir: "site",
      ssg: true,
      search: false,
      ogViewer: false,
    }),
  };
}

function createSsgDevServer(root: string): Promise<ViteDevServer> {
  return createServer({
    ...viteConfig(root),
    server: { middlewareMode: true },
  });
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

function closeHttpServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}

async function expectMissing(filePath: string): Promise<void> {
  await expect(fs.access(filePath)).rejects.toThrow();
}
