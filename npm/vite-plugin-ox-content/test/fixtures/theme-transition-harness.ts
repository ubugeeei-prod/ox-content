import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../../package.json", import.meta.url)));
const runtimeSource = readFileSync(
  join(packageRoot, "../../crates/ox_content_ssg/src/html/theme_transition_runtime.js"),
  "utf8",
);

export const TRANSITION_ATTR = "data-ox-theme-transition";
export const SUPPRESS_ATTR = "data-ox-theme-transition-suppress";

export type Animation = { keyframes: Record<string, unknown>; options: Record<string, unknown> };

export type Harness = {
  applyThemeTransition: (options: Record<string, unknown>) => Promise<void>;
  attributes: Map<string, string>;
  attributeLog: string[];
  animations: Animation[];
  applied: number;
  startCalls: number;
  skips: number;
  settle: () => Promise<void>;
  skipNext: boolean;
  failReady: boolean;
};

/**
 * The runtime is plain function declarations so the SSG can inline it, which
 * also means it can be instantiated against stub globals without a DOM.
 */
export function harness(options: { reducedMotion?: boolean; supported?: boolean } = {}): Harness {
  const { reducedMotion = false, supported = true } = options;
  const state = {
    attributes: new Map<string, string>(),
    attributeLog: [] as string[],
    animations: [] as Animation[],
    applied: 0,
    startCalls: 0,
    skips: 0,
    skipNext: false,
    failReady: false,
  };

  let resolveReady: () => void = () => {};
  let rejectReady: () => void = () => {};
  let resolveFinished: () => void = () => {};
  let rejectFinished: () => void = () => {};

  const root = {
    offsetHeight: 0,
    setAttribute(name: string, value: string) {
      state.attributes.set(name, value);
      state.attributeLog.push(`+${name}`);
    },
    removeAttribute(name: string) {
      state.attributes.delete(name);
      state.attributeLog.push(`-${name}`);
    },
    getAttribute(name: string) {
      return state.attributes.get(name) ?? null;
    },
    animate(keyframes: Record<string, unknown>, animationOptions: Record<string, unknown>) {
      state.animations.push({ keyframes, options: animationOptions });
      return { finished: Promise.resolve() };
    },
  };

  const documentStub: Record<string, unknown> = { documentElement: root };
  if (supported) {
    documentStub.startViewTransition = (callback: () => void) => {
      state.startCalls += 1;
      callback();
      const ready = new Promise<void>((resolve, reject) => {
        resolveReady = resolve;
        rejectReady = reject;
      });
      const finished = new Promise<void>((resolve, reject) => {
        resolveFinished = resolve;
        rejectFinished = reject;
      });
      // An unobserved rejection here would be reported as unhandled; the
      // runtime is expected to attach handlers to both.
      return {
        ready,
        finished,
        skipTransition() {
          state.skips += 1;
          rejectReady(new Error("skipped"));
          rejectFinished(new Error("skipped"));
        },
      };
    };
  }

  const factory = new Function(
    "document",
    "matchMedia",
    "innerWidth",
    "innerHeight",
    `${runtimeSource}\nreturn applyThemeTransition;`,
  ) as (
    doc: unknown,
    mm: (query: string) => { matches: boolean },
    width: number,
    height: number,
  ) => Harness["applyThemeTransition"];

  const applyThemeTransition = factory(
    documentStub,
    (query: string) => ({ matches: reducedMotion && query.includes("reduce") }),
    1000,
    500,
  );

  const h: Harness = {
    applyThemeTransition,
    attributes: state.attributes,
    attributeLog: state.attributeLog,
    animations: state.animations,
    applied: 0,
    startCalls: 0,
    skips: 0,
    skipNext: false,
    failReady: false,
    async settle() {
      if (h.failReady) rejectReady(new Error("unsupported"));
      else resolveReady();
      await Promise.resolve();
      resolveFinished();
      await Promise.resolve();
      await Promise.resolve();
    },
  };
  // `startCalls` and `skips` live on the stub, so mirror them on read.
  Object.defineProperties(h, {
    startCalls: { get: () => state.startCalls },
    skips: { get: () => state.skips },
  });
  return h;
}
