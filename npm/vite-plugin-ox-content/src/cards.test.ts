import { describe, expect, it } from "vite-plus/test";
import { resolveCardOptions } from "./card-options";

describe("cards option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveCardOptions(undefined).enabled).toBe(false);
    expect(resolveCardOptions(false).enabled).toBe(false);
    expect(resolveCardOptions(true).enabled).toBe(true);
    expect(resolveCardOptions({}).enabled).toBe(true);
  });
});
