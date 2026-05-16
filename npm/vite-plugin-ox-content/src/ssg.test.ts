import { describe, expect, it } from "vite-plus/test";
import { resolveSsgOptions } from "./ssg";

describe("resolveSsgOptions", () => {
  it("disables git timestamps by default", () => {
    expect(resolveSsgOptions(undefined).lastUpdated).toBe(false);
  });

  it("enables git timestamps when requested", () => {
    expect(resolveSsgOptions({ lastUpdated: true }).lastUpdated).toBe(true);
  });
});
