import { describe, expect, it } from "vite-plus/test";
import { resolveTimelineOptions, toJsTimelineOptions } from "./timeline-options";

describe("resolveTimelineOptions", () => {
  it("keeps timelines disabled when omitted or explicitly false", () => {
    expect(resolveTimelineOptions(undefined)).toEqual({
      enabled: false,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
    expect(resolveTimelineOptions(false)).toEqual({
      enabled: false,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
    expect(resolveTimelineOptions({ enabled: false })).toEqual({
      enabled: false,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
  });

  it("enables strict timeline diagnostics from true or object options", () => {
    expect(resolveTimelineOptions(true)).toEqual({
      enabled: true,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
    expect(resolveTimelineOptions({})).toEqual({
      enabled: true,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
  });

  it("normalizes diagnostic modes and keeps ordering overrides", () => {
    expect(
      resolveTimelineOptions({
        ordered: false,
        invalidDate: "warn",
        unknownMeta: "ignore",
        empty: "warn",
      }),
    ).toEqual({
      enabled: true,
      ordered: false,
      invalidDate: "warn",
      unknownMeta: "ignore",
      empty: "warn",
    });
    expect(
      resolveTimelineOptions({ invalidDate: "off" as never, unknownMeta: "bad" as never }),
    ).toEqual({
      enabled: true,
      ordered: true,
      invalidDate: "error",
      unknownMeta: "error",
      empty: "error",
    });
  });
});

describe("toJsTimelineOptions", () => {
  it("omits disabled timelines from native transform options", () => {
    expect(toJsTimelineOptions(resolveTimelineOptions(false))).toBeUndefined();
  });

  it("passes native timeline options with camel-case diagnostics", () => {
    expect(
      toJsTimelineOptions(
        resolveTimelineOptions({
          ordered: false,
          invalidDate: "warn",
          unknownMeta: "ignore",
          empty: "warn",
        }),
      ),
    ).toEqual({
      enabled: true,
      ordered: false,
      invalidDate: "warn",
      unknownMeta: "ignore",
      empty: "warn",
    });
  });
});
