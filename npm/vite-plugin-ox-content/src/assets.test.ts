import * as fs from "node:fs/promises";
import * as http from "node:http";
import type { AddressInfo } from "node:net";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, createServer, type InlineConfig, type ViteDevServer } from "vite";
import { oxContent, type OxContentOptions } from ".";
import { resolveSelfHostedAssetManifest, writeSelfHostedAssets } from "./assets";
import {
  createCollectionAssetsMiddleware,
  planCollectionAssets,
  writeCollectionAssets,
} from "./collection-assets";
import { resolveOptions } from "./resolve-options";

const fixtureFont = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "fixtures",
  "ox-test-font.woff2",
);
const tempDirs: string[] = [];
const activeServers: ViteDevServer[] = [];
const activeListeners: http.Server[] = [];

afterEach(async () => {
  await Promise.all(activeListeners.splice(0).map((server) => closeHttpServer(server)));
  await Promise.all(activeServers.splice(0).map((server) => server.close().catch(() => {})));
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("self-hosted asset contract", () => {
  it("publishes stable head metadata for custom hosts", () => {
    const manifest = resolveSelfHostedAssetManifest(assetOptions("/docs/"));

    expect(manifest.stylesheets).toEqual([
      "/docs/__ox_icons__/icons.css",
      "/docs/__ox_fonts__/fonts.css",
    ]);
    expect(manifest.preloads).toEqual([
      {
        href: "/docs/__ox_fonts__/ox-test-400-normal-latin.woff2",
        as: "font",
        type: "font/woff2",
        crossorigin: true,
      },
    ]);
    expect(manifest.headTags).toContain(
      '<link rel="stylesheet" href="/docs/__ox_icons__/icons.css">',
    );
    expect(manifest.headTags).toContain('href="/docs/__ox_fonts__/ox-test-400-normal-latin.woff2"');
  });

  it("writes the same icon and font assets without buildSsg", async () => {
    const root = await createProject("ox-assets-write-");
    const outDir = path.join(root, "dist");

    const result = await writeSelfHostedAssets({ options: assetOptions("/", root), root, outDir });

    expect(result.errors).toEqual([]);
    expect(result.files).toEqual(
      expect.arrayContaining([
        path.join(outDir, "__ox_icons__", "icons.css"),
        path.join(outDir, "__ox_fonts__", "fonts.css"),
        path.join(outDir, "__ox_fonts__", "ox-test-400-normal-latin.woff2"),
      ]),
    );
    expect(await fs.readFile(path.join(outDir, "__ox_icons__", "icons.css"), "utf8")).toContain(
      "icon-\\[ox--mark\\]",
    );
    expect(await fs.readFile(path.join(outDir, "__ox_fonts__", "fonts.css"), "utf8")).not.toMatch(
      /fonts\.(?:googleapis|gstatic)\.com/,
    );
  });

  it("serves and builds assets for an ssg-disabled custom Vite host", async () => {
    const root = await createProject("ox-assets-vite-");
    const server = await trackDevServer(createServer(viteConfig(root)));
    const listener = await listen(server);

    const [iconCss, fontCss, fontBytes] = await Promise.all([
      readDevAsset(listener.port, "/__ox_icons__/icons.css"),
      readDevAsset(listener.port, "/__ox_fonts__/fonts.css"),
      readDevAsset(listener.port, "/__ox_fonts__/ox-test-400-normal-latin.woff2"),
    ]);

    expect(iconCss.text).toContain("icon-\\[ox--mark\\]");
    expect(iconCss.text).not.toContain("api.iconify.design");
    expect(fontCss.text).toContain('font-family: "Ox Test"');
    expect(fontCss.text).not.toMatch(/fonts\.(?:googleapis|gstatic)\.com/);
    expect(fontBytes.bytes.length).toBeGreaterThan(0);

    await server.close();
    await viteBuild(viteConfig(root));

    const distIconCss = await fs.readFile(
      path.join(root, "dist", "__ox_icons__", "icons.css"),
      "utf8",
    );
    const distFont = await fs.readFile(
      path.join(root, "dist", "__ox_fonts__", "ox-test-400-normal-latin.woff2"),
    );
    const builtClient = await readBuiltClient(root);

    expect(distIconCss).toContain("icon-\\[ox--mark\\]");
    expect(distFont.length).toBeGreaterThan(0);
    expect(builtClient).toContain("__ox_icons__/icons.css");
    expect(builtClient).toContain("headTags");
  });
});

describe("collection asset contract", () => {
  it("plans, deduplicates, writes, and serves arbitrary collection aliases", async () => {
    const root = await createProject("ox-collection-assets-");
    const showcase = path.join(root, "src", "content", "showcase");
    await fs.mkdir(showcase, { recursive: true });
    await fs.writeFile(path.join(showcase, "project-cover.jpg"), "cover");
    await fs.writeFile(path.join(showcase, "project-copy.jpg"), "cover");

    const manifest = await planCollectionAssets({
      root,
      assets: [
        {
          sourcePath: "src/content/showcase/project-cover.jpg",
          publicPath: ["/works/showcase/assets/project cover.jpg", "/works/showcase/cover.jpg"],
        },
        {
          sourcePath: "src/content/showcase/project-copy.jpg",
          publicPath: "/works/showcase/copy.jpg",
        },
      ],
    });

    expect(manifest.assets).toHaveLength(2);
    expect(manifest.assets[0]).toMatchObject({
      publicPaths: ["/works/showcase/assets/project%20cover.jpg", "/works/showcase/cover.jpg"],
      contentPath: expect.stringMatching(/^\/assets\/content\/[a-f0-9]{64}\.jpg$/),
    });
    expect(manifest.assets[1]?.contentPath).toBe(manifest.assets[0]?.contentPath);

    const outDir = path.join(root, "dist");
    const result = await writeCollectionAssets({ manifest, outDir });
    expect(result.files).toHaveLength(4);
    await expect(
      fs.readFile(path.join(outDir, "works", "showcase", "assets", "project cover.jpg"), "utf8"),
    ).resolves.toBe("cover");
    await expect(
      fs.readFile(path.join(outDir, "works", "showcase", "copy.jpg"), "utf8"),
    ).resolves.toBe("cover");

    const middleware = createCollectionAssetsMiddleware(manifest);
    const server = http.createServer((req, res) => middleware(req, res, () => res.end("missing")));
    activeListeners.push(server);
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address() as AddressInfo;
    const response = await readDevAsset(address.port, "/works/showcase/assets/project%20cover.jpg");
    expect(response.text).toBe("cover");
    expect(response.bytes.length).toBe(5);
  });

  it("rejects unsafe aliases and source paths", async () => {
    const root = await createProject("ox-collection-asset-safety-");
    await fs.mkdir(path.join(root, "src", "content"), { recursive: true });
    await fs.writeFile(path.join(root, "src", "content", "cover.jpg"), "cover");
    await fs.writeFile(path.join(root, "src", "content", "other.jpg"), "other");

    await expect(
      planCollectionAssets({
        root,
        assets: [{ sourcePath: "src/content/cover.jpg", publicPath: "/works/%2e%2e/cover.jpg" }],
      }),
    ).rejects.toThrow("unsafe");
    await expect(
      planCollectionAssets({
        root,
        assets: [{ sourcePath: "../outside.jpg", publicPath: "/works/cover.jpg" }],
      }),
    ).rejects.toThrow("must stay within root");
    await expect(
      planCollectionAssets({
        root,
        assets: [
          { sourcePath: "src/content/cover.jpg", publicPath: "/works/cover.jpg" },
          { sourcePath: "src/content/other.jpg", publicPath: "/works/COVER.jpg" },
        ],
      }),
    ).rejects.toThrow("case-insensitive filesystems");
  });
});

async function createProject(prefix: string): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(root);
  await writeFixtureCollection(root);
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.writeFile(path.join(root, "content", "index.md"), "# Home\n");
  await fs.writeFile(
    path.join(root, "index.html"),
    '<div id="app"></div><script type="module" src="/src/main.js"></script>\n',
  );
  await fs.writeFile(
    path.join(root, "src", "main.js"),
    [
      'import "virtual:ox-content/assets.css";',
      'import { headTags } from "virtual:ox-content/asset-manifest";',
      'document.head.insertAdjacentHTML("beforeend", headTags);',
      'document.querySelector("#app").innerHTML = "<span class=\\"icon-[ox--mark]\\"></span>";',
    ].join("\n"),
  );
  return root;
}

async function writeFixtureCollection(root: string): Promise<void> {
  const dir = path.join(root, "node_modules", "@iconify-json", "ox");
  await fs.mkdir(dir, { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), "{}\n");
  await fs.writeFile(
    path.join(dir, "icons.json"),
    JSON.stringify({
      prefix: "ox",
      width: 24,
      height: 24,
      icons: { mark: { body: '<path fill="currentColor" d="M2 2h20v20H2z"/>' } },
    }),
  );
}

function assetOptions(base = "/", root = process.cwd()) {
  return resolveOptions(assetInputOptions(base, root));
}

function assetInputOptions(base = "/", root = process.cwd()): OxContentOptions {
  return {
    base,
    srcDir: "content",
    icons: { safelist: ["ox:mark"] },
    ssg: {
      enabled: false,
      theme: {
        fonts: {
          sans: {
            family: "Ox Test",
            provider: "local",
            path: fixtureFont,
            weights: [400],
            selfHost: true,
            preload: true,
          },
        },
      },
    },
    outDir: path.join(root, "dist"),
    search: false,
    ogViewer: false,
  };
}

function viteConfig(root: string): InlineConfig {
  return {
    root,
    configFile: false,
    logLevel: "silent",
    appType: "custom",
    plugins: oxContent(assetInputOptions("/", root)),
    build: { outDir: "dist", emptyOutDir: true },
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
  const address = listener.address() as AddressInfo;
  return { port: address.port };
}

async function readDevAsset(port: number, assetPath: string) {
  const response = await fetch(`http://127.0.0.1:${port}${assetPath}`);
  const bytes = new Uint8Array(await response.arrayBuffer());
  expect(response.status).toBe(200);
  return { bytes, text: new TextDecoder().decode(bytes) };
}

async function readBuiltClient(root: string): Promise<string> {
  const assetsDir = path.join(root, "dist", "assets");
  const files = await fs.readdir(assetsDir);
  const jsFile = files.find((file) => file.endsWith(".js"));
  if (!jsFile) {
    throw new Error("Expected a built client asset.");
  }
  return fs.readFile(path.join(assetsDir, jsFile), "utf8");
}

function closeHttpServer(server: http.Server): Promise<void> {
  return new Promise((resolve) => {
    server.close(() => resolve());
  });
}
