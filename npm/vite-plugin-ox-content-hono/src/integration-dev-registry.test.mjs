import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { createServer, normalizePath } from "vite";
import { expect, test, vi } from "vite-plus/test";
import { HONO_MODULES_ID, oxContentHono } from "./vite.js";

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
