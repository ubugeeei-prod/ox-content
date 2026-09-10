import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild, type InlineConfig } from "vite";
import { oxContentCustomHost } from ".";
import { resolveCustomHostStylesheetContent } from "./custom-host-stylesheet-content";
import { resolveCustomHostStylesheets } from "./custom-host-stylesheets";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveCustomHostStylesheetContent", () => {
  it("reads build CSS artifacts in resolved descriptor order", async () => {
    const outDir = await createTempDir("ox-custom-host-style-content-unit-");
    await fs.mkdir(path.join(outDir, "assets"), { recursive: true });
    await fs.writeFile(path.join(outDir, "assets", "page.css"), ".page{color:green}\n");
    await fs.writeFile(path.join(outDir, "assets", "child.css"), ".child{color:maroon}\n");
    const styles = resolveCustomHostStylesheets({
      modules: ["/src/page.css", "/src/client.ts"],
      root: "/repo",
      base: "/docs/",
      manifest: {
        "src/page.css": { file: "assets/page.css", src: "src/page.css" },
        "src/client.ts": { file: "assets/client.js", css: ["assets/child.css"] },
      },
    });

    const result = await resolveCustomHostStylesheetContent({
      build: true,
      outDir,
      stylesheets: styles.stylesheets,
    });

    expect(result.stylesheets.map((stylesheet) => stylesheet.href)).toEqual([
      "/docs/assets/page.css",
      "/docs/assets/child.css",
    ]);
    expect(result.stylesheets.map((stylesheet) => stylesheet.content)).toEqual([
      ".page{color:green}\n",
      ".child{color:maroon}\n",
    ]);
    expect(result.diagnostics).toEqual([]);
  });

  it("reports explicit diagnostics outside build artifacts", async () => {
    const outDir = await createTempDir("ox-custom-host-style-content-diagnostic-");
    const dev = await resolveCustomHostStylesheetContent({
      build: false,
      outDir,
      stylesheets: [{ kind: "style", href: "/src/page.css", moduleId: "/src/page.css" }],
    });
    const missing = await resolveCustomHostStylesheetContent({
      build: true,
      outDir,
      stylesheets: [
        {
          kind: "style",
          href: "/assets/missing.css",
          moduleId: "/src/page.css",
          outputPath: "assets/missing.css",
        },
      ],
    });

    expect(dev.diagnostics).toEqual([
      expect.objectContaining({ code: "unavailable", href: "/src/page.css" }),
    ]);
    expect(missing.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-artifact", href: "/assets/missing.css" }),
    ]);
  });

  it("uses the dev stylesheet content resolver before reporting unavailable CSS", async () => {
    const outDir = await createTempDir("ox-custom-host-style-content-dev-resolver-");
    const result = await resolveCustomHostStylesheetContent({
      build: false,
      outDir,
      resolveDevContent: async (href) =>
        href === "/src/page.css" ? ".page{color:red}\n" : undefined,
      stylesheets: [{ kind: "style", href: "/src/page.css", moduleId: "/src/page.css" }],
    });

    expect(result.stylesheets).toEqual([
      {
        stylesheet: { kind: "style", href: "/src/page.css", moduleId: "/src/page.css" },
        href: "/src/page.css",
        moduleId: "/src/page.css",
        content: ".page{color:red}\n",
      },
    ]);
    expect(result.diagnostics).toEqual([]);
  });
});

describe("custom host stylesheet content", () => {
  it("exposes build stylesheet bytes through the asset context", async () => {
    const root = await createProject("ox-custom-host-style-content-build-");

    await viteBuild(viteConfig(root));

    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");
    expect(html).toContain('href="/docs/assets/');
    expect(html).toContain("<style>.page{color:green}");
    expect(html).toContain(".child{color:maroon}");
    expect(html).toContain('data-style-content-diagnostics=""');
    expect(html).not.toContain("missing-artifact");
  });
});

function viteConfig(root: string): InlineConfig {
  return {
    root,
    base: "/docs/",
    configFile: false,
    appType: "custom",
    logLevel: "silent",
    plugins: [
      ...oxContentCustomHost({
        host: "./src/host.ts",
        oxContent: {
          base: "/docs/",
          srcDir: "content",
          outDir: "dist",
          resources: false,
          docs: false,
          search: false,
          ogViewer: false,
          feeds: false,
          siteMaps: false,
        },
      }),
    ],
    build: {
      outDir: "dist",
      emptyOutDir: true,
      manifest: true,
      rollupOptions: {
        input: {
          client: path.join(root, "src", "client.ts"),
          page: path.join(root, "src", "page.css"),
        },
      },
    },
  };
}

async function createProject(prefix: string): Promise<string> {
  const root = await createTempDir(prefix);
  await fs.mkdir(path.join(root, "src"), { recursive: true });
  await fs.mkdir(path.join(root, "content"), { recursive: true });
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "src", "client.ts"), 'import "./child.css";\n');
  await fs.writeFile(path.join(root, "src", "page.css"), ".page{color:green}\n");
  await fs.writeFile(path.join(root, "src", "child.css"), ".child{color:maroon}\n");
  await fs.writeFile(path.join(root, "src", "host.ts"), hostModuleSource());
  return root;
}

function hostModuleSource(): string {
  return `
export default {
  routes: [
    {
      path: "/",
      async render(ctx) {
        const styles = ctx.assets.stylesheets({ modules: ["/src/page.css", "/src/client.ts"] });
        const content = await ctx.assets.stylesheetContent({ stylesheets: styles.stylesheets });
        const assets = ctx.assets.document({
          islandStyles: styles.stylesheets,
          inlineStyles: content.stylesheets.map((stylesheet) => ({
            key: "critical:" + stylesheet.href,
            content: stylesheet.content,
          })),
          clientEntries: ["src/client.ts"],
        });
        return {
          html: "<!doctype html><html><head>" + assets.headHtml + "</head><body data-style-content-diagnostics=\\"" + content.diagnostics.map((diagnostic) => diagnostic.code).join(",") + "\\"><main>Home</main></body></html>",
        };
      },
    },
  ],
};
`;
}

async function createTempDir(prefix: string): Promise<string> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}
