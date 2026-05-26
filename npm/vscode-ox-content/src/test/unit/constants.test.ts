import { describe, expect, it } from "vitest";

import * as constants from "../../constants";

describe("constants", () => {
  it("uses the `oxContent.` namespace for every client-facing command", () => {
    for (const [name, value] of Object.entries(constants)) {
      if (!name.startsWith("COMMAND_")) continue;
      expect(value, `${name} should be a string`).toEqual(expect.any(String));
      expect(value).toMatch(/^oxContent\./);
    }
  });

  it("uses a `oxContent.` namespace for LSP-side commands too", () => {
    expect(constants.SERVER_COMMAND_PREVIEW_HTML).toMatch(/^oxContent\./);
  });

  it("does not duplicate command identifiers across constants", () => {
    const values = Object.values(constants).filter((v): v is string => typeof v === "string");
    expect(new Set(values).size).toBe(values.length);
  });
});
