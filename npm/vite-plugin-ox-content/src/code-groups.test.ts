import { describe, expect, it } from "vite-plus/test";
import { resolveCodeGroupOptions } from "./code-group-options";

describe("codeGroups option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveCodeGroupOptions(undefined).enabled).toBe(false);
    expect(resolveCodeGroupOptions(false).enabled).toBe(false);
    expect(resolveCodeGroupOptions(true).enabled).toBe(true);
    expect(resolveCodeGroupOptions({}).enabled).toBe(true);
  });
});
