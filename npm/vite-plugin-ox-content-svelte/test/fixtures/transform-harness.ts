import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { Component } from "svelte";
import type { ResolvedSvelteOptions } from "../../src/types";

export type GeneratedComponent = Component<Record<string, unknown>>;

export type GeneratedModule = {
  default: GeneratedComponent;
  hydrateIslands?: (options?: Record<string, unknown>) => unknown;
};

export function createOptions(
  overrides: Partial<ResolvedSvelteOptions> = {},
): ResolvedSvelteOptions {
  return {
    srcDir: "docs",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    gfm: true,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    codeAnnotations: { enabled: false, metaKey: "annotate" },
    components: { Alert: "./src/components/Alert.svelte" },
    runes: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    mdxDocumentProps: false,
    ...overrides,
  } as ResolvedSvelteOptions;
}

const packageRoot = path.dirname(fileURLToPath(new URL("../../package.json", import.meta.url)));
// Inside the package rather than the OS temp dir: the generated module can
// import `@ox-content/islands`, which only resolves from here.
const scratchRoot = path.join(packageRoot, "node_modules", ".ox-content-test");

/**
 * Replaces the island runtime import with inert locals.
 *
 * These specs assert server output, never hydration, and `@ox-content/islands`
 * is a workspace package whose `dist` is not built when this suite runs.
 */
export function stubIslandRuntime(code: string): string {
  return code.replace(
    /import\s*\{([^}]*)\}\s*from\s*['"]@ox-content\/islands['"];?/g,
    (_match, names: string) =>
      names
        .split(",")
        .map((name) => name.trim())
        .filter(Boolean)
        .map((name) => `const ${name} = () => {};`)
        .join("\n"),
  );
}

/** Writes the compiled module to disk and imports it, so SSR output is real. */
export async function withGeneratedModule(
  code: string,
  callback: (component: GeneratedComponent, module: GeneratedModule) => void | Promise<void>,
): Promise<void> {
  await mkdir(scratchRoot, { recursive: true });
  const dir = await mkdtemp(path.join(scratchRoot, "page-"));
  const file = path.join(dir, "page.mjs");
  await writeFile(file, stubIslandRuntime(code), "utf8");
  try {
    const mod = (await import(`${pathToFileURL(file).href}?t=${Date.now()}`)) as GeneratedModule;
    await callback(mod.default, mod);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

export function stripSvelteComments(html: string): string {
  return html.replace(/<!--[\s\S]*?-->/g, "");
}
