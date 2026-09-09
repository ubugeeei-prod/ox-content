import { beforeEach, describe, expect, it, vi } from "vite-plus/test";

const { render } = vi.hoisted(() => ({
  render: vi.fn((html: string) => ({
    html: html.replace(
      /<pre><code class="language-mermaid">[\s\S]*?<\/code><\/pre>/g,
      '<div class="ox-mermaid"><svg></svg></div>',
    ),
    errors: [],
  })),
}));
vi.mock("../napi", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../napi")>();
  return {
    ...actual,
    importNapiModule: async () => ({
      ...(await actual.importNapiModule()),
      transformMermaid: render,
    }),
  };
});
import { transformMermaidStatic } from "./mermaid";
import { renderMarkdown } from "../render-markdown";

beforeEach(() => {
  render.mockClear();
});

describe("Mermaid fence dispatch", () => {
  it("renders ordinary fences when the public mermaid option is enabled", async () => {
    const source = "```mermaid\ngraph TD; A-->B;\n```";
    const enabled = await renderMarkdown(source, "/virtual/diagram.md", {
      mermaid: true,
      highlight: false,
      embeds: false,
    });
    expect(enabled.html).toContain("<svg>");
    expect(render).toHaveBeenCalledTimes(1);
    const disabled = await renderMarkdown(source, "/virtual/diagram.md", {
      mermaid: false,
      highlight: false,
      embeds: false,
    });
    expect(disabled.html).toContain('class="language-mermaid"');
    expect(render).toHaveBeenCalledTimes(1);
  });
  it("dispatches ordinary Markdown renderer output without a rendered-diagram marker", async () => {
    const html = '<pre><code class="language-mermaid">graph TD; A--&gt;B;</code></pre>\n';
    expect(await transformMermaidStatic(html)).toContain("<svg>");
    expect(render).toHaveBeenCalledTimes(1);
    expect(render.mock.calls[0]?.[0]).toBe(html);
  });

  it("leaves already-rendered diagrams and ordinary text on the no-op path", async () => {
    for (const html of [
      '<div class="ox-mermaid"><svg></svg></div>',
      "<p>language-mermaid</p>",
      '<pre><code class="language-ts">const n = 1;</code></pre>',
    ]) {
      expect(await transformMermaidStatic(html)).toBe(html);
    }
    expect(render).not.toHaveBeenCalled();
  });
});
