import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { renderThemeTokenCss, tokensToCss, type ThemeTokenSource } from "./theme-tokens";
import { resolveTheme, themeToNapi } from "./theme";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

describe("renderThemeTokenCss", () => {
  it("emits the three selectors the SSG runtime switches between", () => {
    const css = renderThemeTokenCss({
      tokens: { "syntax-foreground": "#545464" },
      darkTokens: { "syntax-foreground": "#dcd7ba" },
    });

    expect(css).toContain(":root {\n  --octc-syntax-foreground: #545464;\n}");
    expect(css).toContain('[data-theme="dark"] {\n  --octc-syntax-foreground: #dcd7ba;\n}');
    expect(css).toContain("@media (prefers-color-scheme: dark) {");
    // Explicit light has to beat the operating-system dark preference, which is
    // what the `:not()` in the media block encodes.
    expect(css).toContain(':root:not([data-theme="light"]) {');
    expect(css).not.toContain(":root:not([data-theme='dark'])");
  });

  it("omits the dark blocks when a theme has no dark tokens", () => {
    const css = renderThemeTokenCss({ tokens: { "syntax-foreground": "#545464" } });

    expect(css).toBe(":root {\n  --octc-syntax-foreground: #545464;\n}");
    expect(css).not.toContain("prefers-color-scheme");
  });

  it("returns nothing for a theme that declares no tokens", () => {
    expect(renderThemeTokenCss({})).toBe("");
    expect(renderThemeTokenCss([])).toBe("");
  });

  it("keeps only the tokens the include predicate accepts", () => {
    const css = renderThemeTokenCss(
      {
        tokens: { "syntax-token-keyword": "#b35b79", "surface-glass": "#e7dba0" },
        darkTokens: { "syntax-token-keyword": "#957fb8", "surface-glass": "#2a2a37" },
      },
      { include: (name) => name.startsWith("syntax-") },
    );

    expect(css).toContain("--octc-syntax-token-keyword: #b35b79;");
    expect(css).toContain("--octc-syntax-token-keyword: #957fb8;");
    expect(css).not.toContain("surface-glass");
  });

  it("drops an excluded token together with the override a later layer would apply", () => {
    const css = renderThemeTokenCss(
      [{ tokens: { "surface-glass": "#base" } }, { tokens: { "surface-glass": "#override" } }],
      { include: (name) => name !== "surface-glass" },
    );

    expect(css).toBe("");
  });

  it("composes layers left to right and flattens each extends chain base-first", () => {
    const base: ThemeTokenSource = {
      tokens: { "syntax-foreground": "#base", "syntax-background": "#base-bg" },
    };
    const skin: ThemeTokenSource = { extends: base, tokens: { "syntax-foreground": "#skin" } };
    const scheme: ThemeTokenSource = { tokens: { "syntax-background": "#scheme-bg" } };

    const css = renderThemeTokenCss([skin, scheme]);

    expect(css).toContain("--octc-syntax-foreground: #skin;");
    expect(css).toContain("--octc-syntax-background: #scheme-bg;");
    expect(css).not.toContain("#base-bg");
  });

  it("does not hang on a theme whose extends chain forms a cycle", () => {
    const a: ThemeTokenSource = { tokens: { "syntax-foreground": "#a" } };
    const b: ThemeTokenSource = { extends: a, tokens: { "syntax-background": "#b" } };
    a.extends = b;

    expect(renderThemeTokenCss(b)).toContain("--octc-syntax-foreground: #a;");
  });

  it("skips tokens with an empty value instead of emitting a malformed declaration", () => {
    const css = renderThemeTokenCss({
      tokens: { "syntax-foreground": "", "syntax-background": "#e7dba0" },
    });

    expect(css).toBe(":root {\n  --octc-syntax-background: #e7dba0;\n}");
  });

  it("rejects token names that would break out of the declaration block", () => {
    for (const name of ["", " ", "Syntax-Foreground", "syntax foreground", "syntax:bar", "a}b"]) {
      expect(() => renderThemeTokenCss({ tokens: { [name]: "#000" } }), name).toThrow(
        /Invalid theme token name/,
      );
    }
  });

  it("validates dark token names too", () => {
    expect(() => renderThemeTokenCss({ darkTokens: { "--octc-nested": "#000" } })).toThrow(
      /Invalid theme token name/,
    );
  });

  it("renders the same declarations the built-in SSG emits for the same theme", () => {
    const theme = {
      name: "shared",
      tokens: { "syntax-foreground": "#545464" },
      darkTokens: { "syntax-foreground": "#dcd7ba" },
    };

    const ssgCss = themeToNapi(resolveTheme(theme)).css ?? "";

    expect(ssgCss).toContain(renderThemeTokenCss(resolveTheme(theme)));
  });

  it("re-exports the renderer the theme layer builds on", () => {
    expect(tokensToCss({ "syntax-foreground": "#545464" }, {})).toBe(
      renderThemeTokenCss({ tokens: { "syntax-foreground": "#545464" } }),
    );
  });
});

describe("bare host consuming @ox-content/theme-color-kanagawa", () => {
  // Loaded through a runtime URL rather than a static import: the theme package
  // is a sibling workspace package, so importing its source by name would need
  // its `dist/`, and importing it by path would pull a file outside this
  // package's `rootDir` into the TypeScript program. The point of the fixture
  // is that a published `ThemeConfig` flows through the public API unchanged.
  async function loadKanagawa(): Promise<ThemeTokenSource> {
    const source = join(packageRoot, "../theme-color/kanagawa/src/index.ts");
    const module = (await import(pathToFileURL(source).href)) as {
      kanagawa: ThemeTokenSource;
    };
    return module.kanagawa;
  }

  it("renders the official syntax palette without page colors or layout", async () => {
    const kanagawa = await loadKanagawa();

    const css = renderThemeTokenCss(kanagawa, {
      include: (name) => name.startsWith("syntax-"),
    });

    expect(css).toContain("--octc-syntax-token-keyword: #b35b79;");
    expect(css).toContain("--octc-syntax-token-keyword: #957fb8;");
    expect(css).toContain("--octc-syntax-background: #e7dba0;");
    expect(css).toContain("--octc-syntax-background: #16161d;");

    // Nothing from the page palette, code-block chrome, or brand accents.
    expect(css).not.toMatch(/--octc-(?!syntax-)/);
    expect(css).not.toContain("surface-glass");
    expect(css).not.toContain("color-code-line");
  });

  it("covers explicit light, explicit dark, and the operating-system fallback", async () => {
    const kanagawa = await loadKanagawa();

    const css = renderThemeTokenCss(kanagawa, {
      include: (name) => name === "syntax-foreground",
    });

    expect(css).toBe(
      [
        ":root {",
        "  --octc-syntax-foreground: #545464;",
        "}",
        '[data-theme="dark"] {',
        "  --octc-syntax-foreground: #dcd7ba;",
        "}",
        "@media (prefers-color-scheme: dark) {",
        '  :root:not([data-theme="light"]) {',
        "    --octc-syntax-foreground: #dcd7ba;",
        "  }",
        "}",
      ].join("\n"),
    );
  });

  it("renders every token when no filter is passed", async () => {
    const kanagawa = await loadKanagawa();

    const css = renderThemeTokenCss(kanagawa);

    expect(css).toContain("--octc-surface-glass:");
    expect(css).toContain("--octc-syntax-token-keyword:");
  });
});
