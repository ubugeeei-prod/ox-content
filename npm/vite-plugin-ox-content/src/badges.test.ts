import { describe, expect, it } from "vite-plus/test";
import { resolveBadgeOptions } from "./index";

describe("resolveBadgeOptions", () => {
  it("omitted => false; true => true; {} => true", () => {
    expect(resolveBadgeOptions(undefined).enabled).toBe(false);
    expect(resolveBadgeOptions(true).enabled).toBe(true);
    expect(resolveBadgeOptions({}).enabled).toBe(true);
  });
});
