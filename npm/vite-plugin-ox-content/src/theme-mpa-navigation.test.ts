import { describe, expect, it } from "vite-plus/test";
import { defaultTheme, resolveTheme, themeToNapi } from "./theme";

describe("theme MPA navigation", () => {
  it("enables cross-document transitions in the built-in theme", () => {
    expect(defaultTheme.viewTransitions).toBe(true);
    expect(resolveTheme(undefined).viewTransitions).toBe(true);
    expect(themeToNapi(resolveTheme(undefined)).viewTransitions).toBe(true);
  });

  it("preserves an explicit transition opt-out through NAPI conversion", () => {
    const resolved = resolveTheme({ viewTransitions: false });

    expect(resolved.viewTransitions).toBe(false);
    expect(themeToNapi(resolved).viewTransitions).toBe(false);
  });
});
