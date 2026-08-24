import { describe, expect, it } from "vite-plus/test";
import { resolveStepsOptions } from "./step-options";

describe("steps option resolution", () => {
  it("treats omitted as disabled, and true or {} as enabled", () => {
    expect(resolveStepsOptions(undefined).enabled).toBe(false);
    expect(resolveStepsOptions(false).enabled).toBe(false);
    expect(resolveStepsOptions(true).enabled).toBe(true);
    expect(resolveStepsOptions({}).enabled).toBe(true);
    expect(resolveStepsOptions({ enabled: false }).enabled).toBe(false);
  });
});
