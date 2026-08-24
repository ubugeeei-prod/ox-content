import { describe, expect, it } from "vite-plus/test";
import { resolveIncludeOptions } from "./include-options";

describe("includes option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveIncludeOptions(undefined).enabled).toBe(false);
    expect(resolveIncludeOptions(false).enabled).toBe(false);
    expect(resolveIncludeOptions(true).enabled).toBe(true);
    expect(resolveIncludeOptions({}).enabled).toBe(true);
    expect(resolveIncludeOptions({ rootDir: "docs" })).toEqual({
      enabled: true,
      rootDir: "docs",
    });
  });
});
