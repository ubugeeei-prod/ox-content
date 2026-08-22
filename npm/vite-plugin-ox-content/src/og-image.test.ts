import { describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { rolldown } from "rolldown";
import { isBareSpecifier, tsTemplateBundleOptions } from "./og-image";

describe("isBareSpecifier", () => {
  it("treats package specifiers as resolvable at runtime", () => {
    for (const id of [
      "@ox-content/vite-plugin",
      "@ox-content/vite-plugin/jsx-runtime",
      "react",
      "react-dom/server",
      "lodash-es",
    ]) {
      expect(isBareSpecifier(id), id).toBe(true);
    }
  });

  it("keeps the template's own files in the bundle", () => {
    for (const id of [
      "./card",
      "../shared/card.ts",
      ".",
      "/abs/card.ts",
      "C:\\project\\card.ts",
      "C:/project/card.ts",
      "\0virtual:module",
    ]) {
      expect(isBareSpecifier(id), id).toBe(false);
    }
  });
});

describe("tsTemplateBundleOptions", () => {
  it("leaves a resolvable package import external instead of inlining it", async () => {
    // Regression: the `.ts` bundle had no `external` at all, so importing the
    // plugin's own helpers pulled the whole plugin — chokidar, fsevents and
    // all — into the template bundle and the build failed outright.
    //
    // The fixture ships its own `node_modules` so the specifier really does
    // resolve. Pointing at a package the fixture cannot see would leave the
    // import external either way and prove nothing.
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-og-template-"));
    try {
      const pkg = path.join(dir, "node_modules", "fake-runtime");
      await fs.mkdir(pkg, { recursive: true });
      await fs.writeFile(
        path.join(pkg, "package.json"),
        JSON.stringify({ name: "fake-runtime", version: "1.0.0", main: "index.js" }),
      );
      await fs.writeFile(
        path.join(pkg, "index.js"),
        'export const renderToString = (node) => node.__html + "/*INLINED-RUNTIME*/";\n',
      );
      await fs.writeFile(
        path.join(dir, "card.ts"),
        "export const card = (title: string) => `<h1>${title}</h1>`;\n",
      );
      await fs.writeFile(
        path.join(dir, "template.ts"),
        [
          'import { renderToString } from "fake-runtime";',
          'import { card } from "./card";',
          "export default (props: { title: string }) =>",
          "  renderToString({ __html: card(props.title) });",
          "",
        ].join("\n"),
      );

      const bundle = await rolldown(tsTemplateBundleOptions(path.join(dir, "template.ts")));
      const { output } = await bundle.generate({ format: "esm" });
      await bundle.close();
      const code = output.map((chunk) => ("code" in chunk ? chunk.code : "")).join("\n");

      expect(code).toContain('from "fake-runtime"');
      expect(code).not.toContain("INLINED-RUNTIME");
      // The template's own modules still travel with it.
      expect(code).toContain("<h1>");
    } finally {
      await fs.rm(dir, { recursive: true, force: true });
    }
  });
});
