import { describe, it, expect, beforeEach } from "vite-plus/test";
import {
  transformIslands,
  hasIslands,
  extractIslandInfo,
  generateHydrationScript,
  resetIslandCounter,
} from "./parse";

describe("island/parse", () => {
  beforeEach(() => {
    resetIslandCounter();
  });

  describe("hasIslands", () => {
    it("detects <island> regardless of case", () => {
      expect(hasIslands('<Island load="visible"></Island>')).toBe(true);
      expect(hasIslands("<island></island>")).toBe(true);
    });

    it("returns false when no island element is present", () => {
      expect(hasIslands("<p>plain</p>")).toBe(false);
      // The boundary check means a longer tag name does not match.
      expect(hasIslands("<islander></islander>")).toBe(false);
    });
  });

  describe("transformIslands", () => {
    it("rewrites an <Island> into a data-attributed wrapper and collects its info", async () => {
      const { html, islands } = await transformIslands(
        '<Island load="visible"><Counter initial={0} /></Island>',
      );

      expect(islands).toHaveLength(1);
      expect(islands[0].load).toBe("visible");
      expect(islands[0].props).toEqual({ initial: 0 });

      expect(html).toContain('id="ox-island-0"');
      expect(html).toContain('data-ox-load="visible"');
      expect(html).toContain('class="ox-island"');
      expect(html).toContain("data-ox-island=");
    });

    it("defaults the load strategy to eager when omitted", async () => {
      const { islands } = await transformIslands("<Island><Widget /></Island>");
      expect(islands).toHaveLength(1);
      expect(islands[0].load).toBe("eager");
    });

    it("records a media query when present", async () => {
      const { html, islands } = await transformIslands(
        '<Island load="media" media="(min-width: 768px)"><Widget /></Island>',
      );
      expect(islands[0].mediaQuery).toBe("(min-width: 768px)");
      expect(html).toContain('data-ox-media="(min-width: 768px)"');
    });

    it("parses JSX-style prop values into JS primitives and JSON", async () => {
      const { islands } = await transformIslands(
        '<Island><Widget count={42} active={true} label="hi" config={{"a":1}} /></Island>',
      );
      expect(islands[0].props).toMatchObject({
        count: 42,
        active: true,
        label: "hi",
        config: { a: 1 },
      });
    });

    it("increments island ids across multiple islands", async () => {
      const { html, islands } = await transformIslands(
        "<Island><A /></Island><Island><B /></Island>",
      );
      expect(islands).toHaveLength(2);
      expect(html).toContain('id="ox-island-0"');
      expect(html).toContain('id="ox-island-1"');
    });

    it("leaves island-free HTML without any island wrappers", async () => {
      const { html, islands } = await transformIslands("<p>just prose</p>");
      expect(islands).toHaveLength(0);
      expect(html).not.toContain("data-ox-island");
    });
  });

  describe("extractIslandInfo", () => {
    it("returns the island metadata without requiring the caller to read the html", async () => {
      const islands = await extractIslandInfo('<Island load="idle"><Widget /></Island>');
      expect(islands).toHaveLength(1);
      expect(islands[0].load).toBe("idle");
    });
  });

  describe("generateHydrationScript", () => {
    it("returns an empty string when there are no components", () => {
      expect(generateHydrationScript([])).toBe("");
    });

    it("imports each component and wires up initIslands", () => {
      const script = generateHydrationScript(["Counter", "Chart"]);
      expect(script).toContain("import Counter from './Counter';");
      expect(script).toContain("import Chart from './Chart';");
      expect(script).toContain("initIslands(");
      expect(script).toContain("@ox-content/islands");
    });
  });
});
