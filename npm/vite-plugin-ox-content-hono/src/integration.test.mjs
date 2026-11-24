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

const importedMarkdown = (island) => `import Island from "../content/${island}.js";\n\n<Island />`;

async function withImportedMarkdownSite(run, oxContent) {
  const root = await mkdtemp(join(import.meta.dirname, ".hono-imported-"));
  try {
    await mkdir(join(root, "content"));
    await mkdir(join(root, "snippets"));
    await Promise.all([
      writeFile(join(root, "snippets/a.mdx"), importedMarkdown("First")),
      writeFile(join(root, "snippets/b.mdx"), importedMarkdown("Second")),
      writeFile(join(root, "content/First.js"), 'export default () => "First";'),
      writeFile(join(root, "content/Second.js"), 'export default () => "Second";'),
    ]);
    const server = await createServer({
      root,
      configFile: false,
      logLevel: "error",
      server: { watch: null, ws: false },
      plugins: oxContentHono({ routes: false, oxContent }),
    });
    try {
      await run(server, root);
    } finally {
      await server.close();
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("updates the cached client registry after SSR imports another document", async () => {
  await withImportedMarkdownSite(async (server, root) => {
    const client = server.environments.client;
    const registry = () => client.transformRequest(HONO_MODULES_ID);

    await server.ssrLoadModule("/snippets/a.mdx");
    expect((await registry()).code).toContain('"/content/First.js": () => import(');

    const { default: Markdown } = await server.ssrLoadModule("/snippets/b.mdx");
    expect(String(await Markdown())).toContain('data-ox-module="/content/Second.js"');
    const updated = await registry();
    expect(updated.code).toContain('"/content/First.js": () => import(');
    expect(updated.code).toContain('"/content/Second.js": () => import(');

    server.environments.ssr.moduleGraph.onFileChange(normalizePath(join(root, "snippets/b.mdx")));
    await server.ssrLoadModule("/snippets/b.mdx");
    expect(await registry()).toBe(updated);
  });
});

test("retries discovery when another SSR document arrives during a registry load", async () => {
  const resume = Promise.withResolvers();
  let firstDocumentRenders = 0;
  await withImportedMarkdownSite(
    async (server) => {
      await server.ssrLoadModule("/snippets/a.mdx");
      const pending = server.environments.client.transformRequest(HONO_MODULES_ID);
      try {
        await vi.waitFor(() => expect(firstDocumentRenders).toBe(2));
        await server.ssrLoadModule("/snippets/b.mdx");
        resume.resolve();
        expect((await pending).code).toContain('"/content/Second.js": () => import(');
      } finally {
        resume.resolve();
        await pending.catch(() => {});
      }
    },
    {
      transformers: [
        {
          name: "pause-registry-discovery",
          async transform(ast, { filePath }) {
            if (
              normalizePath(filePath).endsWith("/snippets/a.mdx") &&
              ++firstDocumentRenders === 2
            ) {
              await resume.promise;
            }
            return ast;
          },
        },
      ],
    },
  );
});

test("keeps the latest SSR snapshot when an older Markdown compile finishes last", async () => {
  const resume = Promise.withResolvers();
  let renders = 0;
  await withImportedMarkdownSite(
    async (server, root) => {
      const url = "/snippets/a.mdx";
      const documentPath = join(root, url.slice(1));
      const ssr = server.environments.ssr;
      const older = ssr.transformRequest(url);
      try {
        await vi.waitFor(() => expect(renders).toBe(1));
        await writeFile(documentPath, importedMarkdown("Second"));
        ssr.moduleGraph.onFileChange(normalizePath(documentPath));
        const newer = await ssr.transformRequest(url);

        resume.resolve();
        expect((await older).code).toContain("/content/First.js");
        expect(await ssr.transformRequest(url)).toBe(newer);
        const { default: Markdown } = await server.ssrLoadModule(url);
        expect(String(await Markdown())).toContain('data-ox-module="/content/Second.js"');

        const registry = await server.environments.client.transformRequest(HONO_MODULES_ID);
        expect(registry.code).toContain('"/content/Second.js": () => import(');
        expect(registry.code).not.toContain('"/content/First.js": () => import(');
      } finally {
        resume.resolve();
        await older.catch(() => {});
      }
    },
    {
      transformers: [
        {
          name: "pause-older-markdown-compile",
          async transform(ast) {
            if (++renders === 1) await resume.promise;
            return ast;
          },
        },
      ],
    },
  );
});

test.each([
  { change: "delete", expectedModules: [] },
  { change: "edit", expectedModules: ["/content/After.js"] },
  { change: "rewrite", expectedModules: ["/content/After.js"] },
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
          {
            name: "fixture:rewrite-ssr-markdown-import",
            enforce: "pre",
            transform(markdown, id, options) {
              if (change === "rewrite" && options?.ssr && id === normalizePath(documentPath)) {
                return markdown.replace("Before.js", "After.js");
              }
            },
          },
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

        const { default: Markdown } = await server.ssrLoadModule(documentUrl);
        const initialKeys = await registryKeys();
        if (change === "rewrite") {
          expect(String(await Markdown())).toContain('data-ox-module="/content/After.js"');
          expect(await readFile(documentPath, "utf8")).toBe(source);
        } else {
          expect(initialKeys).toEqual(["/content/Before.js"]);
        }
        const transformsBeforeChange = registryTransforms;

        if (change === "delete") {
          await rm(documentPath);
          await waitForUpdate("delete", documentPath);
        }
        if (change !== "edit") {
          // Rebuild independently of unlink invalidation or another SSR transform.
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
