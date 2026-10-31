import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build } from "vite";
import { oxContentCustomHost } from ".";
import type { OxContentCustomHostOptions } from "./custom-host-types";

const projects: string[] = [];
const html = "<!doctype html><html><body>雪🌸</body></html>";

afterEach(async () => {
  await Promise.all(
    projects.splice(0).map((root) => fs.rm(root, { recursive: true, force: true })),
  );
});

describe("custom host build loader lifecycle", () => {
  it.each(["byte HTML", "HTML with transforms disabled"])(
    "does not start the loader for inline %s",
    async (kind) => {
      const root = await fixture();
      await generate(root, {
        host: {
          routes: [
            {
              path: "/",
              render: () => ({
                body: kind === "byte HTML" ? new TextEncoder().encode(html) : html,
                contentType: "text/html",
              }),
            },
          ],
        },
        build: { transformHtml: kind === "byte HTML" },
      });
      expect(await output(root)).toBe(html);
      expect(await lifecycle(root)).toEqual([]);
    },
  );

  it("shares one loader between concurrent module requests and closes it", async () => {
    const root = await fixture();
    await generate(root, {
      host: {
        async routes(context) {
          const modules = await Promise.all([
            context.loadModule("/src/message.js"),
            context.loadModule("/src/message.js"),
          ]);
          expect(modules[0]).toBe(modules[1]);
          expect(modules[0]).toMatchObject({ message: "雪🌸" });
          return [{ path: "/", render: () => ({ html }) }];
        },
      },
      build: { transformHtml: false },
    });
    expect(await output(root)).toBe(html);
    expect((await lifecycle(root)).filter((event) => event === "open")).toHaveLength(1);
    expect(await lifecycle(root)).toContain("close");
  });

  it("loads file hosts through the loader", async () => {
    const root = await fixture();
    await fs.writeFile(
      path.join(root, "host.js"),
      `export default { routes: [{ path: '/', render: () => ({ html: ${JSON.stringify(html)} }) }] };`,
    );
    await generate(root, { host: "./host.js", build: { transformHtml: false } });
    expect(await output(root)).toBe(html);
    expect((await lifecycle(root)).filter((event) => event === "open")).toHaveLength(1);
    expect(await lifecycle(root)).toContain("close");
  });

  it("starts the loader when string HTML needs Vite transforms", async () => {
    const root = await fixture();
    await generate(root, { host: { routes: [{ path: "/", render: () => ({ html }) }] } });
    expect(await output(root)).toContain('<meta name="loader-transform" content="yes">');
    expect(await output(root)).toContain("雪🌸");
    expect((await lifecycle(root)).filter((event) => event === "open")).toHaveLength(1);
    expect(await lifecycle(root)).toContain("close");
  });

  it("closes a started loader when route discovery fails", async () => {
    const root = await fixture();
    await expect(
      generate(root, {
        host: {
          async routes(context) {
            await context.loadModule("/src/message.js");
            throw new Error("route discovery failed");
          },
        },
      }),
    ).rejects.toThrow("route discovery failed");
    expect(await lifecycle(root)).toContain("close");
  });
});

async function fixture(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-custom-host-build-loader-"));
  projects.push(root);
  await fs.mkdir(path.join(root, "src"));
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "src", "main.js"), "export {};\n");
  await fs.writeFile(path.join(root, "src", "message.js"), "export const message = '雪🌸';\n");
  await fs.writeFile(
    path.join(root, "vite.config.mjs"),
    `import { appendFileSync } from 'node:fs';
const record = (event) => appendFileSync(${JSON.stringify(path.join(root, "lifecycle.txt"))}, event + '\\n');
export default { plugins: [{
  name: 'loader-lifecycle-observer', apply: 'serve',
  configResolved() { record('open'); },
  transformIndexHtml() { return [{ tag: 'meta', attrs: { name: 'loader-transform', content: 'yes' }, injectTo: 'head' }]; },
  closeBundle() { record('close'); }
}] };
`,
  );
  return root;
}

async function generate(root: string, input: OxContentCustomHostOptions): Promise<void> {
  await build({
    root,
    configFile: path.join(root, "vite.config.mjs"),
    logLevel: "silent",
    plugins: oxContentCustomHost({
      ...input,
      oxContent: {
        docs: false,
        search: false,
        resources: false,
        ssg: { lastUpdated: false, generateOgImage: false },
      },
      build: { enabled: true, minifyHtml: false, ...input.build },
    }),
    build: {
      outDir: "dist",
      manifest: true,
      rollupOptions: { input: path.join(root, "src", "main.js") },
    },
  });
}

async function output(root: string): Promise<string> {
  return fs.readFile(path.join(root, "dist", "index.html"), "utf8");
}

async function lifecycle(root: string): Promise<string[]> {
  return fs.readFile(path.join(root, "lifecycle.txt"), "utf8").then(
    (events) => events.trim().split("\n"),
    (error: NodeJS.ErrnoException) => {
      if (error.code === "ENOENT") return [];
      throw error;
    },
  );
}
