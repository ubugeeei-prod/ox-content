import fs from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path, { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };
import {
  createSolidHtmlHostLazyHydrate,
  initSolidHtmlHost,
  readSolidHtmlHostSlot,
  type SolidHtmlHostClientError,
} from "./html-host-client";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const require = createRequire(import.meta.url);

describe("Solid HTML host browser client", () => {
  it("declares a browser-only package subpath and pack entry", () => {
    const exportsField = packageJson.exports as unknown as Record<string, PackageConditionalExport>;
    const client = exportsField["./html-host/client"];

    expect(client.import.types).toBe("./dist/html-host-client.d.mts");
    expect(client.import.default).toBe("./dist/html-host-client.mjs");
    expect(client.require.types).toBe("./dist/html-host-client.d.cts");
    expect(client.require.default).toBe("./dist/html-host-client.cjs");

    const viteConfig = requireViteConfig();
    expect(viteConfig.default.pack.entry).toContain("src/html-host-client.ts");
  });

  it("bundles a minimal browser entry without Vite, Node or native dependencies", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-solid-html-client-"));
    try {
      await fs.writeFile(
        path.join(root, "entry.js"),
        [
          'import { createSolidHtmlHostLazyHydrate } from "@ox-content/vite-plugin-solid/html-host/client";',
          "console.log(typeof createSolidHtmlHostLazyHydrate);",
        ].join("\n"),
      );

      const output = await build({
        root,
        configFile: false,
        logLevel: "silent",
        resolve: {
          alias: {
            "@ox-content/vite-plugin-solid/html-host/client": fileURLToPath(
              new URL("./html-host-client.ts", import.meta.url),
            ),
            "@ox-content/islands": fileURLToPath(
              new URL("../../ox-content-islands/src/index.ts", import.meta.url),
            ),
          },
        },
        build: {
          write: false,
          lib: {
            entry: path.join(root, "entry.js"),
            formats: ["es"],
          },
        },
      });
      const bundle = bundleCode(output);

      expect(bundle).not.toContain("@ox-content/vite-plugin");
      expect(bundle).not.toContain("node:");
      expect(bundle).not.toContain("fsevents");
      expect(bundle).not.toContain("resvg");
    } finally {
      await fs.rm(root, { force: true, recursive: true });
    }
  });

  it("loads distinct document-scoped modules for reused local component names", async () => {
    const calls: string[] = [];
    const hydrate = createSolidHtmlHostLazyHydrate({
      modules: {
        "./docs/a/Chart.tsx": () => ({ default: "chart-a" }),
        "./docs/b/Chart.tsx": () => ({ default: "chart-b" }),
      },
      render: ({ component, moduleId }) => {
        calls.push(`${moduleId}:${String(component)}`);
      },
    });

    hydrate(
      element({ oxIsland: "Chart", oxModule: "./docs/a/Chart.tsx", oxExport: "default" }),
      {},
    );
    hydrate(
      element({ oxIsland: "Chart", oxModule: "./docs/b/Chart.tsx", oxExport: "default" }),
      {},
    );
    await settle();

    expect(calls).toEqual(["./docs/a/Chart.tsx:chart-a", "./docs/b/Chart.tsx:chart-b"]);
  });

  it("passes a synchronous lazy hydrate function to the supplied initIslands", () => {
    const controller = { destroy: () => {} };
    let received:
      | {
          hydrate: (element: HTMLElement, props: Record<string, unknown>) => unknown;
          selector?: string;
        }
      | undefined;

    const returned = initSolidHtmlHost({
      initIslands(hydrate, options) {
        received = { hydrate, selector: options?.selector };
        return controller;
      },
      modules: { "./Chart.tsx": () => ({ default: "chart" }) },
      render: () => {},
      options: { selector: ".ox-content [data-ox-island]" },
    });

    expect(returned).toBe(controller);
    expect(typeof received?.hydrate).toBe("function");
    expect(received?.selector).toBe(".ox-content [data-ox-island]");
  });

  it("reports unknown modules, load failures, missing exports and render failures", async () => {
    const errors: SolidHtmlHostClientError[] = [];
    const hydrate = createSolidHtmlHostLazyHydrate({
      modules: {
        "./Bad.tsx": () => Promise.reject(new Error("network gone")),
        "./Missing.tsx": () => ({}),
        "./Render.tsx": () => ({ default: "component" }),
      },
      render: () => {
        throw new Error("render boom");
      },
      onError: (error) => errors.push(error),
    });

    hydrate(element({ oxIsland: "Unknown", oxModule: "./Unknown.tsx" }), {});
    hydrate(element({ oxIsland: "Bad", oxModule: "./Bad.tsx" }), {});
    hydrate(element({ oxIsland: "Missing", oxModule: "./Missing.tsx" }), {});
    hydrate(element({ oxIsland: "Render", oxModule: "./Render.tsx" }), {});
    await settle();

    expect(errors.map((error) => error.code).sort()).toEqual([
      "missing-export",
      "module-load-failed",
      "render-failed",
      "unknown-module",
    ]);
    expect(errors.map((error) => error.message).join("\n")).toContain("network gone");
    expect(errors.map((error) => error.message).join("\n")).toContain("render boom");
  });

  it("cancels stale pending loads and disposes mounted islands exactly once", async () => {
    const pending = deferred<Record<string, unknown>>();
    let renders = 0;
    let disposals = 0;
    const hydrate = createSolidHtmlHostLazyHydrate({
      modules: { "./Slow.tsx": () => pending.promise },
      render: () => {
        renders += 1;
        return () => {
          disposals += 1;
        };
      },
    });

    const staleDispose = hydrate(element({ oxIsland: "Slow", oxModule: "./Slow.tsx" }), {});
    staleDispose();
    pending.resolve({ default: "slow" });
    await settle();
    expect(renders).toBe(0);

    const dispose = hydrate(element({ oxIsland: "Slow", oxModule: "./Slow.tsx" }), {});
    await settle();
    dispose();
    dispose();

    expect(renders).toBe(1);
    expect(disposals).toBe(1);
  });

  it("passes authored slots without treating self-closing SSR output as children", async () => {
    const calls: Array<{ component: string; slotHtml: string | undefined }> = [];
    const hydrate = createSolidHtmlHostLazyHydrate({
      modules: {
        "./Slot.tsx": () => ({ default: "slot" }),
        "./Self.tsx": () => ({ default: "self" }),
      },
      render: ({ component, slotHtml }) => {
        calls.push({ component: String(component), slotHtml });
      },
    });
    const slot = element(
      { oxIsland: "Slot", oxModule: "./Slot.tsx", oxSsr: "true", oxContent: "<em>authored</em>" },
      "<strong>SSR output</strong>",
    );
    const selfClosing = element(
      { oxIsland: "Self", oxModule: "./Self.tsx", oxSsr: "true" },
      "<strong>SSR output</strong>",
    );

    hydrate(slot, {});
    hydrate(selfClosing, {});
    await settle();

    expect(calls).toEqual([
      { component: "slot", slotHtml: "<em>authored</em>" },
      { component: "self", slotHtml: undefined },
    ]);
    expect(slot.innerHTML).toBe("");
    expect(selfClosing.innerHTML).toBe("");
    expect(
      readSolidHtmlHostSlot(
        element({}, '<script type="application/json">{"props":{}}</script><em>fallback</em>'),
      ),
    ).toBe("<em>fallback</em>");
  });
});

interface PackageConditionalExport {
  import: { types: string; default: string };
  require: { types: string; default: string };
}

function requireViteConfig(): { default: { pack: { entry: string[] } } } {
  const configPath = join(packageRoot, "vite.config.ts");
  return require(configPath) as { default: { pack: { entry: string[] } } };
}

function bundleCode(output: unknown): string {
  const outputs = Array.isArray(output) ? output : [output];
  return outputs
    .flatMap((result) => {
      const chunks = (result as { output?: unknown }).output;
      return Array.isArray(chunks) ? chunks : [];
    })
    .map((chunk) => {
      const outputChunk = chunk as { type?: string; code?: string };
      return outputChunk.type === "chunk" ? (outputChunk.code ?? "") : "";
    })
    .join("\n");
}

function element(
  dataset: Record<string, string> = {},
  innerHTML = "",
): HTMLElement & { events: unknown[] } {
  const classes = new Set<string>();
  const events: unknown[] = [];
  return {
    dataset: { ...dataset },
    innerHTML,
    classList: {
      add: (...names: string[]) => names.forEach((name) => classes.add(name)),
      remove: (...names: string[]) => names.forEach((name) => classes.delete(name)),
    },
    dispatchEvent(event: Event) {
      events.push(event);
      return true;
    },
    events,
  } as unknown as HTMLElement & { events: unknown[] };
}

function settle(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function deferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (error: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
