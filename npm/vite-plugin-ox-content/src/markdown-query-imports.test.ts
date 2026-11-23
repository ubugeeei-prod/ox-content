import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { pathToFileURL } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { build as viteBuild } from "vite";
import { oxContent } from ".";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("Markdown imports with a Vite query", () => {
  it("leaves ?raw to Vite and still compiles plain Markdown imports", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-md-query-"));
    tempDirs.push(root);
    await fs.mkdir(path.join(root, "src"));
    await fs.writeFile(path.join(root, "src", "hello.md"), "# Hello\n");
    await fs.writeFile(
      path.join(root, "src", "main.js"),
      [
        'import raw from "./hello.md?raw";',
        'import compiled from "./hello.md";',
        "export { raw, compiled };",
      ].join("\n"),
    );

    await viteBuild({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: oxContent({ srcDir: "src", ssg: { enabled: false }, search: false, docs: false }),
      build: { ssr: "src/main.js", outDir: "dist", emptyOutDir: true },
    });

    const output = (await import(pathToFileURL(path.join(root, "dist", "main.mjs")).href)) as {
      raw: unknown;
      compiled: { html: string };
    };
    expect(output.raw).toBe("# Hello\n");
    expect(output.compiled.html).toContain("Hello</h1>");
  });
});
