import * as fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, createServer, type InlineConfig, type ViteDevServer } from "vite";
import {
  oxContentCustomHost,
  planCollectionAssets,
  type CollectionAssetManifest,
  type OxContentCustomHostCollectionAssetsContext,
} from ".";

const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map(closeHttpServer));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("custom host collection asset manifest context", () => {
  it("shares the configured manifest with build route rendering and the writer", async () => {
    const root = await createProject("ox-custom-host-collection-assets-build-");
    await fs.writeFile(path.join(root, "content", "logo.txt"), "first");
    await fs.writeFile(path.join(root, "src", "host.ts"), manifestRouteHostSource());
    let manifestLoads = 0;

    await viteBuild(
      config(root, {
        manifest: async (ctx) => {
          manifestLoads += 1;
          return collectionManifest(ctx.root);
        },
      }),
    );

    const manifest = await collectionManifest(root);
    const asset = manifest.assets[0]!;
    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");

    expect(html).toContain(asset.contentPath);
    expect(html).toContain("/media/logo.txt");
    expect(await fs.readFile(path.join(root, "dist", "media", "logo.txt"), "utf8")).toBe("first");
    await expect(
      fs.readFile(path.join(root, "dist", asset.contentPath.slice(1)), "utf8"),
    ).resolves.toBe("first");
    expect(manifestLoads).toBe(1);
  });

  it("shares successful dev replans with route rendering and middleware", async () => {
    const root = await createProject("ox-custom-host-collection-assets-dev-");
    const sourcePath = path.join(root, "content", "logo.txt");
    const gatePath = path.join(root, "content", "manifest-gate.txt");
    await fs.writeFile(sourcePath, "first");
    await fs.writeFile(gatePath, "ok\n");
    await fs.writeFile(path.join(root, "src", "host.ts"), manifestRouteHostSource());
    let manifestLoads = 0;

    const server = await trackDevServer(
      createServer(
        config(root, {
          dev: { reloadDebounceMs: 1 },
          manifest: async (ctx) => {
            manifestLoads += 1;
            const gate = await fs.readFile(gatePath, "utf8");
            if (gate.includes("fail")) {
              throw new Error("manifest failed");
            }
            return collectionManifest(ctx.root);
          },
        }),
      ),
    );
    const listener = await listen(server);

    const firstManifest = await collectionManifest(root);
    const firstAsset = firstManifest.assets[0]!;
    expect(await text(listener.port, "/")).toMatchObject({ status: 200 });
    expect((await text(listener.port, "/")).text).toContain(firstAsset.contentPath);
    expect(await text(listener.port, firstAsset.contentPath)).toMatchObject({
      status: 200,
      text: "first",
    });
    const initialLoads = manifestLoads;
    expect(initialLoads).toBeGreaterThanOrEqual(1);

    await fs.writeFile(sourcePath, "second");
    server.watcher.emit("change", sourcePath);

    const secondManifest = await collectionManifest(root);
    const secondAsset = secondManifest.assets[0]!;
    const routeAfterChange = await waitForResponse(listener.port, "/", (response) =>
      response.text.includes(secondAsset.contentPath),
    );
    expect(routeAfterChange).toMatchObject({ status: 200 });
    expect(await text(listener.port, secondAsset.contentPath)).toMatchObject({
      status: 200,
      text: "second",
    });
    expect(manifestLoads).toBeGreaterThan(initialLoads);

    await fs.writeFile(sourcePath, "third");
    await fs.writeFile(gatePath, "fail\n");
    const loadsBeforeFailedReplan = manifestLoads;
    server.watcher.emit("change", sourcePath);
    await waitFor(() => manifestLoads > loadsBeforeFailedReplan);

    const routeAfterFailedReplan = await waitForResponse(listener.port, "/", (response) =>
      response.text.includes(secondAsset.contentPath),
    );
    expect(routeAfterFailedReplan).toMatchObject({ status: 200 });
    const failedLoads = manifestLoads;

    await fs.writeFile(gatePath, "ok\n");
    server.watcher.emit("change", gatePath);

    const thirdManifest = await collectionManifest(root);
    const thirdAsset = thirdManifest.assets[0]!;
    const recoveredRoute = await waitForResponse(listener.port, "/", (response) =>
      response.text.includes(thirdAsset.contentPath),
    );
    expect(recoveredRoute).toMatchObject({ status: 200 });
    expect(await text(listener.port, thirdAsset.contentPath)).toMatchObject({
      status: 200,
      text: "third",
    });
    expect(manifestLoads).toBeGreaterThan(failedLoads);
  });
});

function config(
  root: string,
  input: {
    manifest(context: OxContentCustomHostCollectionAssetsContext): Promise<CollectionAssetManifest>;
    dev?: { reloadDebounceMs?: number };
  },
): InlineConfig {
  return {
    root,
    configFile: false,
    appType: "custom",
    logLevel: "silent",
    plugins: [
      ...oxContentCustomHost({
        host: "./src/host.ts",
        dev: input.dev,
        oxContent: { srcDir: "content", outDir: "dist", docs: false, search: false },
        collectionAssets: {
          manifest: (ctx) => input.manifest(ctx),
          watch: [{ path: "content", kind: "directory" }],
          ownedPrefixes: ["/assets/content"],
        },
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
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "src", "main.ts"), "export {};\n");
  return root;
}

function collectionManifest(root: string): Promise<CollectionAssetManifest> {
  return planCollectionAssets({
    root,
    assets: [{ sourcePath: "content/logo.txt", publicPath: "/media/logo.txt" }],
  });
}

function manifestRouteHostSource(): string {
  return `
export default {
  routes: [
    {
      path: "/",
      async render(ctx) {
        const manifest = await ctx.assets.collectionManifest();
        const asset = manifest?.assets[0];
        return {
          html: "<pre>" + JSON.stringify({
            contentPath: asset?.contentPath,
            publicPaths: asset?.publicPaths ?? [],
          }) + "</pre>",
        };
      },
    },
  ],
};
`;
}

async function trackDevServer(serverPromise: Promise<ViteDevServer>): Promise<ViteDevServer> {
  const server = await serverPromise;
  activeServers.push(server);
  return server;
}

async function listen(server: ViteDevServer): Promise<{ server: http.Server; port: number }> {
  const listener = http.createServer(server.middlewares);
  activeListeners.push(listener);
  await new Promise<void>((resolve) => listener.listen(0, "127.0.0.1", resolve));
  const address = listener.address() as AddressInfo;
  return { server: listener, port: address.port };
}

async function text(port: number, requestPath: string) {
  const response = await fetch(`http://127.0.0.1:${port}${requestPath}`);
  return { status: response.status, text: await response.text() };
}

async function waitForResponse(
  port: number,
  requestPath: string,
  predicate: (response: Awaited<ReturnType<typeof text>>) => boolean,
) {
  let lastResponse: Awaited<ReturnType<typeof text>> | undefined;
  await waitFor(async () => {
    lastResponse = await text(port, requestPath);
    return predicate(lastResponse);
  });
  return lastResponse!;
}

async function waitFor(predicate: () => boolean | Promise<boolean>): Promise<void> {
  const deadline = Date.now() + 1000;
  while (Date.now() < deadline) {
    if (await predicate()) {
      return;
    }
    await wait(20);
  }
  throw new Error("Timed out waiting for custom host collection asset replan.");
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function closeHttpServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
