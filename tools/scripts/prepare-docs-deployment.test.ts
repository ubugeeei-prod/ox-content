import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";
import { prepareCloudflareBuildOutput } from "../deploy/build-output.ts";
import { prepareDocsDeployment } from "./prepare-docs-deployment.ts";

function fixture(t: TestContext): string {
  const root = mkdtempSync(join(tmpdir(), "ox-content-deploy-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  for (const directory of [
    "docs/dist/docs/assets",
    "target/doc/ox_content",
    "examples/playground/dist",
  ]) {
    mkdirSync(join(root, directory), { recursive: true });
  }
  for (const [file, content] of [
    ["docs/dist/docs/index.html", "docs"],
    ["docs/dist/docs/404.html", "not found"],
    ["docs/dist/docs/assets/docs.css", "styles"],
    ["target/doc/ox_content/index.html", "Rust API"],
    ["examples/playground/dist/index.html", "playground"],
  ]) {
    writeFileSync(join(root, file), content);
  }
  return root;
}

test("stages docs, custom 404, assets, Rust API, and playground for cf deployment", async (t) => {
  const root = fixture(t);
  prepareDocsDeployment(root);
  await prepareCloudflareBuildOutput(root);
  for (const [file, content] of [
    ["index.html", "docs"],
    ["404.html", "not found"],
    ["assets/docs.css", "styles"],
    ["api/ox_content/index.html", "Rust API"],
    ["playground/index.html", "playground"],
  ]) {
    assert.equal(readFileSync(join(root, "dist", file), "utf8"), content);
    assert.equal(
      readFileSync(
        join(root, "tools/deploy/.cloudflare/output/v0/workers/default/assets", file),
        "utf8",
      ),
      content,
    );
  }
});

test("removes pages left over from a previous build", (t) => {
  const root = fixture(t);
  mkdirSync(join(root, "dist"));
  writeFileSync(join(root, "dist", "stale.html"), "old page");
  prepareDocsDeployment(root);
  assert.equal(existsSync(join(root, "dist", "stale.html")), false);
});

test("preserves the previous bundle if any build output is missing", (t) => {
  const root = fixture(t);
  mkdirSync(join(root, "dist"));
  writeFileSync(join(root, "dist", "index.html"), "previous build");
  rmSync(join(root, "examples/playground/dist"), { recursive: true });
  assert.throws(() => prepareDocsDeployment(root), /ENOENT/);
  assert.equal(readFileSync(join(root, "dist", "index.html"), "utf8"), "previous build");
});
