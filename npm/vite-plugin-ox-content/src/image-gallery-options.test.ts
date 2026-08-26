import { describe, expect, it } from "vite-plus/test";
import { resolveImageGalleryOptions, toJsImageGalleryOptions } from "./image-gallery-options";

describe("resolveImageGalleryOptions", () => {
  it("keeps galleries disabled when omitted or explicitly false", () => {
    expect(resolveImageGalleryOptions(undefined)).toEqual({
      enabled: false,
      lazy: true,
      missingAlt: "error",
      empty: "error",
    });
    expect(resolveImageGalleryOptions(false)).toEqual({
      enabled: false,
      lazy: true,
      missingAlt: "error",
      empty: "error",
    });
    expect(resolveImageGalleryOptions({ enabled: false })).toEqual({
      enabled: false,
      lazy: true,
      missingAlt: "error",
      empty: "error",
    });
  });

  it("enables strict gallery diagnostics from true or object options", () => {
    expect(resolveImageGalleryOptions(true)).toEqual({
      enabled: true,
      missingAlt: "error",
      empty: "error",
    });
    expect(resolveImageGalleryOptions({})).toEqual({
      enabled: true,
      lazy: undefined,
      missingAlt: "error",
      empty: "error",
    });
  });

  it("normalizes diagnostic modes and keeps an explicit lazy override", () => {
    expect(
      resolveImageGalleryOptions({
        lazy: false,
        missingAlt: "warn",
        empty: "ignore",
      }),
    ).toEqual({
      enabled: true,
      lazy: false,
      missingAlt: "warn",
      empty: "ignore",
    });
    expect(
      resolveImageGalleryOptions({ missingAlt: "off" as never, empty: "bad" as never }),
    ).toEqual({
      enabled: true,
      lazy: undefined,
      missingAlt: "error",
      empty: "error",
    });
  });
});

describe("toJsImageGalleryOptions", () => {
  it("omits disabled galleries from native transform options", () => {
    expect(
      toJsImageGalleryOptions(resolveImageGalleryOptions(false), { enabled: true, lazy: true }),
    ).toBeUndefined();
  });

  it("inherits lazy loading from resolved image options", () => {
    expect(
      toJsImageGalleryOptions(resolveImageGalleryOptions(true), { enabled: true, lazy: false }),
    ).toEqual({
      enabled: true,
      lazy: false,
      missingAlt: "error",
      empty: "error",
    });
  });

  it("uses explicit gallery lazy and diagnostic modes", () => {
    expect(
      toJsImageGalleryOptions(
        resolveImageGalleryOptions({ lazy: false, missingAlt: "warn", empty: "ignore" }),
        { enabled: true, lazy: true },
      ),
    ).toEqual({
      enabled: true,
      lazy: false,
      missingAlt: "warn",
      empty: "ignore",
    });
  });
});
