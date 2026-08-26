import { describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "./index";
import { createMarkdownProcessor, renderMarkdown as renderMarkdownDirect } from "./render-markdown";
import { resolveOptions } from "./resolve-options";
import { transformMarkdown } from "./transform";

const publicOptions = { ssg: false as const, highlight: false };

describe("renderMarkdown", () => {
  it("returns structured TransformResult from public OxContentOptions", async () => {
    const result = await renderMarkdown("# Hi", "/virtual/article.md", publicOptions);

    expect(result.html).toContain('<h1 id="hi">Hi</h1>');
    expect(result.frontmatter).toEqual({});
    expect(result.toc).toEqual([{ depth: 1, text: "Hi", slug: "hi", children: [] }]);
    expect(result.imports).toEqual([]);
    expect(result.exports).toEqual([]);
    expect(result.components).toEqual([]);
    expect(result.html).not.toMatch(/export const html = /);
  });

  it("parses frontmatter without reading generated module source", async () => {
    const result = await renderMarkdown(
      "---\ntitle: Guide\n---\n# Intro\n",
      "/virtual/article.md",
      publicOptions,
    );

    expect(result.frontmatter).toEqual({ title: "Guide" });
    expect(result.html).toContain('<h1 id="intro">Intro</h1>');
    expect(result.toc[0]?.text).toBe("Intro");
  });

  it("infers MDX from the file path the same way as the Vite plugin", async () => {
    const source = "import Card from './Card'\n\n<Card>Visible copy</Card>\n";
    const inferred = await renderMarkdown(source, "docs/Guide.MDX?raw", publicOptions);
    const markdown = await renderMarkdown(source, "/virtual/article.md", publicOptions);
    const optedOut = await renderMarkdown(source, "docs/guide.mdx", {
      ...publicOptions,
      mdx: false,
    });
    const optedIn = await renderMarkdown(source, "docs/guide.md", { ...publicOptions, mdx: true });

    expect(inferred.html).toContain('data-ox-island="Card"');
    expect(inferred.html).not.toContain("import Card");
    expect(markdown.html).toContain("import Card");
    expect(markdown.html).not.toContain('data-ox-island="Card"');
    expect(optedOut.html).toContain("import Card");
    expect(optedIn.html).toContain('data-ox-island="Card"');
  });

  it("matches oxContent / resolveOptions built-in defaults", async () => {
    const source = "# Hi\n\nhttps://example.com\n";
    const resolved = resolveOptions({});
    const viaPublicApi = await renderMarkdownDirect(source, "/virtual/article.md");
    const viaResolved = await transformMarkdown(source, "/virtual/article.md", resolved);

    expect(resolved.gfm).toBe(true);
    expect(resolved.highlight).toBe(false);
    expect(resolved.frontmatter).toBe(true);
    expect(resolved.toc).toBe(true);
    expect(resolved.tocMaxDepth).toBe(3);
    expect(viaPublicApi.html).toBe(viaResolved.html);
    expect(viaPublicApi.toc).toEqual(viaResolved.toc);
    expect(viaPublicApi.frontmatter).toEqual(viaResolved.frontmatter);
    expect(viaPublicApi.html).toContain('href="https://example.com"');
  });

  it("reuses resolved options across documents with createMarkdownProcessor", async () => {
    const processor = createMarkdownProcessor(publicOptions);
    const first = await processor.render("# Alpha", "/virtual/a.md");
    const second = await processor.render("# Beta", "/virtual/b.mdx");

    expect(first.html).toContain('<h1 id="alpha">Alpha</h1>');
    expect(second.html).toContain('<h1 id="beta">Beta</h1>');
  });
});
