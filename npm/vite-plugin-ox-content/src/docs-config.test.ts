import * as fs from "node:fs/promises";
import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";

describe("official docs config", () => {
  it("enables authored containers for published component examples", async () => {
    const configRoot = path.join(import.meta.dirname, "../../../docs");
    const [config, embeds] = await Promise.all([
      fs.readFile(path.join(configRoot, "vite.config.ts"), "utf8"),
      fs.readFile(path.join(configRoot, "embed-providers.config.ts"), "utf8"),
    ]);

    expect(config).toMatch(/containers:\s*true/);
    expect(config).toMatch(/embeds:\s*docsEmbedProviders\(base\)/);
    expect(embeds).toMatch(/packageRegistry:\s*\{\s*fetch:\s*false\s*\}/);
    expect(embeds).toMatch(/playgrounds:\s*\{\s*fetch:\s*false\s*\}/);
    expect(embeds).toMatch(/googleMaps:\s*true/);
  });
});
