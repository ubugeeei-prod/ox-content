import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { join } from "node:path";
import { compileHonoDocument } from "./document.js";
import { oxContentHono } from "./vite.js";

const options = {
  highlight: false,
  resources: false,
  docs: false,
  search: false,
  feeds: false,
  siteMaps: false,
};
test("Markdown component compilation preserves YAML, named exports and safe runtime boundaries", async () => {
  const documentPath = join(process.cwd(), "test/component.mdx");
  const result = await compileHonoDocument({
    source:
      '---\ntags:\n  - one\nnested:\n  value: two\n---\n\nimport {Card} from "./Card.tsx";\n\n<Card title="hello" client:visible>**slot**</Card>',
    documentPath,
    root: process.cwd(),
    oxContent: options,
  });
  assert.deepEqual(result.parsed.frontmatter, {
    tags: ["one"],
    nested: { value: "two" },
  });
  assert.match(result.code, /\["Card"\]/);
  assert.match(result.code, /client:visible/);
  assert.match(result.code, /data-ox-module/);
  assert.match(result.code, /data-ox-export/);
  assert.match(result.code, /@ox-content\/vite-plugin-hono\/server/);
  assert.doesNotMatch(result.code, /@ox-content\/vite-plugin(?:\/|['"])|@ox-content\/napi|node:/);
});
test("unregistered islands fail compilation rather than silently dropping components", async () => {
  await assert.rejects(
    compileHonoDocument({
      source: "<Missing />",
      documentPath: "/app/missing.mdx",
      root: "/app",
      oxContent: options,
    }),
    /not registered/,
  );
});
test("automatic entry preserves existing string, array and named inputs; routes false leaves app configuration alone", () => {
  for (const input of ["src/app.ts", ["src/app.ts", "src/admin.ts"], { app: "src/app.ts" }]) {
    const plugin = oxContentHono()[0];
    const config = plugin.config({ build: { rollupOptions: { input } } });
    const result = config.build.rollupOptions.input;
    assert.match(JSON.stringify(result), /src\/app.ts/);
    assert.match(JSON.stringify(result), /virtual:ox-content-hono\/client/);
  }
  const collision = oxContentHono()[0].config({
    build: {
      rollupOptions: { input: { "ox-content-hono": "src/existing.ts" } },
    },
  }).build.rollupOptions.input;
  assert.equal(collision["ox-content-hono"], "src/existing.ts");
  assert.equal(collision["ox-content-hono_"], "virtual:ox-content-hono/client");
  assert.equal(oxContentHono({ routes: false })[0].config({}), undefined);
});
