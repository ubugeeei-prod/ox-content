import { readFileSync } from "node:fs";
import { describe, expect, it } from "vite-plus/test";
import { importNapiModuleSync } from "./napi";

const media = JSON.parse(
  readFileSync(new URL("../../../tools/benchmarks/extension-media.json", import.meta.url), "utf8"),
);

describe("extension benchmark media fixtures", () => {
  it("covers every native provider exactly once with an active, diagnostic-free fixture", () => {
    const napi = importNapiModuleSync();
    expect(media.fixtures.map((fixture: { name: string }) => fixture.name).sort()).toEqual(
      napi
        .mediaEmbedTags()
        .map((tag) => tag.name)
        .sort(),
    );
    for (const fixture of media.fixtures) {
      const output = napi.transformMediaEmbedsWithDiagnostics(fixture.html, media.options);
      expect(output.diagnostics, fixture.name).toEqual([]);
      expect(output.html, fixture.name).not.toBe(fixture.html);
    }
  });
});
