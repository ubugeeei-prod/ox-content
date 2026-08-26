import { describe, expect, it } from "vite-plus/test";
import { resolveDataTableOptions, toJsDataTableOptions } from "./data-table-options";

describe("dataTables option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveDataTableOptions(undefined)).toEqual({
      enabled: false,
      missing: "error",
    });
    expect(resolveDataTableOptions(false).enabled).toBe(false);
    expect(resolveDataTableOptions(true)).toEqual({
      enabled: true,
      missing: "error",
    });
    expect(resolveDataTableOptions({}).enabled).toBe(true);
    expect(resolveDataTableOptions({ enabled: false }).enabled).toBe(false);
    expect(resolveDataTableOptions({ missing: "warn" })).toEqual({
      enabled: true,
      rootDir: undefined,
      missing: "warn",
    });
  });

  it("omits native options when the transform is off", () => {
    expect(toJsDataTableOptions(resolveDataTableOptions(false))).toBeUndefined();
    expect(toJsDataTableOptions(resolveDataTableOptions(true))).toEqual({
      enabled: true,
      rootDir: undefined,
      missing: "error",
    });
  });
});
