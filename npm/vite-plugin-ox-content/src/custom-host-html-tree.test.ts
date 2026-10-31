import * as path from "node:path";
import type { Element, Root } from "hast";
import { describe, expect, it } from "vite-plus/test";
import { createAssetsContext } from "./custom-host-assets";
import { createBaseContext } from "./custom-host-loader";
import { parseHtmlFragment } from "./html-fragment";
import { resolveOptions } from "./resolve-options";

function renderer() {
  const root = process.cwd();
  const outDir = path.join(root, "unused-test-output");
  const options = resolveOptions({
    docs: false,
    search: false,
    embeds: false,
    highlight: false,
    ssg: { generateOgImage: false, lastUpdated: false, readerChrome: false },
  });
  return createBaseContext(
    "build",
    root,
    outDir,
    options,
    async () => ({}),
    createAssetsContext(options, outDir, undefined, undefined),
  ).markdown;
}

function elements(tree: Root | Element) {
  return tree.children.filter((node): node is Element => node.type === "element");
}

describe("custom host generated HTML trees", () => {
  it("repairs HTML fragments and preserves entities, SVG, and template contents", () => {
    const tree = parseHtmlFragment(
      '<table><tr><td>&amp;雪🌸</table><svg viewBox="0 0 1 1"><path d="M0 0"/></svg><template><em>slot</em></template>',
    );
    const [table, svg, template] = elements(tree);
    expect(table!.tagName).toBe("table");
    const tbody = elements(table!)[0]!;
    expect(tbody.tagName).toBe("tbody");
    expect(elements(elements(tbody)[0]!)[0]!.children).toEqual([{ type: "text", value: "&雪🌸" }]);
    expect(svg!.properties.viewBox).toBe("0 0 1 1");
    expect(template!.content?.children[0]).toMatchObject({ type: "element", tagName: "em" });
    expect(tree.position).toBeUndefined();
    expect(svg!.position).toBeUndefined();
  });

  it("reuses one tree inside a callback and keeps edits separate from the HTML string", async () => {
    const markdown = renderer();
    const result = await markdown.render({
      source: "# 雪🌸",
      documentPath: "content/tree.md",
      renderHtml(context) {
        const tree = context.parseHtml();
        expect(context.parseHtml()).toBe(tree);
        expect(elements(tree)[0]!.tagName).toBe("h1");
        tree.children = [];
        return context.html;
      },
    });
    expect(result.html).toContain("雪🌸");
  });

  it("retains the callback input when parsing is deferred until after the callback", async () => {
    const markdown = renderer();
    const result = await markdown.render<() => Root>({
      source: "# Original",
      documentPath: "content/tree.md",
      renderHtml(context) {
        return { html: `<main>${context.html}</main>`, metadata: context.parseHtml };
      },
    });
    expect(result.html).toContain("<main>");
    expect(elements(result.metadata!())[0]!.tagName).toBe("h1");
  });

  it("isolates mutable trees between renders of the same document", async () => {
    const markdown = renderer();
    const render = () =>
      markdown.render<Root>({
        source: "# Original",
        documentPath: "content/tree.md",
        renderHtml(context) {
          return { html: context.html, metadata: context.parseHtml() };
        },
      });
    const first = await render();
    first.metadata!.children = [];
    const second = await render();
    expect(second.metadata).not.toBe(first.metadata);
    expect(elements(second.metadata!)[0]!.tagName).toBe("h1");
  });
});
