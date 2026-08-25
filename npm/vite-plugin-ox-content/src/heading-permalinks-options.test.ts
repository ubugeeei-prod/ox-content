import { describe, expect, it } from "vite-plus/test";
import { resolveHeadingPermalinksOptions } from "./heading-permalinks-options";

describe("resolveHeadingPermalinksOptions", () => {
  it("stays off when omitted or false", () => {
    expect(resolveHeadingPermalinksOptions(undefined)).toEqual({ enabled: false });
    expect(resolveHeadingPermalinksOptions(false)).toEqual({ enabled: false });
  });

  it("enables from true or an object", () => {
    expect(resolveHeadingPermalinksOptions(true)).toEqual({ enabled: true });
    expect(resolveHeadingPermalinksOptions({})).toEqual({ enabled: true });
    expect(resolveHeadingPermalinksOptions({ enabled: false })).toEqual({ enabled: false });
  });
});
