import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { createAssetsContext } from "./custom-host-assets";
import { createBaseContext } from "./custom-host-loader";
import { customHostOxContentOptions } from "./custom-host-options";
import { createOxContentMarkdownRenderer } from "./markdown-public";
import { resolveOptions } from "./resolve-options";

const oxContent = {
  base: "/site/",
  srcDir: "content",
  outDir: "dist",
  docs: false,
  search: false,
  embeds: false,
  highlight: false,
  ssg: { routePrefix: "guides", generateOgImage: false, lastUpdated: false, readerChrome: false },
} as const;

describe("standalone Custom Host Markdown", () => {
  it("preserves the build host pipeline, paths, metadata, and deferred module output", async () => {
    const root = path.resolve("standalone-project");
    const options = resolveOptions(customHostOxContentOptions(oxContent));
    const outDir = path.resolve(root, options.outDir);
    const assets = createAssetsContext(options, outDir, undefined, undefined, undefined, root);
    const hosted = createBaseContext("build", root, outDir, options, async () => ({}), assets);
    const standalone = createOxContentMarkdownRenderer({ root, oxContent });
    const input = {
      source:
        '---\ntitle: 雪🌸\n---\n\n# Heading\n\n[Sibling](./other.md)\n\n<youtube id="dQw4w9WgXcQ"></youtube>',
      documentPath: "content/guide.md",
      convertMdLinks: false,
    };
    const expected = await hosted.markdown.render(input);
    const actual = await standalone.render(input);
    expect(actual.html).toBe(expected.html);
    expect(actual.markdownHtml).toBe(expected.markdownHtml);
    expect(actual.frontmatter).toEqual(expected.frontmatter);
    expect(actual.dependencies).toEqual([path.join(root, input.documentPath)]);
    expect(actual.transform.code).toBe(expected.transform.code);
    expect(actual.html).toContain("youtube");
  });

  it("uses caller loaders and assets while retaining the HTML tree callback contract", async () => {
    const root = path.resolve("standalone-project");
    const options = resolveOptions(customHostOxContentOptions(oxContent));
    const assets = createAssetsContext(options, path.resolve(root, "dist"), undefined, undefined);
    const loaded: string[] = [];
    const markdown = createOxContentMarkdownRenderer({
      root,
      oxContent,
      mode: "serve",
      assets,
      async loadModule(moduleId) {
        loaded.push(moduleId);
        return { suffix: "done" };
      },
    });
    const result = await markdown.render<{ suffix: string }>({
      source: "# Heading",
      documentPath: "content/guide.md",
      async renderHtml(context) {
        expect(context.root).toBe(root);
        expect(context.srcDir).toBe(path.join(root, "content"));
        expect(context.mode).toBe("serve");
        expect(context.assets).toBe(assets);
        expect(context.parseHtml()).toBe(context.parseHtml());
        expect(context.parseHtml().position).toBeUndefined();
        const metadata = (await context.loadModule("./renderer.mjs")) as { suffix: string };
        return { html: context.html + metadata.suffix, metadata };
      },
    });
    expect(loaded).toEqual(["./renderer.mjs"]);
    expect(result.metadata).toEqual({ suffix: "done" });
    expect(result.html).toContain("done");
  });

  it("requires an explicit loader only when a callback requests a module", async () => {
    const markdown = createOxContentMarkdownRenderer({ oxContent });
    await expect(
      markdown.render({
        source: "# Heading",
        documentPath: "content/guide.md",
        renderHtml: (context) => context.loadModule("./renderer.mjs").then(() => context.html),
      }),
    ).rejects.toThrow("Pass loadModule to createOxContentMarkdownRenderer()");
  });
});
