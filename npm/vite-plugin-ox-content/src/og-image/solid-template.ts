import { mkdir } from "node:fs/promises";
import * as path from "node:path";
import type { OgImageTemplateFn } from "./types";

/** Compile Solid JSX for the server; it cannot use React's JSX transform. */
export async function resolveSolidTemplate(
  templatePath: string,
  root: string,
): Promise<OgImageTemplateFn> {
  const [{ rolldown }, compiler, { renderToString }, { createComponent }] = await Promise.all([
    import("rolldown"),
    import("@solidjs/compiler"),
    import("@solidjs/web"),
    import("solid-js"),
  ]);
  const outfile = path.join(root, ".cache", "og-images", "_template_solid.mjs");
  await mkdir(path.dirname(outfile), { recursive: true });
  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: (id) => /^(solid-js|@solidjs\/web|@ox-content\/vite-plugin)(\/|$)/.test(id),
    plugins: [
      {
        name: "ox-content-og-solid",
        async transform(code, id) {
          if (!/\.[jt]sx$/.test(id)) return null;
          const result = await compiler.transformAsync(code, {
            filename: id,
            generate: "ssr",
            hydratable: false,
          });
          return { code: result.code, moduleType: "ts" };
        },
      },
    ],
  });
  try {
    await bundle.write({ file: outfile, format: "esm" });
  } finally {
    await bundle.close();
  }
  const { default: Component } = await import(`${outfile}?t=${Date.now()}`);
  return (props) => renderToString(() => createComponent(Component, props));
}
