import { describe, expect, it } from "vite-plus/test";
import {
  createFrameworkMarkdownOptions,
  escapeSvelteMarkup,
  renderHtmlToReactCreateElement,
  renderHtmlToVueH,
} from "./framework";

describe("framework Markdown utilities", () => {
  it("creates framework-safe ox-content transform options", () => {
    const options = createFrameworkMarkdownOptions({
      srcDir: "docs",
      outDir: "dist",
      base: "/docs/",
      extensions: [".md"],
      gfm: true,
      toc: true,
      tocMaxDepth: 2,
      codeAnnotations: {
        enabled: true,
        metaKey: "mark",
      },
      embeds: {
        github: false,
        openGraph: { timeout: 500 },
      },
    });

    expect(options.ssg.enabled).toBe(false);
    expect(options.docs).toBe(false);
    expect(options.search.enabled).toBe(false);
    expect(options.i18n).toBe(false);
    expect(options.highlight).toBe(false);
    expect(options.frontmatter).toBe(false);
    expect(options.codeAnnotations).toMatchObject({
      enabled: true,
      notation: "attribute",
      metaKey: "mark",
    });
    expect(options.embeds.github).toBe(false);
    expect(options.embeds.openGraph).toEqual({ timeout: 500 });
    expect(options.embeds.pm).toBe(false);
  });

  it("renders React VDOM code with React-compatible attributes", () => {
    const code = renderHtmlToReactCreateElement(
      [
        '<section class="lead" for="name" data-id="42" aria-label="Intro">',
        '<p style="font-weight: bold; --brand: red;">Hello <strong>world</strong></p>',
        "</section>",
      ].join(""),
    );

    expect(code).toContain("createElement('div', { className: 'ox-content' }");
    expect(code).toContain('"className": "lead"');
    expect(code).toContain('"htmlFor": "name"');
    expect(code).toContain('"data-id": "42"');
    expect(code).toContain('"aria-label": "Intro"');
    expect(code).toContain('"style": { "fontWeight": "bold", "--brand": "red" }');
    expect(code).toContain('createElement("strong", null, "world")');
  });

  it("renders Vue VDOM code with Vue-compatible attributes", () => {
    const code = renderHtmlToVueH(
      '<label class="field" for="name"><span>Name</span><input disabled type="text"></label>',
    );

    expect(code).toContain("h('div', { class: 'ox-content' }");
    expect(code).toContain('h("label", { "class": "field", "for": "name" }');
    expect(code).toContain('h("span", null, "Name")');
    expect(code).toContain('h("input", { "disabled": true, "type": "text" })');
  });

  it("renders framework component islands for native React and Vue targets", () => {
    const html = '<p>Before</p><div data-ox-island="Alert" data-ox-id="ox-island-0"></div>';
    const islands = [
      {
        id: "ox-island-0",
        name: "Alert",
        props: { tone: "info", active: true },
        content: "Read docs",
      },
    ];

    const reactCode = renderHtmlToReactCreateElement(html, islands);
    const vueCode = renderHtmlToVueH(html, islands);

    expect(reactCode).toContain('createElement(Alert, { "active": true, "tone": "info" }');
    expect(reactCode).toContain('"Read docs")');
    expect(vueCode).toContain('h(Alert, { "active": true, "tone": "info" }');
    expect(vueCode).toContain('"Read docs")');
  });

  it("escapes Svelte expression delimiters before emitting static markup", () => {
    expect(escapeSvelteMarkup("<p>{count} and }</p>")).toBe("<p>&#123;count&#125; and &#125;</p>");
  });
});
