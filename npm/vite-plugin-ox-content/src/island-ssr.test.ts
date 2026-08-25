import { describe, expect, it } from "vite-plus/test";
import { applyIslandSsrHtml } from "./island-ssr";

describe("applyIslandSsrHtml", () => {
  it("injects adapter HTML without importing a framework SSR package", async () => {
    const html = [
      '<div class="ox-island" data-ox-island="GtvChart" data-ox-props="{&quot;props&quot;:{&quot;title&quot;:&quot;ok&quot;}}">',
      '<script type="application/json">{"props":{"title":"ok"}}</script>',
      "</div>",
    ].join("");

    const rendered = await applyIslandSsrHtml(
      html,
      (name, props, filePath) =>
        `<span class="ssr">${name}:${String(props.title)}:${filePath}</span>`,
      "/repo/docs/guide.mdx",
    );

    expect(rendered).toContain('<span class="ssr">GtvChart:ok:/repo/docs/guide.mdx</span>');
    expect(rendered).toContain('data-ox-island="GtvChart"');
    expect(rendered).toContain('<script type="application/json">');
    expect(rendered).not.toContain("svelte/server");
    expect(rendered).not.toContain("react-dom/server");
    expect(rendered).not.toContain("solid-js/web");
  });
});
