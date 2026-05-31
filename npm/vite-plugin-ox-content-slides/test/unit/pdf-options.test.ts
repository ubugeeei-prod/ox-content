import { describe, expect, it } from "vitest";
import { resolveSlidePdfOptions } from "../../src/pdf-options";

describe("resolveSlidePdfOptions", () => {
  it("disables pdf export by default", () => {
    expect(resolveSlidePdfOptions(undefined)).toMatchObject({
      enabled: false,
      fileName: "deck.pdf",
    });
  });

  it("clamps invalid scales to safe bounds", () => {
    expect(resolveSlidePdfOptions({ scale: Number.NaN })).toMatchObject({ scale: 1 });
    expect(resolveSlidePdfOptions({ scale: 5 })).toMatchObject({ scale: 2 });
    expect(resolveSlidePdfOptions({ scale: 0 })).toMatchObject({ scale: 0.1 });
  });
});
