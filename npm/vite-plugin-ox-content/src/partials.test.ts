import { describe, expect, it } from "vite-plus/test";
import { resolvePartialsOptions } from "./partials-options";

describe("partials option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolvePartialsOptions(undefined).enabled).toBe(false);
    expect(resolvePartialsOptions(false).enabled).toBe(false);
    expect(resolvePartialsOptions(true)).toEqual({
      enabled: true,
      root: "_partials",
      missing: "literal",
    });
    expect(resolvePartialsOptions({})).toEqual({
      enabled: true,
      root: "_partials",
      missing: "literal",
    });
    expect(resolvePartialsOptions({ rootDir: "docs", root: "snippets", missing: "error" })).toEqual(
      {
        enabled: true,
        rootDir: "docs",
        root: "snippets",
        missing: "error",
      },
    );
  });
});
