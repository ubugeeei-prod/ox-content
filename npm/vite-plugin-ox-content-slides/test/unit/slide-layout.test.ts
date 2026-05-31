import { describe, expect, it } from "vitest";
import {
  resolveSlideLayout,
  resolveSlidePlacements,
  wrapSlideContent,
} from "../../src/slide-layout";

describe("slide layout metadata", () => {
  it("normalizes frontmatter layout tokens", () => {
    expect(
      resolveSlideLayout({
        layout: "Canvas",
        align: "center",
        density: "spacious",
        accent: "#176b5d",
      }),
    ).toEqual({
      layout: "canvas",
      align: "center",
      density: "spacious",
      accent: "#176b5d",
    });
  });

  it("falls back to stable defaults for unknown values", () => {
    expect(
      resolveSlideLayout({
        layout: "absolute",
        align: "middle",
        density: "huge",
        accent: "url(javascript:alert(1))",
      }),
    ).toEqual({
      layout: "stack",
      align: "start",
      density: "balanced",
      accent: undefined,
    });
  });

  it("wraps rendered html with classes and accent token", () => {
    expect(wrapSlideContent("<h1>Hello</h1>", { layout: "quote", accent: "#b54" })).toContain(
      'class="ox-slide-layout ox-slide-layout--quote ox-slide-align--start ox-slide-density--balanced" style="--ox-slide-accent: #b54"',
    );
  });

  it("emits canvas placement styles from JSON frontmatter", () => {
    const html = wrapSlideContent("<h1>Hello</h1><p>World</p>", {
      layout: "canvas",
      placements: '[{"x":10,"y":12,"w":40,"h":20},{"x":55,"y":24,"w":35,"h":52}]',
    });

    expect(resolveSlidePlacements({ placements: '[{"x":10,"y":12,"w":40,"h":20}]' })).toEqual([
      { x: 10, y: 12, w: 40, h: 20 },
    ]);
    expect(html).toContain('data-ox-has-placements="true"');
    expect(html).toContain("left:10.000%;top:12.000%;width:40.000%;height:20.000%;");
    expect(html).toContain("left:55.000%;top:24.000%;width:35.000%;height:52.000%;");
  });
});
