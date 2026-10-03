import { spawnSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { parse } from "jsonc-parser";
import { planJsonEdit, writePlan } from "../bin/oxct-config-files.mjs";

const bin = fileURLToPath(new URL("../bin/oxct.mjs", import.meta.url));
const wrapper = fileURLToPath(new URL("../../oxct/bin/oxct.mjs", import.meta.url));
const dirs = [];
async function fixture() {
  const dir = await mkdtemp(join(tmpdir(), "oxct-setup-"));
  dirs.push(dir);
  return dir;
}
const run = (args, cwd, env = {}) =>
  spawnSync(process.execPath, [bin, ...args], {
    cwd,
    env: { ...process.env, ...env },
    encoding: "utf8",
  });
afterEach(async () =>
  Promise.all(dirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true }))),
);

describe("oxct project and IDE setup", () => {
  it("exposes the standalone package binary without a project or native binding", async () => {
    const result = spawnSync(process.execPath, [wrapper, "--help"], {
      cwd: await fixture(),
      encoding: "utf8",
    });
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("ide install");
    expect(result.stdout).toContain("new [directory]");
  });

  it.each(["docs", "blog", "minimal"])(
    "creates a complete %s project and refuses overwrites",
    async (template) => {
      const cwd = await fixture();
      const result = run(["new", "site", "--yes", "--no-install", "--template", template], cwd);
      expect(result.status, result.stderr).toBe(0);
      const root = join(cwd, "site");
      const pkg = JSON.parse(await readFile(join(root, "package.json"), "utf8"));
      expect(pkg.scripts.build).toBe("vp build");
      expect(pkg.devDependencies["@ox-content/theme-editorial"]).toBe(
        pkg.devDependencies["@ox-content/vite-plugin"],
      );
      expect(await readFile(join(root, "content/index.md"), "utf8")).toContain('title: "site"');
      if (template === "blog")
        expect(await readFile(join(root, "vite.config.ts"), "utf8")).toContain("defineCollections");
      const again = run(["new", "site", "--yes"], cwd);
      expect(again.status).toBe(1);
      expect(again.stderr).toContain("Directory is not empty");
      expect(JSON.parse(await readFile(join(root, "package.json"), "utf8"))).toEqual(pkg);
    },
  );

  it("merges JSONC comments, recommendations, associations and file types idempotently", async () => {
    const cwd = await fixture();
    await mkdir(join(cwd, ".vscode"));
    await mkdir(join(cwd, ".zed"));
    await writeFile(
      join(cwd, ".vscode/settings.json"),
      '{\n // Keep this comment\n "editor.tabSize": 4,\n "files.associations": {"*.foo": "text"},\n}\n',
      { mode: 0o600 },
    );
    await writeFile(
      join(cwd, ".vscode/extensions.json"),
      '{"recommendations":["other.extension"]}\n',
    );
    await writeFile(join(cwd, ".zed/settings.json"), '{"file_types":{"Markdown":["custommd"]}}\n');
    const args = ["ide", "install", "--ide", "vscode", "--ide", "zed", "--config-only", "--yes"];
    expect(run(args, cwd).status).toBe(0);
    const settings = await readFile(join(cwd, ".vscode/settings.json"), "utf8");
    expect(settings).toContain("Keep this comment");
    expect(parse(settings)["files.associations"]).toEqual({ "*.foo": "text", "*.mdc": "markdown" });
    expect(parse(settings)["editor.tabSize"]).toBe(4);
    if (process.platform !== "win32")
      expect((await stat(join(cwd, ".vscode/settings.json"))).mode & 0o777).toBe(0o600);
    expect(
      parse(await readFile(join(cwd, ".vscode/extensions.json"), "utf8")).recommendations,
    ).toEqual(["other.extension", "ubugeeei.vscode-ox-content"]);
    expect(
      parse(await readFile(join(cwd, ".zed/settings.json"), "utf8")).file_types.Markdown,
    ).toContain("custommd");
    const before = await readdir(join(cwd, ".vscode"));
    expect(before.some((file) => file.endsWith(".bak"))).toBe(true);
    expect(run(args, cwd).status).toBe(0);
    expect(await readdir(join(cwd, ".vscode"))).toEqual(before);
  });

  it("keeps dry runs free of writes, including user-level extension settings", async () => {
    const cwd = await fixture();
    const result = run(
      ["ide", "install", "--ide", "zed", "--ide", "neovim", "--dry-run", "--yes"],
      cwd,
      {
        XDG_CONFIG_HOME: join(cwd, "config"),
        XDG_DATA_HOME: join(cwd, "data"),
      },
    );
    expect(result.status).toBe(0);
    expect(result.stdout).toContain(".zed/settings.json");
    expect(await readdir(cwd)).toEqual([]);
  });

  it("does not replace malformed or concurrently changed configuration", async () => {
    const cwd = await fixture();
    const file = join(cwd, "settings.json");
    await writeFile(file, '{ "broken": ');
    await expect(planJsonEdit(file, [[["enabled"], true]])).rejects.toThrow(
      "Invalid configuration",
    );
    await writeFile(file, "{}\n");
    const plan = await planJsonEdit(file, [[["enabled"], true]]);
    await writeFile(file, '{"concurrent":true}\n');
    await expect(writePlan(plan)).rejects.toThrow("changed during setup");
    expect(await readFile(file, "utf8")).toContain("concurrent");
  });

  it.each([
    ["new", "--skin", "missing"],
    ["ide", "install", "--ide", "missing"],
    ["new", "--template"],
  ])("rejects invalid choices before writing: %j", async (...args) => {
    const cwd = await fixture();
    expect(run(args, cwd).status).toBe(1);
    expect(await readdir(cwd)).toEqual([]);
  });
});
