import { describe, expect, it } from "vite-plus/test";
import { defaultTheme, resolveTheme, themeToNapi } from "./theme";

describe("theme header logo", () => {
  it("defaults the header logo switch on without inventing a logo URL", () => {
    const resolved = resolveTheme(defaultTheme);

    expect(resolved.header.showLogo).toBe(true);
    expect(themeToNapi(resolved).header).toBeUndefined();
  });

  it("passes an explicit logo opt-out to NAPI without inventing a logo URL", () => {
    const napi = themeToNapi(resolveTheme({ header: { showLogo: false } }));

    expect(napi.header?.showLogo).toBe(false);
    expect(napi.header?.logo).toBeUndefined();
  });
});
