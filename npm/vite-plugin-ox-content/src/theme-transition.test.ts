import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };
import { defaultTheme, resolveTheme, themeToNapi } from "./theme";
import { SUPPRESS_ATTR, TRANSITION_ATTR, harness } from "../test/fixtures/theme-transition-harness";

type PackageConditionalExport = {
  import: { types: string; default: string };
  require: { types: string; default: string };
};

describe("theme transition public API", () => {
  it("declares the client and stylesheet subpaths", () => {
    const exportsField = packageJson.exports as Record<string, PackageConditionalExport | string>;

    expect(exportsField["./styles/theme-transition.css"]).toBe(
      "./dist/styles/theme-transition.css",
    );

    const client = exportsField["./theme-transition/client"] as PackageConditionalExport;
    expect(client.import.types).toBe("./dist/theme-transition-client.d.mts");
    expect(client.import.default).toBe("./dist/theme-transition-client.mjs");
    expect(client.require.types).toBe("./dist/theme-transition-client.d.cts");
    expect(client.require.default).toBe("./dist/theme-transition-client.cjs");
  });

  it("keeps the built-in toggle opt-in and separate from navigation transitions", () => {
    expect(defaultTheme.toggleTransition).toBe(false);
    expect(resolveTheme({}).toggleTransition).toBe(false);
    expect(resolveTheme({ toggleTransition: "circle" }).toggleTransition).toBe("circle");

    // The Rust side reads a string and treats anything unknown as off, so an
    // explicit `false` has to arrive as absent rather than as "false".
    expect(themeToNapi(resolveTheme({})).toggleTransition).toBeUndefined();
    expect(themeToNapi(resolveTheme({ toggleTransition: "circle" })).toggleTransition).toBe(
      "circle",
    );
    // Cross-document navigation keeps its own lifecycle.
    expect(resolveTheme({ toggleTransition: "circle" }).viewTransitions).toBe(true);
  });
});

describe("applyThemeTransition fallbacks", () => {
  it("applies immediately when the reader asked for reduced motion", async () => {
    const h = harness({ reducedMotion: true });
    let applied = 0;
    await h.applyThemeTransition({ nextTheme: "dark", apply: () => (applied += 1) });

    expect(applied).toBe(1);
    expect(h.startCalls).toBe(0);
    expect(h.attributes.size).toBe(0);
  });

  it("applies immediately when View Transitions are missing", async () => {
    const h = harness({ supported: false });
    let applied = 0;
    await h.applyThemeTransition({ nextTheme: "dark", apply: () => (applied += 1) });

    expect(applied).toBe(1);
    expect(h.animations).toHaveLength(0);
  });

  it("does nothing without a theme mutation to run", async () => {
    const h = harness();
    await h.applyThemeTransition({ nextTheme: "dark" });
    expect(h.startCalls).toBe(0);
  });
});

describe("applyThemeTransition reveal geometry", () => {
  it("grows the incoming snapshot when switching to dark", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({
      event: { clientX: 100, clientY: 50, detail: 1 } as unknown as Event,
      nextTheme: "dark",
      apply: () => {},
    });
    await h.settle();
    await pending;

    expect(h.animations).toHaveLength(1);
    const [animation] = h.animations;
    expect(animation.options.pseudoElement).toBe("::view-transition-new(root)");
    const clip = animation.keyframes.clipPath as string[];
    expect(clip[0]).toBe("circle(0px at 100px 50px)");
    // Furthest corner from (100, 50) in a 1000x500 viewport is (1000, 500).
    expect(clip[1]).toBe(`circle(${Math.hypot(900, 450)}px at 100px 50px)`);
  });

  it("collapses the outgoing snapshot when switching to light", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({
      event: { clientX: 0, clientY: 0, detail: 1 } as unknown as Event,
      nextTheme: "light",
      apply: () => {},
    });
    await h.settle();
    await pending;

    const [animation] = h.animations;
    expect(animation.options.pseudoElement).toBe("::view-transition-old(root)");
    const clip = animation.keyframes.clipPath as string[];
    expect(clip[0]).toBe(`circle(${Math.hypot(1000, 500)}px at 0px 0px)`);
    expect(clip[1]).toBe("circle(0px at 0px 0px)");
  });

  it("falls back to the trigger centre for a keyboard activation", async () => {
    const h = harness();
    // A keyboard-driven click reports detail 0 and no coordinates.
    const pending = h.applyThemeTransition({
      event: {
        clientX: 0,
        clientY: 0,
        detail: 0,
        currentTarget: {
          getBoundingClientRect: () => ({ left: 200, top: 100, width: 40, height: 20 }),
        },
      } as unknown as Event,
      nextTheme: "dark",
      apply: () => {},
    });
    await h.settle();
    await pending;

    const clip = h.animations[0].keyframes.clipPath as string[];
    expect(clip[0]).toBe("circle(0px at 220px 110px)");
  });

  it("falls back to the viewport centre with no event at all", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({ nextTheme: "dark", apply: () => {} });
    await h.settle();
    await pending;

    const clip = h.animations[0].keyframes.clipPath as string[];
    expect(clip[0]).toBe("circle(0px at 500px 250px)");
  });

  it("honours a caller's duration and easing", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({
      nextTheme: "dark",
      apply: () => {},
      duration: 1200,
      easing: "linear",
    });
    await h.settle();
    await pending;

    expect(h.animations[0].options.duration).toBe(1200);
    expect(h.animations[0].options.easing).toBe("linear");
  });
});

describe("applyThemeTransition cleanup", () => {
  it("suppresses element transitions only while the theme is mutated", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({ nextTheme: "dark", apply: () => {} });

    // The mutation runs inside startViewTransition, so suppression is on.
    expect(h.attributes.has(SUPPRESS_ATTR)).toBe(true);
    await h.settle();
    await pending;
    expect(h.attributes.has(SUPPRESS_ATTR)).toBe(false);
  });

  it("leaves no attribute behind once the transition settles", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({ nextTheme: "dark", apply: () => {} });
    expect(h.attributes.get(TRANSITION_ATTR)).toBe("expand");

    await h.settle();
    await pending;
    expect(h.attributes.has(TRANSITION_ATTR)).toBe(false);
  });

  it("settles cleanly when the transition is skipped rather than finished", async () => {
    const h = harness();
    let applied = 0;
    const pending = h.applyThemeTransition({
      nextTheme: "light",
      apply: () => (applied += 1),
    });

    // A skip rejects both `ready` and `finished`; neither may escape.
    const second = h.applyThemeTransition({ nextTheme: "dark", apply: () => (applied += 1) });
    await h.settle();
    await Promise.all([pending, second]);

    expect(applied).toBe(2);
    expect(h.skips).toBe(1);
    expect(h.attributes.has(SUPPRESS_ATTR)).toBe(false);
  });

  it("lets the newer toggle keep the attribute when it interrupts an older one", async () => {
    const h = harness();
    const first = h.applyThemeTransition({ nextTheme: "light", apply: () => {} });
    expect(h.attributes.get(TRANSITION_ATTR)).toBe("shrink");

    // The second toggle skips the first. The first one's cleanup must not
    // reach in and clear the direction the second one just claimed.
    const second = h.applyThemeTransition({ nextTheme: "dark", apply: () => {} });
    await Promise.resolve();
    await Promise.resolve();
    expect(h.attributes.get(TRANSITION_ATTR)).toBe("expand");

    await h.settle();
    await Promise.all([first, second]);
    expect(h.attributes.has(TRANSITION_ATTR)).toBe(false);
  });

  it("hands the suppressed styles back even when the snapshot never becomes ready", async () => {
    const h = harness();
    const pending = h.applyThemeTransition({ nextTheme: "dark", apply: () => {} });
    expect(h.attributes.has(SUPPRESS_ATTR)).toBe(true);

    h.failReady = true;
    await h.settle();
    await pending;

    expect(h.attributes.has(SUPPRESS_ATTR)).toBe(false);
    expect(h.animations).toHaveLength(0);
  });
});
