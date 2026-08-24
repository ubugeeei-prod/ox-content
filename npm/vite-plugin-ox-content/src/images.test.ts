import { describe, expect, it } from "vite-plus/test";
import { resolveImageOptions } from "./resolve-image-options";

describe("resolveImageOptions", () => {
  it("omitted => false", () => {
    expect(resolveImageOptions(undefined)).toEqual({ enabled: false, lazy: true });
  });

  it("true => true", () => {
    expect(resolveImageOptions(true)).toEqual({ enabled: true, lazy: true });
  });

  it("{} => true", () => {
    expect(resolveImageOptions({})).toEqual({ enabled: true, lazy: true });
  });
});
