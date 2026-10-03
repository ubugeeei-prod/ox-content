import { spawnSync } from "node:child_process";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { frontmatterFixture } from "./oxct-frontmatter-fixture.mjs";

const bin = fileURLToPath(new URL("../bin/oxct.mjs", import.meta.url));
const dirs = [];
afterEach(async () =>
  Promise.all(dirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true }))),
);

describe("frontmatter typecheck CLI", () => {
  it("runs async project validators and prints uncontaminated JSON with source locations", async () => {
    const fixture = await frontmatterFixture();
    dirs.push(fixture.root);
    const result = spawnSync(process.execPath, [bin, "typecheck", "--format", "json"], {
      cwd: fixture.root,
      encoding: "utf8",
    });
    expect(result.status, result.stderr).toBe(1);
    expect(result.stderr).toContain("project log 日本語");
    expect(JSON.parse(result.stdout)).toMatchObject({
      checked: 1,
      diagnostics: [
        { file: fixture.file, line: 2, column: 8, message: "Title must be an accepted string" },
      ],
    });
    await writeFile(fixture.file, "---\ntitle: good\n---\n");
    await mkdir(join(fixture.root, "content/other"));
    await writeFile(join(fixture.root, "content/other/b.md"), "---\ntitle: 42\n---\n");
    const valid = spawnSync(process.execPath, [bin, "typecheck", "content/**/*.md"], {
      cwd: fixture.root,
      encoding: "utf8",
    });
    expect(valid.status, valid.stderr).toBe(0);
    expect(valid.stdout).toContain("passed for 1 document.");
    const empty = spawnSync(process.execPath, [bin, "typecheck", "missing/*.md"], {
      cwd: fixture.root,
      encoding: "utf8",
    });
    expect(empty.status).toBe(1);
    expect(empty.stderr).toContain("No documents matched");
  });

  it("prints help without loading project configuration", () => {
    const result = spawnSync(process.execPath, [bin, "typecheck", "--help"], { encoding: "utf8" });
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("Standard Schema");
  });
});
