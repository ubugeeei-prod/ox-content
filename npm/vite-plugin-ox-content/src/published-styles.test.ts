import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const ssgSrc = join(packageRoot, "../../crates/ox_content_ssg/src");

const DOCUMENTED_STYLE_IMPORTS = [
  "core.css",
  "magic-links.css",
  "social.css",
  "twitter-full.css",
  "ogp.css",
  "github.css",
  "youtube.css",
  "tabs.css",
  "mermaid.css",
  "all.css",
] as const;

const CRATE_SOURCES: Record<
  Exclude<(typeof DOCUMENTED_STYLE_IMPORTS)[number], "all.css">,
  string[]
> = {
  "core.css": ["ssg.css"],
  "magic-links.css": ["plugins/magic-links.css"],
  "social.css": ["plugins/social.css", "plugins/social-twitter-rich.css"],
  "twitter-full.css": ["plugins/social-tweet-full.css", "plugins/social-tweet-full-media.css"],
  "ogp.css": ["plugins/ogp.css"],
  "github.css": ["plugins/github.css"],
  "youtube.css": ["plugins/youtube.css"],
  "tabs.css": ["plugins/tabs.css"],
  "mermaid.css": ["plugins/mermaid.css"],
};

function crateCss(relativePath: string): string {
  return readFileSync(join(ssgSrc, relativePath), "utf8");
}

function packedCss(tarball: string, name: string): string {
  const result = spawnSync("tar", ["-xOf", tarball, `package/dist/styles/${name}`], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || `failed to read package/dist/styles/${name}`);
  }
  return result.stdout;
}

function generateStyles(): void {
  const result = spawnSync("node", ["scripts/copy-component-styles.mjs"], {
    cwd: packageRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || result.stdout || "copy-component-styles failed");
  }
}

function packPlugin(): { tarball: string; cleanup: () => void } {
  const dest = mkdtempSync(join(tmpdir(), "ox-content-style-pack-"));
  const result = spawnSync("pnpm", ["pack", "--json", "--pack-destination", dest], {
    cwd: packageRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0) {
    rmSync(dest, { recursive: true, force: true });
    throw new Error(result.stderr || result.stdout || "pnpm pack failed");
  }
  const jsonStart = result.stdout.search(/[[{]/);
  const packed = JSON.parse(result.stdout.slice(jsonStart)) as { filename?: string };
  if (!packed.filename) {
    rmSync(dest, { recursive: true, force: true });
    throw new Error(`pnpm pack did not emit a filename:\n${result.stdout}`);
  }
  return {
    tarball: packed.filename,
    cleanup() {
      rmSync(dest, { recursive: true, force: true });
    },
  };
}

describe("published component styles", () => {
  it("documents every feature stylesheet in package exports", () => {
    const exportsField = packageJson.exports as Record<string, unknown>;
    for (const name of DOCUMENTED_STYLE_IMPORTS) {
      expect(exportsField[`./styles/${name}`]).toBe(`./dist/styles/${name}`);
    }
  });

  it("packs the documented CSS imports from the crate sources", () => {
    generateStyles();
    const { tarball, cleanup } = packPlugin();
    try {
      for (const [name, sources] of Object.entries(CRATE_SOURCES)) {
        expect(packedCss(tarball, name)).toBe(sources.map(crateCss).join(""));
      }

      const twitterFull = packedCss(tarball, "twitter-full.css");
      expect(twitterFull).toContain("react-tweet");
      expect(twitterFull).toContain("sveltweet");
      expect(twitterFull).toContain("Copyright (c) 2023 Luis Alvarez");
      expect(twitterFull).toContain("Copyright (c) 2024 ryoppippi");
      expect(twitterFull).toContain("Permission is hereby granted");

      const allCss = packedCss(tarball, "all.css");
      for (const name of Object.keys(CRATE_SOURCES)) {
        expect(allCss).toContain(`@import "./${name}";`);
      }
    } finally {
      cleanup();
    }
  });
});
