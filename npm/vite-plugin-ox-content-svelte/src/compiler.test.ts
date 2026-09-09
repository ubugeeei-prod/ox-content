import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSvelte } from "./transform";
import type { SvelteCompileFunction } from "./types";
import { createOptions } from "./test/fixtures/transform-harness";

describe("Svelte compiler selection", () => {
  it("forwards generated modules through the configured compiler", async () => {
    const map = { mappings: "", sources: ["alt.md"], version: 3 };
    const warnings = [{ code: "fixture-warning", message: "fixture warning" }];
    const calls: Array<{ source: string; generate: string; filename: string; runes: boolean }> = [];
    const compiler: SvelteCompileFunction = async (source, options) => {
      calls.push({
        source,
        filename: options.filename,
        generate: options.generate,
        runes: options.runes,
      });
      return {
        js: { code: "export default function Page() {}", map },
        warnings,
      };
    };

    const result = await transformMarkdownWithSvelte(
      "# Alternative compiler",
      "/repo/docs/alt.md",
      createOptions({ compiler, runes: false }),
    );

    expect(calls).toHaveLength(1);
    expect(calls[0]).toMatchObject({
      filename: "/repo/docs/alt.md",
      generate: "client",
      runes: false,
    });
    expect(calls[0]?.source).toContain("<h1");
    expect(result.map).toBe(map);
    expect(result.warnings).toBe(warnings);
    expect(result.code).toContain("export const frontmatter = {};");
  });

  it("compiles Ox Content-generated MDX modules with rsvelte for client and server", async () => {
    const { compile: compileWithRsvelte } =
      (await import("@rsvelte/vite-plugin-svelte-native")) as { compile: SvelteCompileFunction };
    const calls: Array<{ source: string; generate: string; filename: string; runes: boolean }> = [];
    const compiler: SvelteCompileFunction = (source, options) => {
      calls.push({
        source,
        filename: options.filename,
        generate: options.generate,
        runes: options.runes,
      });
      return compileWithRsvelte(source, options);
    };

    const source = "import Counter from './Counter.svelte'\n\n<Counter initial={1} />\n";
    const client = await transformMarkdownWithSvelte(
      source,
      "/repo/docs/page.mdx",
      createOptions({ components: {}, compiler }),
    );
    const server = await transformMarkdownWithSvelte(
      source,
      "/repo/docs/page.mdx",
      createOptions({ components: {}, compiler, ssr: true }),
    );

    expect(calls.map((call) => call.generate)).toEqual(["client", "server"]);
    expect(calls.every((call) => call.filename === "/repo/docs/page.mdx")).toBe(true);
    expect(calls.every((call) => call.runes)).toBe(true);
    expect(calls.every((call) => call.source.includes("Counter from './Counter.svelte'"))).toBe(
      true,
    );
    expect(client.code).toContain("from 'svelte/internal/client'");
    expect(server.code).toContain("from 'svelte/internal/server'");
  });
});
