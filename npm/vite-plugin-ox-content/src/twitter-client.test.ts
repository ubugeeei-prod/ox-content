import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

describe("twitter client public API", () => {
  it("declares the client subpath and generated runtime entry", () => {
    const exportsField = packageJson.exports as Record<string, PackageConditionalExport | string>;
    const client = exportsField["./twitter/client"] as PackageConditionalExport;

    expect(client.import.types).toBe("./dist/twitter-client.d.mts");
    expect(client.import.default).toBe("./dist/twitter-client.mjs");
    expect(client.require.types).toBe("./dist/twitter-client.d.cts");
    expect(client.require.default).toBe("./dist/twitter-client.cjs");

    const buildScript = readFileSync(join(packageRoot, "scripts/build-twitter-client.mjs"), "utf8");
    expect(buildScript).toContain("plugins/twitter.js");
    expect(buildScript).toContain("twitter-client.d.mts");
    expect(packageJson.scripts.build).toContain("node scripts/build-twitter-client.mjs");
  });
});

interface PackageConditionalExport {
  import: {
    types: string;
    default: string;
  };
  require: {
    types: string;
    default: string;
  };
}
