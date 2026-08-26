import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { decodePayload } from "./payload";
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
    expect(html).toContain(" inert>");
    expect(html).toContain("ox-code-play.js");
    expect(html).toContain('type="module"');
  });

  it("does not inject ox-code-play.js on a page without play widgets", async () => {
    const root = mkdtempSync(path.join(tmpdir(), "ox-code-play-ssg-plain-"));
    const srcDir = path.join(root, "content");
    const outDir = path.join(root, "dist");
    mkdirSync(srcDir, { recursive: true });
    mkdirSync(outDir, { recursive: true });
    writeFileSync(path.join(srcDir, "index.md"), "# Plain\n\nHello.\n");
    writeFileSync(path.join(outDir, "index.html"), `<p>Hello.</p>\n`);

    const plugin = codePlay({
      languages: { javascript: true },
      srcDir: "content",
      outDir: "dist",
    });
    hookFn(plugin.configResolved)({
      command: "build",
      root,
      base: "/",
      build: { outDir: "dist" },
    } as never);
    await hookFn(plugin.closeBundle)();

    const html = readFileSync(path.join(outDir, "index.html"), "utf8");
    expect(html).not.toContain("ox-code-play.js");
    expect(html).not.toContain("data-ox-code-play");
    expect(existsSync(path.join(outDir, "ox-code-play.js"))).toBe(false);
  });

  it("omits TypeScript typecheck from written SSG payloads without an endpoint", async () => {
    const root = mkdtempSync(path.join(tmpdir(), "ox-code-play-ssg-ts-"));
    const srcDir = path.join(root, "content");
    const outDir = path.join(root, "dist");
    mkdirSync(srcDir, { recursive: true });
    mkdirSync(outDir, { recursive: true });
    writeFileSync(path.join(srcDir, "index.md"), "```ts play\nconst n: number = 1;\n```\n");
    writeFileSync(
      path.join(outDir, "index.html"),
      `<pre><code class="language-ts">const n: number = 1;</code></pre>\n`,
    );

    const plugin = codePlay({
      languages: { typescript: { execute: true, typecheck: true } },
      srcDir: "content",
      outDir: "dist",
    });
    hookFn(plugin.configResolved)({
      command: "build",
      root,
      base: "/",
      build: { outDir: "dist" },
    } as never);
    await hookFn(plugin.closeBundle)();

    const html = readFileSync(path.join(outDir, "index.html"), "utf8");
    const encoded = /data-ox-code-play="([^"]+)"/.exec(html)?.[1];
    expect(encoded).toBeTruthy();
    const payload = decodePayload(encoded ?? "");
    expect(payload.capabilities.typecheck).toBe(false);
    expect(payload.endpoints?.typecheck).toBeUndefined();
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
