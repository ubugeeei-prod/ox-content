import { describe, it, expect } from "vite-plus/test";
import { defineTheme, resolveTheme, themeToNapi } from "./theme";

describe("theme aside", () => {
  it("defaults to false on defineTheme and resolveTheme", () => {
    expect(defineTheme({}).aside).toBeUndefined();
    expect(resolveTheme(undefined).aside).toBe(false);
    expect(resolveTheme(defineTheme({})).aside).toBe(false);
    expect(themeToNapi(resolveTheme(undefined)).aside).toBe(false);
  });

  it("disables breadcrumbs by default", () => {
    expect(defineTheme({}).breadcrumbs).toBeUndefined();
    expect(resolveTheme(undefined).breadcrumbs).toBe(false);
    expect(resolveTheme(defineTheme({})).breadcrumbs).toBe(false);
    expect(themeToNapi(resolveTheme(undefined)).breadcrumbs).toBe(false);
  });

  it("enables breadcrumbs when breadcrumbs is true", () => {
    expect(defineTheme({ breadcrumbs: true }).breadcrumbs).toBe(true);
    expect(resolveTheme({ breadcrumbs: true }).breadcrumbs).toBe(true);
    expect(resolveTheme(defineTheme({ breadcrumbs: true })).breadcrumbs).toBe(true);
    expect(themeToNapi(resolveTheme({ breadcrumbs: true })).breadcrumbs).toBe(true);
  });

  it("enables breadcrumbs when an object is passed", () => {
    expect(resolveTheme({ breadcrumbs: {} }).breadcrumbs).toBe(true);
    expect(themeToNapi(resolveTheme({ breadcrumbs: {} })).breadcrumbs).toBe(true);
  });

  it("enables the outline when aside is true", () => {
    expect(defineTheme({ aside: true }).aside).toBe(true);
    expect(resolveTheme({ aside: true }).aside).toBe(true);
    expect(resolveTheme(defineTheme({ aside: true })).aside).toBe(true);
    expect(themeToNapi(resolveTheme({ aside: true })).aside).toBe(true);
  });
});
