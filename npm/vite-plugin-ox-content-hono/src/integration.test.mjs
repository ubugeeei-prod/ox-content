import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { build, createServer, normalizePath } from "vite";
import { expect, test, vi } from "vite-plus/test";
import { HONO_MODULES_ID, oxContentHono } from "./vite.js";

test("finds layout imports through Vite and applies publish state only to builds", async () => {
  const root = await mkdtemp(join(import.meta.dirname, ".hono-site-"));
  const files = {
    "vite.config.mjs": `
      import { fileURLToPath } from "node:url";
      import { oxContentHono } from "@ox-content/vite-plugin-hono/vite";
      export default ({ mode }) => ({
        resolve: { alias: { "@snippets": fileURLToPath(new URL("./snippets", import.meta.url)) } },
        plugins: [
          ...oxContentHono({
            layout: "./Layout.tsx",
            oxContent: {
              extensions: ["note"],
              mdx: true,
              publishState: { now: "2026-01-01", includeDrafts: mode === "draft-preview" },
            },
          }),
          {
            name: "fixture:resolve-promo",
            resolveId(id) {
              if (id === "site-promo") return fileURLToPath(new URL("./snippets/Promo.NOTE", import.meta.url));
            },
          },
        ],
      });
    `,
    "Layout.tsx": `
      import Promo from "site-promo";
      import Alias from "@snippets/Alias.note";
      import Relative from "./snippets/Relative.mts";
      export default ({ head, children }) => (
        <html><head>{head}</head><body>{children}<Promo /><Alias /><Relative /></body></html>
      );
    `,
    "content/index.note": "# Published content",
    "content/unlisted.note": "---\nunlisted: true\n---\n# Unlisted content",
    "content/draft.note":
      '---\ndraft: true\n---\n# Draft preview\n\nimport Draft from "./Draft.js";\n\n<Draft />',
    "content/scheduled.note": "---\nscheduled: 2027-01-01\n---\n# Scheduled content",
    "content/Draft.js": 'export default () => "Draft-only island";',
    "content/Promo.js": 'export default () => "Plugin-resolved island";',
    "content/Alias.js": 'export default () => "Aliased island";',
    "content/Relative.js": 'export default () => "Relative layout island";',
    "snippets/Promo.NOTE": 'import Promo from "../content/Promo.js";\n\n<Promo />',
    "snippets/Alias.note": 'import Alias from "../content/Alias.js";\n\n<Alias />',
    "snippets/Relative.mts": 'export { default } from "./Relative.cts";',
    "snippets/Relative.cts": 'export { default } from "./Relative.note";',
    "snippets/Relative.note": 'import Relative from "../content/Relative.js";\n\n<Relative />',
  };
  try {
    await mkdir(join(root, "content"));
    await mkdir(join(root, "snippets"));
    await Promise.all(
      Object.entries(files).map(([name, content]) => writeFile(join(root, name), content)),
    );
    await build({ root, logLevel: "error" });
    const html = await readFile(join(root, "dist/index.html"), "utf8");
    expect(html).toContain("Published content");
    expect(html).toContain("Plugin-resolved island");
    expect(html).toContain("Aliased island");
    expect(html).toContain("Relative layout island");
    expect(html).toContain('data-ox-module="/content/Relative.js"');
    const manifest = await readFile(join(root, "dist/.vite/manifest.json"), "utf8");
    expect(manifest).toContain("content/Promo.js");
    expect(manifest).toContain("content/Alias.js");
    expect(manifest).toContain("content/Relative.js");
    expect(manifest).not.toContain("content/Draft.js");
    await expect(readFile(join(root, "dist/unlisted/index.html"), "utf8")).resolves.toContain(
      "Unlisted content",
    );
    for (const route of ["draft", "scheduled"]) {
      await expect(readFile(join(root, `dist/${route}/index.html`), "utf8")).rejects.toMatchObject({
        code: "ENOENT",
      });
    }
    await build({
      root,
      mode: "draft-preview",
      logLevel: "error",
      build: { outDir: "preview-dist" },
    });
    await expect(readFile(join(root, "preview-dist/draft/index.html"), "utf8")).resolves.toContain(
      "Draft preview",
    );
    const server = await createServer({
      root,
      logLevel: "error",
      server: { port: 0, host: "127.0.0.1", ws: false },
    });
    try {
      await server.listen();
      const url = server.resolvedUrls.local[0];
      const response = await fetch(new URL("/draft/", url));
      expect(response.status).toBe(200);
      expect(await response.text()).toContain("Draft preview");
      const registry = await server.ssrLoadModule("virtual:ox-content-hono/modules");
      expect(Object.keys(registry.default)).toEqual(
        expect.arrayContaining([
          "/content/Draft.js",
          "/content/Promo.js",
          "/content/Alias.js",
          "/content/Relative.js",
        ]),
      );
    } finally {
      await server.close();
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}, 30_000);

test.each([
  { change: "delete", expectedModules: [] },
  { change: "edit", expectedModules: ["/content/After.js"] },
])(
  "refreshes the dev registry after a Markdown $change without a layout",
  async ({ change, expectedModules }) => {
    const root = await mkdtemp(join(import.meta.dirname, ".hono-registry-"));
    const documentUrl = "/snippets/article.mdx";
    const documentPath = join(root, "snippets/article.mdx");
    const refreshPath = join(root, "content/refresh.md");
    const source = 'import Island from "../content/Before.js";\n\n<Island />';
    try {
      await mkdir(join(root, "content"));
      await mkdir(join(root, "snippets"));
      await Promise.all([
        writeFile(documentPath, source),
        writeFile(join(root, "content/Before.js"), 'export default () => "Before";'),
        writeFile(join(root, "content/After.js"), 'export default () => "After";'),
        writeFile(refreshPath, "# Refresh"),
      ]);

      const hotUpdates = new Set();
      let registryTransforms = 0;
      const server = await createServer({
        root,
        configFile: false,
        logLevel: "error",
        server: { host: "127.0.0.1", port: 0, ws: false },
        plugins: [
          ...oxContentHono({ routes: false }),
          {
            name: "fixture:observe-registry-refresh",
            enforce: "post",
            hotUpdate({ file, type }) {
              hotUpdates.add(`${type} ${file}`);
            },
            transform(_source, id) {
              if (id === `\0${HONO_MODULES_ID}`) registryTransforms += 1;
            },
          },
        ],
      });
      try {
        await server.listen();
        const registryKeys = async () =>
          Object.keys((await server.ssrLoadModule(HONO_MODULES_ID)).default);
        const waitForUpdate = (type, file) =>
          vi.waitFor(() => expect(hotUpdates).toContain(`${type} ${normalizePath(file)}`));

        await server.ssrLoadModule(documentUrl);
        expect(await registryKeys()).toEqual(["/content/Before.js"]);
        const transformsBeforeChange = registryTransforms;

        if (change === "delete") {
          await rm(documentPath);
          await waitForUpdate("delete", documentPath);
          // A separate content edit rebuilds the registry independently of unlink invalidation.
          await writeFile(refreshPath, "# Registry refresh");
          await waitForUpdate("update", refreshPath);
        } else {
          await writeFile(documentPath, source.replace("Before.js", "After.js"));
          await waitForUpdate("update", documentPath);
        }

        const updatedKeys = await registryKeys();
        expect(registryTransforms).toBeGreaterThan(transformsBeforeChange);
        expect(updatedKeys).toEqual(expectedModules);
      } finally {
        await server.close();
      }
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  },
  15_000,
);
