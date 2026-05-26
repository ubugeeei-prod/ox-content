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
        sidebar: [{ text: "Guide", items: [{ text: "Intro", link: "/intro" }], collapsed: true }],
      });
      expect(resolved.sidebar).toEqual([
        { text: "Guide", items: [{ text: "Intro", link: "/intro" }], collapsed: true },
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
