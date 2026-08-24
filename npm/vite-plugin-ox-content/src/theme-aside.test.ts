import { describe, it, expect } from "vite-plus/test";
import { defineTheme, resolveTheme, themeToNapi } from "./theme";

describe("theme aside", () => {
  it("defaults to false on defineTheme and resolveTheme", () => {
    expect(defineTheme({}).aside).toBeUndefined();
    expect(resolveTheme(undefined).aside).toBe(false);
    expect(resolveTheme(defineTheme({})).aside).toBe(false);
    expect(themeToNapi(resolveTheme(undefined)).aside).toBe(false);
  });

  it("enables the outline when aside is true", () => {
    expect(defineTheme({ aside: true }).aside).toBe(true);
    expect(resolveTheme({ aside: true }).aside).toBe(true);
    expect(resolveTheme(defineTheme({ aside: true })).aside).toBe(true);
    expect(themeToNapi(resolveTheme({ aside: true })).aside).toBe(true);
  });
});
