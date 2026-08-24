import { describe, it, expect } from "vite-plus/test";
import {
  defineTheme,
  defaultTheme,
  mergeThemes,
  resolveTheme,
  themeToNapi,
  type ThemeConfig,
} from "./theme";

describe("theme", () => {
  describe("defineTheme", () => {
    it("should return the config as-is", () => {
      const config: ThemeConfig = {
        name: "custom",
        colors: { primary: "#3498db" },
      };
      expect(defineTheme(config)).toEqual(config);
    });
  });

  describe("defaultTheme", () => {
    it("should have all required properties", () => {
      expect(defaultTheme.name).toBe("default");
      expect(defaultTheme.colors).toBeDefined();
      expect(defaultTheme.colors?.primary).toBe("#4f6fae");
      expect(defaultTheme.darkColors).toBeDefined();
      expect(defaultTheme.fonts).toBeDefined();
      expect(defaultTheme.entryPage?.mode).toBe("default");
      expect(defaultTheme.header?.showSiteNameText).toBe(true);
      expect(defaultTheme.layout).toBeDefined();
      expect(defaultTheme.aside).toBe(false);
    });
  });

  describe("mergeThemes", () => {
    it("should merge multiple themes", () => {
      const theme1: ThemeConfig = {
        colors: { primary: "#ff0000", background: "#ffffff" },
      };
      const theme2: ThemeConfig = {
        colors: { primary: "#00ff00" },
      };

      const merged = mergeThemes(theme1, theme2);
      expect(merged.colors?.primary).toBe("#00ff00");
      expect(merged.colors?.background).toBe("#ffffff");
    });

    it("should return default theme when no themes provided", () => {
      const merged = mergeThemes();
      expect(merged.name).toBe("default");
    });

    it("should deep merge nested objects", () => {
      const theme1: ThemeConfig = {
        footer: { message: "Hello", copyright: "2024" },
      };
      const theme2: ThemeConfig = {
        footer: { copyright: "2025" },
      };

      const merged = mergeThemes(theme1, theme2);
      expect(merged.footer?.message).toBe("Hello");
      expect(merged.footer?.copyright).toBe("2025");
    });
  });

  describe("resolveTheme", () => {
    it("should resolve undefined to default theme", () => {
      const resolved = resolveTheme(undefined);
      expect(resolved.name).toBe("default");
      expect(resolved.colors.primary).toBe("#4f6fae");
    });

    it("should resolve extends chain", () => {
      const customTheme: ThemeConfig = {
        extends: defaultTheme,
        colors: { primary: "#3498db" },
        entryPage: { mode: "subtle" },
      };

      const resolved = resolveTheme(customTheme);
      expect(resolved.colors.primary).toBe("#3498db");
      expect(resolved.colors.background).toBe("#ffffff");
      expect(resolved.entryPage.mode).toBe("subtle");
    });

    it("should derive the code gradient top from a customized background", () => {
      const resolved = resolveTheme({
        extends: defaultTheme,
        colors: { codeBackground: "#f6f8fa", codeText: "#1f2328" },
      });

      expect(resolved.colors.codeBackgroundTop).toBe("#f6f8fa");
      expect(themeToNapi(resolved).colors?.codeBackgroundTop).toBe("#f6f8fa");
    });

    it("should preserve an explicit code gradient top", () => {
      const resolved = resolveTheme({
        colors: { codeBackground: "#f6f8fa", codeBackgroundTop: "#ffffff" },
      });

      expect(resolved.colors.codeBackgroundTop).toBe("#ffffff");
    });

    it("should resolve nested extends", () => {
      const baseTheme: ThemeConfig = {
        name: "base",
        colors: { primary: "#ff0000" },
      };
      const extendedTheme: ThemeConfig = {
        name: "extended",
        extends: baseTheme,
        colors: { background: "#ffffff" },
      };
      const finalTheme: ThemeConfig = {
        name: "final",
        extends: extendedTheme,
        footer: { message: "Hello" },
      };

      const resolved = resolveTheme(finalTheme);
      expect(resolved.name).toBe("final");
      expect(resolved.colors.primary).toBe("#ff0000");
      expect(resolved.colors.background).toBe("#ffffff");
      expect(resolved.footer.message).toBe("Hello");
    });

    it("should preserve explicit sidebar config", () => {
      const resolved = resolveTheme({
        sidebar: [
          {
            text: "Guide",
            items: [{ text: "Intro", link: "/intro" }],
            collapsed: true,
            stickyCollapsed: true,
          },
        ],
      });
      expect(resolved.sidebar).toEqual([
        {
          text: "Guide",
          items: [{ text: "Intro", link: "/intro" }],
          collapsed: true,
          stickyCollapsed: true,
        },
      ]);
    });
  });

  describe("themeToNapi", () => {
    it("should convert resolved theme to NAPI format", () => {
      const resolved = resolveTheme({
        colors: { primary: "#3498db" },
        footer: { message: "Test", copyright: "2025" },
        entryPage: { mode: "subtle" },
        header: { logoLight: "wordmark.svg", showSiteNameText: false },
        socialLinks: { github: "https://github.com/example" },
      });

      const napi = themeToNapi(resolved);
      expect(napi.colors?.primary).toBe("#3498db");
      expect(napi.entryPage?.mode).toBe("subtle");
      expect(napi.header?.showSiteNameText).toBe(false);
      expect(napi.footer?.message).toBe("Test");
      expect(napi.footer?.copyright).toBe("2025");
      expect(napi.socialLinks?.github).toBe("https://github.com/example");
    });

    it("should convert custom social links to NAPI format", () => {
      const svg = '<svg viewBox="0 0 24 24"><path d="M1 1h22v22H1z"/></svg>';
      const resolved = resolveTheme({
        socialLinks: [{ icon: { svg }, link: "https://example.com", ariaLabel: "Example" }],
      });

      const napi = themeToNapi(resolved);

      expect(napi.socialLinks?.links?.[0]).toEqual({
        icon: undefined,
        iconSvg: svg,
        link: "https://example.com",
        ariaLabel: "Example",
      });
    });

    it("should omit empty sections", () => {
      const resolved = resolveTheme(defaultTheme);
      const napi = themeToNapi(resolved);

      // header should be undefined when no logo is set
      expect(napi.header).toBeUndefined();
      // footer should be undefined when no message/copyright
      expect(napi.footer).toBeUndefined();
    });
  });
});

describe("theme composition", () => {
  const skin: ThemeConfig = {
    name: "skin",
    fonts: { sans: "SkinSans" },
    tokens: { "surface-glass": "#eee" },
    css: ".header { border-radius: 0; }",
  };
  const colorScheme: ThemeConfig = {
    name: "color",
    colors: { primary: "#111111" },
    darkColors: { primary: "#eeeeee" },
    tokens: { "brand-violet": "#7aa2f7" },
    darkTokens: { "brand-violet": "#bb9af7" },
  };

  it("concatenates css across layers instead of overwriting", () => {
    const merged = mergeThemes(
      { css: ".a { color: red; }" },
      { css: ".b { color: blue; }" },
      { css: ".c { color: green; }" },
    );

    expect(merged.css).toBe(".a { color: red; }\n.b { color: blue; }\n.c { color: green; }");
  });

  it("concatenates js across layers", () => {
    const merged = mergeThemes({ js: "a();" }, { js: "b();" });

    expect(merged.js).toBe("a();\nb();");
  });

  it("does not repeat a layer reached twice", () => {
    const merged = mergeThemes(skin, { extends: skin, css: skin.css });

    expect(merged.css).toBe(".header { border-radius: 0; }");
  });

  it("resolves an array of layers left to right", () => {
    const resolved = resolveTheme([skin, colorScheme]);

    expect(resolved.fonts.sans).toBe("SkinSans");
    expect(resolved.colors.primary).toBe("#111111");
    expect(resolved.darkColors.primary).toBe("#eeeeee");
    expect(resolved.css).toContain(".header { border-radius: 0; }");
  });

  it("merges tokens key-by-key with the last layer winning", () => {
    const resolved = resolveTheme([skin, colorScheme, { tokens: { "surface-glass": "#fafafa" } }]);

    expect(resolved.tokens).toEqual({
      "surface-glass": "#fafafa",
      "brand-violet": "#7aa2f7",
    });
    expect(resolved.darkTokens).toEqual({ "brand-violet": "#bb9af7" });
  });

  it("keeps later layer overrides for plain fields", () => {
    const resolved = resolveTheme([skin, colorScheme, { fonts: { sans: "OverrideSans" } }]);

    expect(resolved.fonts.sans).toBe("OverrideSans");
  });

  it("still applies the default theme underneath an array", () => {
    const resolved = resolveTheme([{ colors: { primary: "#abcdef" } }]);

    expect(resolved.colors.primary).toBe("#abcdef");
    expect(resolved.colors.background).toBe(defaultTheme.colors?.background);
  });

  it("falls back to the default theme for an empty array", () => {
    expect(resolveTheme([])).toEqual(resolveTheme(defaultTheme));
  });

  it("does not hang on a cyclic extends chain", () => {
    const cyclic: ThemeConfig = { name: "cyclic", colors: { primary: "#0f0f0f" } };
    cyclic.extends = cyclic;

    expect(resolveTheme(cyclic).colors.primary).toBe("#0f0f0f");
  });

  it("emits token css before the theme's own css", () => {
    const napi = themeToNapi(resolveTheme([skin, colorScheme]));
    const css = napi.css ?? "";

    expect(css).toContain("--octc-brand-violet: #7aa2f7;");
    expect(css).toContain('[data-theme="dark"]');
    expect(css).toContain("@media (prefers-color-scheme: dark)");
    expect(css.indexOf("--octc-surface-glass")).toBeLessThan(css.indexOf(".header"));
  });

  it("rejects a token name that would break out of the declaration block", () => {
    expect(() => themeToNapi(resolveTheme({ tokens: { "bad: red; }": "x" } }))).toThrow(
      /Invalid theme token name/,
    );
  });
});

describe("stable theme package layer composition", () => {
  const scheme = defineTheme({
    name: "scheme",
    colors: { primary: "#235fb1", background: "#e1e2e7", text: "#3760bf" },
    darkColors: { primary: "#7aa2f7", background: "#1a1b26", text: "#c0caf5" },
    tokens: {
      "shiki-foreground": "#3358b0",
      "shiki-background": "#d0d5e3",
      "shiki-token-keyword": "#9854f1",
    },
    darkTokens: {
      "shiki-foreground": "#c0caf5",
      "shiki-background": "#16161e",
      "shiki-token-keyword": "#bb9af7",
    },
  });
  const skin = defineTheme({
    name: "skin",
    layout: { sidebarWidth: "268px", headerHeight: "66px" },
    tokens: { "motion-base": "420ms" },
    css: ".skin-header { border-radius: 12px; }",
  });

  it("defineTheme + mergeThemes produce scheme tokens and skin css/layout", () => {
    const merged = mergeThemes(scheme, skin);

    expect(merged.colors?.primary).toBe("#235fb1");
    expect(merged.darkColors?.primary).toBe("#7aa2f7");
    expect(merged.tokens?.["shiki-foreground"]).toBe("#3358b0");
    expect(merged.tokens?.["shiki-token-keyword"]).toBe("#9854f1");
    expect(merged.tokens?.["motion-base"]).toBe("420ms");
    expect(merged.darkTokens?.["shiki-background"]).toBe("#16161e");
    expect(merged.layout?.sidebarWidth).toBe("268px");
    expect(merged.css).toContain(".skin-header { border-radius: 12px; }");
  });

  it("resolveTheme stacks a scheme layer and a skin layer left to right", () => {
    const resolved = resolveTheme([scheme, skin]);

    expect(resolved.tokens["shiki-foreground"]).toBe("#3358b0");
    expect(resolved.tokens["motion-base"]).toBe("420ms");
    expect(resolved.layout.sidebarWidth).toBe("268px");
    expect(resolved.css).toContain(".skin-header { border-radius: 12px; }");
    expect(resolved.colors.primary).toBe("#235fb1");
    expect(resolved.darkColors.background).toBe("#1a1b26");
  });
});
