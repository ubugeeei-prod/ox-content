import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build } from "vite";
import { oxContentCustomHost } from ".";
import { createRgba, encodePng, pngSize } from "./resources-image";

const projects: string[] = [];
afterEach(async () => {
  await Promise.all(
    projects.splice(0).map((root) => fs.rm(root, { recursive: true, force: true })),
  );
});

describe("custom host byte HTML outputs", () => {
  it("preserves UTF-8 response bytes and typed-array offsets without resources", async () => {
    const html = "<!doctype html><html><body>雪🌸<img src='./pixel.png'></body></html>";
    const expected = new TextEncoder().encode(html);
    const bytes = new Uint8Array(expected.length + 2);
    bytes.set(expected, 1);
    const view = bytes.subarray(1, -1);
    const root = await fixture();
    await generate(root, view);
    expect(await fs.readFile(path.join(root, "dist", "bytes", "index.html"))).toEqual(
      Buffer.from(expected),
    );
    expect(await fs.readFile(path.join(root, "dist", "response", "index.html"))).toEqual(
      Buffer.from(expected),
    );
    await expect(fs.access(path.join(root, "dist", "bytes", "pixel.png"))).rejects.toThrow();
  });

  it("still decodes byte HTML for enabled resource transforms", async () => {
    const root = await fixture();
    await fs.writeFile(
      path.join(root, "content", "pixel.png"),
      encodePng(createRgba(2, 2, () => [255, 0, 0, 255])),
    );
    const html = '<!doctype html><html><body>雪🌸<img src="./pixel.png?width=1"></body></html>';
    await generate(root, new TextEncoder().encode(html), { resources: true });
    const output = await fs.readFile(path.join(root, "dist", "bytes", "index.html"), "utf8");
    expect(output).toContain("雪🌸");
    expect(output).not.toContain("?width=1");
    const images = (await fs.readdir(path.join(root, "dist", "bytes"))).filter((name) =>
      name.endsWith(".png"),
    );
    expect(images).toHaveLength(1);
    expect(pngSize(await fs.readFile(path.join(root, "dist", "bytes", images[0]!)))).toEqual({
      width: 1,
      height: 1,
    });
  });

  it("still decodes byte HTML when final HTML minification is requested", async () => {
    const root = await fixture();
    const html = `<!doctype html>
<html>
  <body>
    <!-- build comment -->
    <main>雪🌸</main>
  </body>
</html>`;
    await generate(root, new TextEncoder().encode(html), { minifyHtml: true });
    const output = await fs.readFile(path.join(root, "dist", "bytes", "index.html"), "utf8");
    expect(output).toContain("<main>雪🌸</main>");
    expect(output).not.toContain("build comment");
    expect(output.length).toBeLessThan(html.length);
  });
});

async function fixture(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-custom-host-byte-build-"));
  projects.push(root);
  await fs.mkdir(path.join(root, "src"));
  await fs.mkdir(path.join(root, "content"));
  await fs.writeFile(path.join(root, "package.json"), '{"type":"module"}\n');
  await fs.writeFile(path.join(root, "src", "main.js"), "export {};\n");
  await fs.writeFile(path.join(root, "content", "page.md"), "# Page\n");
  return root;
}

async function generate(
  root: string,
  bytes: Uint8Array,
  options: { resources?: boolean; minifyHtml?: boolean } = {},
): Promise<void> {
  await build({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: oxContentCustomHost({
      host: {
        routes: [
          {
            path: "/bytes/",
            inputPath: "content/page.md",
            render: () => ({ body: bytes, contentType: "text/html" }),
          },
          {
            path: "/response/",
            inputPath: "content/page.md",
            render: () =>
              new Response(Uint8Array.from(bytes), { headers: { "content-type": "text/html" } }),
          },
        ],
      },
      oxContent: {
        srcDir: "content",
        outDir: "dist",
        docs: false,
        search: false,
        resources: options.resources ?? false,
        ssg: { lastUpdated: false, generateOgImage: false },
      },
      build: { enabled: true, transformHtml: false, minifyHtml: options.minifyHtml ?? false },
    }),
    build: {
      outDir: "dist",
      manifest: true,
      rollupOptions: { input: path.join(root, "src", "main.js") },
    },
  });
}
