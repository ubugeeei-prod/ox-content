import { describe, expect, it } from "vite-plus/test";
import type { Plugin, ResolvedConfig } from "vite";
import { oxContentSolid } from "./index";
import type { SolidIntegrationOptions } from "./types";

describe("oxContentSolid", () => {
  it("returns the transform, verify, environment and hmr plugins", () => {
    const names = pluginNames(oxContentSolid());

    expect(names).toContain("ox-content:solid-transform");
    expect(names).toContain("ox-content:solid-verify");
    expect(names).toContain("ox-content:solid-environment");
    expect(names).toContain("ox-content:solid-hmr");
  });

  it("accepts a config where vite-plugin-solid runs after it", async () => {
    await expect(
      resolveConfigWith({}, ["ox-content:solid-transform", "solid"]),
    ).resolves.toBeUndefined();
  });

  it("rejects a config without vite-plugin-solid", async () => {
    await expect(resolveConfigWith({}, ["ox-content:solid-transform"])).rejects.toThrow(
      /vite-plugin-solid was not found/,
    );
  });

  it("rejects a config where vite-plugin-solid runs first", async () => {
    // Solid would receive raw Markdown instead of the generated JSX.
    await expect(resolveConfigWith({}, ["solid", "ox-content:solid-transform"])).rejects.toThrow(
      /runs before oxContentSolid\(\)/,
    );
  });

  it("skips the check when verifySolidPlugin is disabled", async () => {
    await expect(
      resolveConfigWith({ verifySolidPlugin: false }, ["ox-content:solid-transform"]),
    ).resolves.toBeUndefined();
  });

  it("reports an uncompiled module as a missing extensions entry", () => {
    const verify = findPlugin(oxContentSolid(), "ox-content:solid-verify");
    const errors: string[] = [];
    const context = {
      error(message: string) {
        errors.push(message);
        throw new Error(message);
      },
    };

    const runTransform = (code: string, id: string) =>
      (verify.transform as (this: unknown, code: string, id: string) => unknown).call(
        context,
        code,
        id,
      );

    expect(() =>
      runTransform('<div class="ox-content" innerHTML={rawHtml} />', "/docs/a.md"),
    ).toThrow(/`extensions` option must list the Markdown extensions/);

    // Compiled output and non-Markdown ids pass straight through.
    expect(runTransform("_$template(`<div class=ox-content>`)", "/docs/a.md")).toBeNull();
    expect(runTransform('<div class="ox-content" innerHTML={rawHtml} />', "/src/a.ts")).toBeNull();
  });
});

function pluginNames(plugins: ReturnType<typeof oxContentSolid>): string[] {
  return (plugins as Plugin[]).map((plugin) => plugin.name);
}

function findPlugin(plugins: ReturnType<typeof oxContentSolid>, name: string): Plugin {
  const plugin = (plugins as Plugin[]).find((candidate) => candidate.name === name);
  if (!plugin) throw new Error(`plugin ${name} not found`);
  return plugin;
}

/**
 * Drives `configResolved` on the transform plugin with a plugin list standing in
 * for a resolved Vite config. Only the plugin names matter to the check.
 */
async function resolveConfigWith(
  options: SolidIntegrationOptions,
  pluginNames: string[],
): Promise<void> {
  const transform = findPlugin(oxContentSolid(options), "ox-content:solid-transform");
  const config = {
    root: "/repo",
    plugins: pluginNames.map((name) => ({ name })),
  } as unknown as ResolvedConfig;

  const hook = transform.configResolved as (config: ResolvedConfig) => void | Promise<void>;
  await hook.call(transform, config);
}
