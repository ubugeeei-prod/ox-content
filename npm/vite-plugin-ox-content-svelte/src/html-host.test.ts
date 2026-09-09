import { compile } from "svelte/compiler";
import { describe, expect, it } from "vite-plus/test";
import {
  SvelteHtmlHostRenderError,
  createSvelteHtmlHostHydrate,
  createSvelteHtmlHostRenderer,
  renderSvelteHtmlHost,
  type MdxImport,
} from ".";
import { withGeneratedModule } from "./test/fixtures/transform-harness";

describe("renderSvelteHtmlHost", () => {
  it("renders HTML-string islands through document-local and registered modules", async () => {
    const loaded: string[] = [];
    const result = await renderSvelteHtmlHost({
      html: [
        '<div data-ox-island="Chart">',
        '<script type="application/json">{"props":{"title":"Revenue"},"expressions":{},"spreads":[]}</script>',
        "<p>slot</p>",
        "</div>",
        '<span data-ox-island="Badge"></span>',
      ].join(""),
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      imports: [defaultImport("Chart", "./Chart.svelte")],
      components: { Badge: "./src/components/Badge.svelte" },
      resolveClientModule: (module) => `/assets/${module.name}.js`,
      loadModule: async (moduleId) => {
        loaded.push(moduleId);
        return {
          default: moduleId.endsWith("Chart.svelte") ? "chart-component" : "badge-component",
        };
      },
      renderComponent: (component, props, slotHtml, context) => {
        const title = typeof props.title === "string" ? props.title : "";
        return `<strong data-component="${context.component}">${String(component)}:${title}:${
          slotHtml ?? ""
        }</strong>`;
      },
    });

    expect(loaded.sort()).toEqual(["/repo/docs/Chart.svelte", "/repo/src/components/Badge.svelte"]);
    expect(result.diagnostics).toEqual([]);
    expect(result.modules).toEqual([
      {
        name: "Chart",
        serverModuleId: "/repo/docs/Chart.svelte",
        exportName: "default",
        source: "document",
        clientModuleId: "/assets/Chart.js",
      },
      {
        name: "Badge",
        serverModuleId: "/repo/src/components/Badge.svelte",
        exportName: "default",
        source: "components",
        clientModuleId: "/assets/Badge.js",
      },
    ]);
    expect(result.clientModules).toEqual([
      { name: "Chart", moduleId: "/assets/Chart.js", exportName: "default" },
      { name: "Badge", moduleId: "/assets/Badge.js", exportName: "default" },
    ]);
    expect(result.html).toContain('data-ox-ssr="true"');
    expect(result.html).toContain('data-ox-module="/assets/Chart.js"');
    expect(result.html).toContain('data-ox-export="default"');
    expect(result.html).toContain("data-ox-content='&lt;p&gt;slot&lt;/p&gt;'");
    expect(result.html).toContain(
      '<strong data-component="Chart">chart-component:Revenue:<p>slot</p></strong>',
    );
    expect(result.html).toContain('<strong data-component="Badge">badge-component::</strong>');
  });

  it("supports named document-local imports", async () => {
    const result = await renderSvelteHtmlHost({
      html: '<div data-ox-island="Plot"></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      imports: [namedImport("Plot", "Chart", "./Chart.svelte")],
      resolveClientModule: () => "./Chart.svelte",
      loadModule: async () => ({ Chart: "named-chart" }),
      renderComponent: (component) => `<strong>${String(component)}</strong>`,
    });

    expect(result.diagnostics).toEqual([]);
    expect(result.modules[0]).toMatchObject({ name: "Plot", exportName: "Chart" });
    expect(result.clientModules[0]).toEqual({
      name: "Plot",
      moduleId: "./Chart.svelte",
      exportName: "Chart",
    });
    expect(result.html).toContain('data-ox-module="./Chart.svelte"');
    expect(result.html).toContain('data-ox-export="Chart"');
    expect(result.html).toContain("<strong>named-chart</strong>");
  });

  it("passes slot markup as a raw snippet through the default Svelte SSR renderer", async () => {
    const compiled = compile(
      "<script>let { children } = $props();</script><section>{@render children?.()}</section>",
      {
        filename: "/repo/src/Echo.svelte",
        generate: "server",
        runes: true,
      },
    );

    await withGeneratedModule(compiled.js.code, async (Echo) => {
      const result = await renderSvelteHtmlHost({
        html: '<div data-ox-island="Echo"><em>slot</em></div>',
        documentPath: "/repo/docs/report.mdx",
        root: "/repo",
        srcDir: "docs",
        components: { Echo: "./src/Echo.svelte" },
        loadModule: async () => ({ default: Echo }),
      });

      expect(result.diagnostics).toEqual([]);
      expect(result.html).toContain("data-ox-content='&lt;em&gt;slot&lt;/em&gt;'");
      expect(result.html).toContain("<section>");
      expect(result.html).toContain("<em>slot</em>");
      expect(result.html).not.toContain(">&lt;em&gt;slot&lt;/em&gt;</div>");
    });
  });

  it("reports missing modules and SSR failures without replacing the island shell", async () => {
    const result = await renderSvelteHtmlHost({
      html: '<div data-ox-island="Missing"><em>fallback</em></div><div data-ox-island="Broken"></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      components: { Broken: "./src/Broken.svelte" },
      loadModule: async () => ({ default: "broken" }),
      renderComponent: () => {
        throw new Error("boom");
      },
    });

    expect(result.diagnostics.map((item) => item.code)).toEqual([
      "missing-component",
      "ssr-failed",
    ]);
    expect(result.html).toContain("<em>fallback</em>");
    expect(result.html).toContain('data-ox-island="Broken"');
  });
});

describe("createSvelteHtmlHostRenderer", () => {
  it("creates the standard custom-host renderer with canonical client module ids", async () => {
    const renderIslands = createSvelteHtmlHostRenderer({
      root: "/repo",
      srcDir: "docs",
      loadModule: async () => ({ default: "chart" }),
      renderComponent: (component, props, slotHtml, context) =>
        `<strong data-component="${context.component}">${String(component)}:${
          props.title as string
        }:${slotHtml ?? ""}</strong>`,
    });

    const result = await renderIslands(
      [
        '<div data-ox-island="Chart">',
        '<script type="application/json">{"props":{"title":"Revenue"},"expressions":{},"spreads":[]}</script>',
        "<p>slot</p>",
        "</div>",
      ].join(""),
      {
        documentPath: "/repo/docs/report.mdx",
        imports: [defaultImport("Chart", "./Chart.svelte")],
      },
    );

    expect(result.diagnostics).toEqual([]);
    expect(result.clientModules).toEqual([
      { name: "Chart", moduleId: "/docs/Chart.svelte", exportName: "default" },
    ]);
    expect(result.html).toContain('data-ox-module="/docs/Chart.svelte"');
    expect(result.html).toContain(
      '<strong data-component="Chart">chart:Revenue:<p>slot</p></strong>',
    );
  });

  it("throws diagnostics by default and can collect them for custom policies", async () => {
    const input = {
      root: "/repo",
      srcDir: "docs",
      loadModule: async () => ({}),
    };
    const context = {
      documentPath: "/repo/docs/report.mdx",
      components: { Missing: "./src/Missing.svelte" },
    };

    await expect(
      createSvelteHtmlHostRenderer(input)('<div data-ox-island="Missing"></div>', context),
    ).rejects.toMatchObject({
      name: "SvelteHtmlHostRenderError",
      diagnostics: [expect.objectContaining({ code: "missing-export", component: "Missing" })],
    });

    const collected = await createSvelteHtmlHostRenderer({
      ...input,
      diagnostics: "collect",
    })('<div data-ox-island="Missing"></div>', context);

    expect(collected.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-export", component: "Missing" }),
    ]);
    expect(new SvelteHtmlHostRenderError(collected.diagnostics).message).toContain(
      "missing-export",
    );
  });
});

describe("createSvelteHtmlHostHydrate", () => {
  it("mounts with caller-owned Svelte renderer and preserves slot HTML", () => {
    const element = {
      dataset: { oxIsland: "Badge", oxContent: "<span>SSR</span>" },
      innerHTML: "<span>SSR</span>",
    } as unknown as HTMLElement;
    const calls: unknown[] = [];
    const hydrate = createSvelteHtmlHostHydrate({
      components: { Badge: "badge-component" },
      render: (component, props, target, slotHtml) => {
        calls.push({ component, props, target, slotHtml });
        return () => calls.push("disposed");
      },
    });

    const dispose = hydrate(element, { label: "ok" });

    expect(calls).toEqual([
      {
        component: "badge-component",
        props: { label: "ok" },
        target: element,
        slotHtml: "<span>SSR</span>",
      },
    ]);
    expect(element.innerHTML).toBe("");
    dispose?.();
    expect(calls.at(-1)).toBe("disposed");
  });
});

function defaultImport(local: string, source: string): MdxImport {
  return {
    source,
    specifiers: [{ imported: "default", local, kind: "default" }],
  };
}

function namedImport(local: string, imported: string, source: string): MdxImport {
  return { source, specifiers: [{ imported, local, kind: "named" }] };
}
