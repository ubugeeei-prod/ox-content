import { describe, expect, it } from "vitest";
import { resolveOptions } from "../../src/options";
import { getSlideHref, getSlideRouteLookupKey, normalizeRouteSegment } from "../../src/path-utils";

describe("path utils", () => {
  it("normalizes route segments", () => {
    expect(normalizeRouteSegment("\\foo//bar/")).toBe("foo/bar");
  });

  it("builds slide hrefs with base paths", () => {
    expect(getSlideHref("/docs", "slides", "demo", 3, ".html")).toBe(
      "/docs/slides/demo/3/index.html",
    );
  });

  it("maps request URLs back to slide routes", () => {
    const options = resolveOptions({ base: "/docs", routeBase: "slides" });

    expect(getSlideRouteLookupKey(options, "/docs/slides/presenter/2/")).toBe(
      "/slides/presenter/2",
    );
    expect(getSlideRouteLookupKey(options, "/docs/notes/1/")).toBeNull();
  });
});
