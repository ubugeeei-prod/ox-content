import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { codePlay } from "./plugin";

describe("codePlay SSG HTML enhance", () => {
  it("wraps written SSG fences and injects the browser client script", async () => {
    const root = mkdtempSync(path.join(tmpdir(), "ox-code-play-ssg-"));
    const srcDir = path.join(root, "content");
    const outDir = path.join(root, "dist");
    mkdirSync(srcDir, { recursive: true });
    mkdirSync(outDir, { recursive: true });
    writeFileSync(path.join(srcDir, "index.md"), "```js play\nconsole.log(42);\n```\n");
    writeFileSync(
      path.join(outDir, "index.html"),
      `<pre><code class="language-js">console.log(42);</code></pre>\n`,
    );

    const plugin = codePlay({
      languages: { javascript: true },
      srcDir: "content",
      outDir: "dist",
    });
    const configResolved = hookFn(plugin.configResolved) as (config: {
      command: string;
      root: string;
      base: string;
      build: { outDir: string };
    }) => void;
    configResolved({ command: "build", root, base: "/", build: { outDir: "dist" } });
    await hookFn(plugin.closeBundle)();

    const html = readFileSync(path.join(outDir, "index.html"), "utf8");
    expect(html).toContain("data-ox-code-play");
    expect(html).toContain("<ox-code-play");
    expect(html).toContain("ox-code-play.js");
    expect(html).toContain('type="module"');
  });
});

function hookFn(hook: unknown): (...args: never[]) => unknown {
  if (typeof hook === "function") {
    return hook as (...args: never[]) => unknown;
  }
  if (hook && typeof hook === "object" && "handler" in hook && typeof hook.handler === "function") {
    return hook.handler as (...args: never[]) => unknown;
  }
  throw new Error("expected a plugin hook");
}
