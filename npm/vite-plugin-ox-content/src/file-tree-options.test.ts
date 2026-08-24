import { describe, expect, it } from "vite-plus/test";
import { resolveFileTreeOptions } from "./file-tree-options";

describe("fileTree option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveFileTreeOptions(undefined).enabled).toBe(false);
    expect(resolveFileTreeOptions(false).enabled).toBe(false);
    expect(resolveFileTreeOptions(true).enabled).toBe(true);
    expect(resolveFileTreeOptions({}).enabled).toBe(true);
    expect(resolveFileTreeOptions({ enabled: false }).enabled).toBe(false);
  });
});
