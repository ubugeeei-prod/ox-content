import { describe, expect, it, vi } from "vitest";
import { reportHeadDiagnostics, resolveHeadValidation } from "./page-head";

describe("resolveHeadValidation", () => {
  it("stays off unless warn or strict is set", () => {
    expect(resolveHeadValidation(undefined)).toBe(false);
    expect(resolveHeadValidation(false)).toBe(false);
    expect(resolveHeadValidation("off")).toBe(false);
    expect(resolveHeadValidation("warn")).toBe("warn");
    expect(resolveHeadValidation("strict")).toBe("strict");
  });
});

describe("reportHeadDiagnostics", () => {
  it("throws the first strict finding in strict mode", () => {
    expect(() =>
      reportHeadDiagnostics(
        [
          { strict: false, message: "empty" },
          { strict: true, message: "canonical is not a safe http(s) URL" },
        ],
        "strict",
      ),
    ).toThrow("canonical is not a safe http(s) URL");
  });

  it("warns in warn mode", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    reportHeadDiagnostics([{ strict: true, message: "bad href" }], "warn");
    expect(warn).toHaveBeenCalledWith("[ox-content] bad href");
    warn.mockRestore();
  });
});
