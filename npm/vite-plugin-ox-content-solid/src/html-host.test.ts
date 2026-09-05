import { describe, expect, it } from "vite-plus/test";
import { createSolidHtmlHostHydrate, renderSolidHtmlHost, type MdxImport } from ".";

describe("renderSolidHtmlHost", () => {
  it("renders HTML-string islands through document-local and registered modules", async () => {
    const loaded: string[] = [];
    const result = await renderSolidHtmlHost({
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
      imports: [defaultImport("Chart", "./Chart.tsx")],
      components: { Badge: "./src/components/Badge.tsx" },
      resolveClientModule: (module) => `/assets/${module.name}.js`,
      loadModule: async (moduleId) => {
        loaded.push(moduleId);
        return { default: moduleId.endsWith("Chart.tsx") ? "chart-component" : "badge-component" };
      },
      renderComponent: (component, props, slotHtml, context) => {
        const value = component as string;
        const title = typeof props.title === "string" ? props.title : "";
        return `<strong data-component="${context.component}">${value}:${title}:${slotHtml ?? ""}</strong>`;
      },
    });

    expect(loaded.sort()).toEqual(["/repo/docs/Chart.tsx", "/repo/src/components/Badge.tsx"]);
    expect(result.diagnostics).toEqual([]);
    expect(result.modules).toEqual([
      {
        name: "Chart",
        serverModuleId: "/repo/docs/Chart.tsx",
        exportName: "default",
        source: "document",
        clientModuleId: "/assets/Chart.js",
      },
      {
        name: "Badge",
        serverModuleId: "/repo/src/components/Badge.tsx",
        exportName: "default",
        source: "components",
        clientModuleId: "/assets/Badge.js",
      },
    ]);
    expect(result.clientModules).toEqual([
      { name: "Chart", moduleId: "/assets/Chart.js" },
      { name: "Badge", moduleId: "/assets/Badge.js" },
    ]);
    expect(result.html).toContain('data-ox-ssr="true"');
    expect(result.html).toContain("data-ox-content='&lt;p&gt;slot&lt;/p&gt;'");
    expect(result.html).toContain(
      '<strong data-component="Chart">chart-component:Revenue:<p>slot</p></strong>',
    );
    expect(result.html).toContain('<strong data-component="Badge">badge-component::</strong>');
  });

  it("keeps document-local module identities scoped per document", async () => {
    const first = await renderSolidHtmlHost(hostInput("/repo/docs/a.mdx", "./a/Chart.tsx"));
    const second = await renderSolidHtmlHost(hostInput("/repo/docs/b.mdx", "./b/Chart.tsx"));

    expect(first.modules[0]?.serverModuleId).toBe("/repo/docs/a/Chart.tsx");
    expect(second.modules[0]?.serverModuleId).toBe("/repo/docs/b/Chart.tsx");
    expect(first.modules[0]?.serverModuleId).not.toBe(second.modules[0]?.serverModuleId);
  });

  it("supports named document-local imports", async () => {
    const result = await renderSolidHtmlHost({
      html: '<div data-ox-island="Plot"></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      imports: [
        {
          source: "./Chart.tsx",
          specifiers: [{ imported: "Chart", local: "Plot", kind: "named" }],
        },
      ],
      loadModule: async () => ({ Chart: "named-chart" }),
      renderComponent: (component) => `<strong>${component as string}</strong>`,
    });

    expect(result.diagnostics).toEqual([]);
    expect(result.modules[0]).toMatchObject({ name: "Plot", exportName: "Chart" });
    expect(result.html).toContain("<strong>named-chart</strong>");
  });

  it("passes slot markup as raw HTML through the default Solid SSR renderer", async () => {
    const result = await renderSolidHtmlHost({
      html: '<div data-ox-island="Echo"><em>slot</em></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      components: { Echo: "./src/Echo.tsx" },
      loadModule: async () => ({ default: (props: { children?: unknown }) => props.children }),
    });

    expect(result.diagnostics).toEqual([]);
    expect(result.html).toContain("data-ox-content='&lt;em&gt;slot&lt;/em&gt;'><em>slot</em>");
    expect(result.html).not.toContain(">&lt;em&gt;slot&lt;/em&gt;</div>");
  });

  it("reports load and export failures with document context", async () => {
    const result = await renderSolidHtmlHost({
      html: '<div data-ox-island="MissingExport"></div><div data-ox-island="LoadError"></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      components: {
        MissingExport: "./src/MissingExport.tsx",
        LoadError: "./src/LoadError.tsx",
      },
      loadModule: async (moduleId) => {
        if (moduleId.endsWith("LoadError.tsx")) {
          throw new Error("cannot load");
        }
        return {};
      },
    });

    const byComponent = new Map(result.diagnostics.map((item) => [item.component, item]));
    expect(byComponent.get("MissingExport")).toMatchObject({
      code: "missing-export",
      documentPath: "/repo/docs/report.mdx",
    });
    expect(byComponent.get("LoadError")).toMatchObject({
      code: "module-load-failed",
      documentPath: "/repo/docs/report.mdx",
    });
  });

  it("reports missing modules and SSR failures without replacing the island shell", async () => {
    const result = await renderSolidHtmlHost({
      html: '<div data-ox-island="Missing"><em>fallback</em></div><div data-ox-island="Broken"></div>',
      documentPath: "/repo/docs/report.mdx",
      root: "/repo",
      srcDir: "docs",
      components: { Broken: "./src/Broken.tsx" },
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

  it("keeps module loading scoped to each render call for development edits", async () => {
    let version = "v1";
    let loads = 0;
    const input = () => ({
      ...hostInput("/repo/docs/live.mdx", "./Chart.tsx"),
      loadModule: async () => {
        loads += 1;
        return { default: version };
      },
      renderComponent: (component: unknown) => `<strong>${component as string}</strong>`,
    });

    const first = await renderSolidHtmlHost(input());
    version = "v2";
    const second = await renderSolidHtmlHost(input());

    expect(loads).toBe(2);
    expect(first.html).toContain("<strong>v1</strong>");
    expect(second.html).toContain("<strong>v2</strong>");
  });
});

describe("createSolidHtmlHostHydrate", () => {
  it("mounts with caller-owned Solid renderer and preserves slot HTML", () => {
    const element = {
      dataset: { oxIsland: "Badge", oxContent: "<span>SSR</span>" },
      innerHTML: "<span>SSR</span>",
    } as unknown as HTMLElement;
    const calls: unknown[] = [];
    const hydrate = createSolidHtmlHostHydrate({
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

function hostInput(documentPath: string, specifier: string) {
  return {
    html: '<div data-ox-island="Chart"></div>',
    documentPath,
    root: "/repo",
    srcDir: "docs",
    imports: [defaultImport("Chart", specifier)],
    loadModule: async () => ({ default: "chart" }),
    renderComponent: () => "<strong>chart</strong>",
  };
}

function defaultImport(local: string, source: string): MdxImport {
  return {
    source,
    specifiers: [{ imported: "default", local, kind: "default" }],
  };
}
