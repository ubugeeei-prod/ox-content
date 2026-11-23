import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { build, createServer } from "vite";
import { expect, test } from "vite-plus/test";

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
      import Relative from "./snippets/Relative.note";
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
    "snippets/Promo.NOTE": 'import Promo from "../content/Promo.js";\n\n<Promo />',
    "snippets/Alias.note": 'import Alias from "../content/Alias.js";\n\n<Alias />',
    "snippets/Relative.note": "# Relative layout import",
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
    expect(html).toContain("Relative layout import");
    const manifest = await readFile(join(root, "dist/.vite/manifest.json"), "utf8");
    expect(manifest).toContain("content/Promo.js");
    expect(manifest).toContain("content/Alias.js");
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
        expect.arrayContaining(["/content/Draft.js", "/content/Promo.js", "/content/Alias.js"]),
      );
    } finally {
      await server.close();
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}, 30_000);
