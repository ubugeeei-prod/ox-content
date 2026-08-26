import { describe, expect, it } from "vite-plus/test";
import type { Plugin } from "vite";
import { oxContentSvelte } from "./index";

describe("oxContentSvelte", () => {
  it("keeps core SSG plugins while replacing the markdown transform", () => {
    const names = pluginNames(
      oxContentSvelte({
        ssg: { siteUrl: "https://example.com" },
        redirects: { map: { "/old": "/new" }, netlify: true },
      }),
    );

    expect(names).toContain("ox-content:svelte-transform");
    expect(names).toContain("ox-content:svelte-environment");
    expect(names).toContain("ox-content:ssg");
    expect(names).toContain("ox-content:collections");
    expect(names).toContain("ox-content:search");
    expect(names).toContain("ox-content:docs");
    expect(names).not.toContain("ox-content");
    expect(names).not.toContain("ox-content:environment");
  });
});

function pluginNames(plugins: ReturnType<typeof oxContentSvelte>): string[] {
  return (plugins as Plugin[]).map((plugin) => plugin.name);
}
