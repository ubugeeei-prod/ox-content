import { describe, expect, it } from "vitest";
import { resolveOptions } from "../../src/options";

describe("resolveOptions", () => {
  it("applies stable defaults", () => {
    const options = resolveOptions({});

    expect(options.srcDir).toBe("slides");
    expect(options.routeBase).toBe("slides");
    expect(options.routePrefix).toBe("/slides");
    expect(options.baseHref).toBe("/");
    expect(options.animations).toBe(true);
    expect(options.editor.enabled).toBe(true);
    expect(options.editor.routePrefix).toBe("/slides/editor");
    expect(options.editor.apiPrefix).toBe("/slides/editor/api");
    expect(options.presenter).toBe(true);
    expect(options.pdf.enabled).toBe(false);
  });

  it("normalizes custom base, route, and pdf settings", () => {
    const options = resolveOptions({
      base: "/docs",
      routeBase: "/decks/",
      animations: false,
      editor: {
        route: "/deckhouse/",
      },
      theme: {
        surfaceRadius: "6px",
        darkCanvasBackground: "#000814",
      },
      pdf: {
        fileName: "talk.pdf",
        pageWidth: "10in",
        pageHeight: "5.625in",
        scale: 3,
      },
    });

    expect(options.baseHref).toBe("/docs/");
    expect(options.routeBase).toBe("decks");
    expect(options.routePrefix).toBe("/decks");
    expect(options.animations).toBe(false);
    expect(options.editor.routePrefix).toBe("/deckhouse");
    expect(options.theme.surfaceRadius).toBe("6px");
    expect(options.theme.darkCanvasBackground).toBe("#000814");
    expect(options.napiTheme.surfaceRadius).toBe("6px");
    expect(options.napiTheme.darkCanvasBackground).toBe("#000814");
    expect(options.napiTheme.builtinAnimations).toBe(false);
    expect(options.pdf.enabled).toBe(true);
    expect(options.pdf.fileName).toBe("talk.pdf");
    expect(options.pdf.scale).toBe(2);
  });

  it("can disable the browser editor", () => {
    const options = resolveOptions({ editor: false });

    expect(options.editor.enabled).toBe(false);
    expect(options.editor.routePrefix).toBe("");
  });
});
