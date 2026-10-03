import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, readFile, readdir, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createProject } from "../../npm/vite-plugin-ox-content/bin/oxct-new.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const dirs = [];
afterEach(async () =>
  Promise.all(dirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true }))),
);
describe("generated Ox Content projects", () => {
  for (const manager of ["vp", "npm"]) {
    it.each(["docs", "blog", "minimal"])(
      `builds the %s template with ${manager}`,
      async (template) => {
        const root = await mkdtemp(join(tmpdir(), "oxct-build-"));
        dirs.push(root);
        await createProject(root, { template, manager, skin: "editorial", palette: "nord" });
        // Use this exact PR's built packages, rather than an older registry release.
        const links = {
          "@ox-content/vite-plugin": "npm/vite-plugin-ox-content",
          "@ox-content/theme-editorial": "npm/theme/editorial",
          "@ox-content/theme-color-nord": "npm/theme-color/nord",
          "vite-plus": "node_modules/vite-plus",
          vite: "npm/vite-plugin-ox-content/node_modules/vite",
          typescript: "node_modules/typescript",
        };
        for (const [name, source] of Object.entries(links)) {
          const target = join(root, "node_modules", name);
          await mkdir(dirname(target), { recursive: true });
          await symlink(
            join(repository, source),
            target,
            process.platform === "win32" ? "junction" : "dir",
          );
        }
        const result = spawnSync(
          process.execPath,
          ["--input-type=module", "-e", "const { build } = await import('vite'); await build();"],
          {
            cwd: root,
            encoding: "utf8",
            timeout: 30000,
          },
        );
        expect(result.status, result.stderr + result.stdout).toBe(0);
        const docs = join(root, "dist");
        expect(await readFile(join(docs, "index.html"), "utf8")).toContain(
          "Your content, beautifully rendered",
        );
        expect((await readdir(docs)).length).toBeGreaterThan(0);
        const tsc = join(root, "node_modules/typescript/bin/tsc");
        const check = spawnSync(process.execPath, [tsc, "--noEmit"], {
          cwd: root,
          encoding: "utf8",
          timeout: 30000,
        });
        expect(check.status, check.stderr + check.stdout).toBe(0);
      },
      60000,
    );
  }
});
