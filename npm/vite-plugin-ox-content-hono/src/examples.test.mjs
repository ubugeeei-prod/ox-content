import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import { expect, test } from "vite-plus/test";

test.each(["integ-hono", "integ-hono-custom-host"])(
  "%s builds SSR HTML, island CSS, and a browser-only entry",
  async (example) => {
    const root = fileURLToPath(new URL(`../../../examples/${example}/`, import.meta.url));
    const outDir = await mkdtemp(join(tmpdir(), "ox-content-hono-"));
    try {
      const moduleIds = new Set();
      await build({
        root,
        logLevel: "error",
        build: { outDir },
        plugins: [
          {
            name: "test:hono-browser-modules",
            generateBundle(_options, bundle) {
              if (this.environment.name !== "client") return;
              for (const entry of Object.values(bundle)) {
                if (entry.type !== "chunk") continue;
                for (const id of Object.keys(entry.modules)) moduleIds.add(id);
              }
            },
          },
        ],
      });
      const html = await readFile(join(outDir, "index.html"), "utf8");
      expect(html).toContain("data-ox-hono-context");
      expect(html).toContain("<strong>bold</strong>");
      expect(html).toMatch(/<link[^>]+\.css/);
      expect(html).toMatch(/<script[^>]+type="module"[^>]+\.js/);
      expect(html).not.toContain("/@vite/client");
      expect(html).not.toContain("never sent to client");
      if (example === "integ-hono") {
        expect(html).toContain("Imported MDX");
      }
      expect([...moduleIds].some((id) => id.includes("Counter.tsx"))).toBe(true);
      expect([...moduleIds].join("\n")).not.toMatch(
        /\/node_modules\/(?:@ox-content\/(?:vite-plugin|napi)\/|vite\/|parse5\/)|\/npm\/vite-plugin-ox-content\/|\/crates\/ox_content_napi\//,
      );
    } finally {
      await rm(outDir, { recursive: true, force: true });
    }
  },
  30_000,
);
