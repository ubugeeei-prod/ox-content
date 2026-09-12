import { spawnSync } from "node:child_process";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";

const bin = resolve(dirname(fileURLToPath(import.meta.url)), "..", "bin", "oxct.mjs");
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("oxct CLI", () => {
  it("prints the top-level help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct <command>");
    expect(result.stdout).toContain("i18n <command>");
    expect(result.stdout).toContain("validate");
    expect(result.stdout).toContain("link-check");
    expect(result.stdout).toContain("migrate vitepress");
    expect(result.stdout).toContain("mdc-check");
    expect(result.stdout).toContain("lsp");
    expect(result.stdout).toContain("og-preview");
  });

  it("prints i18n help without loading the native binding", () => {
    const result = spawnSync(process.execPath, [bin, "i18n", "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct i18n check");
    expect(result.stdout).toContain("oxct i18n validate");
  });

  it.each([
    ["link-check", "oxct link-check"],
    ["validate", "oxct validate"],
    ["mdc-check", "oxct mdc-check"],
    ["lsp", "oxct lsp"],
    ["migrate", "oxct migrate vitepress"],
    ["og-preview", "oxct og-preview"],
  ])("prints %s help without loading implementation dependencies", (command, expected) => {
    const result = spawnSync(process.execPath, [bin, command, "--help"], { encoding: "utf8" });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain(expected);
  });

  it("prints nested VitePress migration help with the oxct command path", () => {
    const result = spawnSync(process.execPath, [bin, "migrate", "vitepress", "--help"], {
      encoding: "utf8",
    });

    expect(result.status).toBe(0);
    expect(result.stdout).toContain("oxct migrate vitepress [config]");
  });

  it("runs collection validation hooks from a Vite config", async () => {
    const root = await fs.mkdtemp(resolve(os.tmpdir(), "ox-content-validate-cli-"));
    tempDirs.push(root);
    await fs.mkdir(resolve(root, "content/blog"), { recursive: true });
    await fs.mkdir(resolve(root, "content/docs"), { recursive: true });
    await fs.writeFile(
      resolve(root, "content/blog/first.md"),
      "---\ntitle: First\npermalink: /wrong\n---\n# First\n",
    );
    await fs.writeFile(resolve(root, "content/docs/guide.md"), "---\ntitle: Guide\n---\n# Guide\n");

    const entryUrl = pathToFileURL(resolve(packageRoot, "dist/index.mjs")).href;
    await fs.writeFile(
      resolve(root, "vite.config.mjs"),
      `import { oxContent, defineCollections } from ${JSON.stringify(entryUrl)};

export default {
  plugins: [
    oxContent({
      srcDir: "content",
      collections: defineCollections({
        blog: {
          source: "blog/*.md",
          validate({ frontmatter, path }) {
            return frontmatter.permalink === path
              ? undefined
              : \`expected permalink \${path}, found \${frontmatter.permalink ?? "(missing)"}\`;
          },
        },
        docs: "docs/*.md",
      }),
    }),
  ],
};
`,
    );

    const failed = spawnSync(process.execPath, [bin, "validate", "--config", "vite.config.mjs"], {
      cwd: root,
      encoding: "utf8",
    });
    expect(failed.status).toBe(1);
    expect(failed.stderr).toContain("[ox-content] Collection validation failed with 1 diagnostic.");
    expect(failed.stderr).toContain(
      "- blog: blog/first.md (/blog/first): expected permalink /blog/first, found /wrong",
    );

    const selected = spawnSync(process.execPath, [bin, "validate", "--collection", "docs"], {
      cwd: root,
      encoding: "utf8",
    });
    expect(selected.status).toBe(0);
    expect(selected.stdout).toContain(
      "[ox-content] Collection validation passed for 1 document in 1 collection.",
    );
  });
});
