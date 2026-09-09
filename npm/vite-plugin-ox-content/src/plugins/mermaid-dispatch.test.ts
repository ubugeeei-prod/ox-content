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
vi.mock("../napi", () => ({ importNapiModule: async () => ({ transformMermaid: render }) }));
import { transformMermaidStatic } from "./mermaid";

beforeEach(() => {
  render.mockClear();
});

describe("Mermaid fence dispatch", () => {
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
