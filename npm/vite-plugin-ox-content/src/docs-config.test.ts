import * as fs from "node:fs/promises";
import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";

describe("official docs config", () => {
  it("enables authored containers for published component examples", async () => {
    const config = await fs.readFile(
      path.join(import.meta.dirname, "../../../docs/vite.config.ts"),
      "utf8",
    );

    expect(config).toMatch(/containers:\s*true/);
    expect(config).toMatch(/packageRegistry:\s*\{\s*fetch:\s*false\s*\}/);
    expect(config).toMatch(/playgrounds:\s*\{\s*fetch:\s*false\s*\}/);
    expect(config).toMatch(/googleMaps:\s*true/);
  });
});
