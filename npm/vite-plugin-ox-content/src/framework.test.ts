import { describe, expect, it } from "vite-plus/test";
import {
  createFrameworkMarkdownOptions,
  escapeSvelteMarkup,
  renderHtmlToFrameworkCode,
  renderHtmlToReactCreateElement,
  renderHtmlToReactComponent,
  renderHtmlToSvelteComponent,
  renderHtmlToVueComponent,
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

    expect(code).toMatchSnapshot();
  });

  it("renders Vue VDOM code with Vue-compatible attributes", () => {
    const code = renderHtmlToVueH(
      '<label class="field" for="name"><span>Name</span><input disabled type="text"></label>',
    );

    expect(code).toMatchSnapshot();
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

    expect({ reactCode, vueCode }).toMatchSnapshot();
  });

  it("renders framework component and render-function modules", () => {
    const html = "<p>Hello</p>";

    expect({
      reactComponent: renderHtmlToReactComponent(html),
      vueComponent: renderHtmlToVueComponent(html),
      vueRenderFunction: renderHtmlToFrameworkCode(html, "vue", "renderFunction"),
    }).toMatchSnapshot();
  });

  it("renders Svelte component code without innerHTML", () => {
    expect(renderHtmlToSvelteComponent("<p>{count}</p>")).toBe(
      '<div class="ox-content">\n<p>&#123;count&#125;</p>\n</div>\n',
    );
  });

  it("escapes raw HTML literals without breaking component scripts", () => {
    const html = '</script><p title="line\nnext">{ ok }</p>';
    const react = renderHtmlToFrameworkCode(html, "react", "innerHtml");
    const vue = renderHtmlToFrameworkCode(html, "vue", "innerHtml");
    const svelte = renderHtmlToFrameworkCode(html, "svelte", "innerHtml");

    expect(react).not.toContain("</script>");
    expect(vue).not.toContain("</script>");
    expect(svelte.match(/<\/script>/g)).toHaveLength(1);
    expect(react).toContain(String.raw`\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>`);
    expect(vue).toContain(String.raw`\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>`);
    expect(svelte).toContain(String.raw`\x3C/script>\x3Cp title=\"line\nnext\">{ ok }\x3C/p>`);
  });

  it("escapes Svelte expression delimiters before emitting static markup", () => {
    expect(escapeSvelteMarkup("<p>{count} and }</p>")).toBe("<p>&#123;count&#125; and &#125;</p>");
  });
});
