import * as path from "node:path";
import { createAssetsContext } from "./custom-host-assets";
import { createCustomHostMarkdownRenderer } from "./custom-host-markdown";
import type { OxContentCustomHostMarkdownRenderer } from "./custom-host-markdown";
import { customHostOxContentOptions } from "./custom-host-options";
import type { OxContentCustomHostBaseContext } from "./custom-host-types";
import { resolveOptions } from "./resolve-options";
import type { OxContentOptions } from "./types";

export interface CreateOxContentMarkdownRendererInput {
  /** Project root for document paths and relative options. Defaults to process.cwd(). */
  root?: string;
  /** Callback mode. Defaults to build. This does not start a Vite server. */
  mode?: OxContentCustomHostBaseContext["mode"];
  oxContent?: OxContentOptions;
  /** Framework module loader, when a renderHtml callback needs one. */
  loadModule?: OxContentCustomHostBaseContext["loadModule"];
  /** Supply host asset helpers when a callback needs a client manifest or SSR styles. */
  assets?: OxContentCustomHostBaseContext["assets"];
}

/** Run the Custom Host Markdown pipeline in a worker or other programmatic host. */
export function createOxContentMarkdownRenderer(
  input: CreateOxContentMarkdownRendererInput = {},
): OxContentCustomHostMarkdownRenderer {
  const root = path.resolve(input.root ?? process.cwd());
  const options = resolveOptions(customHostOxContentOptions(input.oxContent));
  const outDir = path.resolve(root, options.outDir);
  return createCustomHostMarkdownRenderer({
    mode: input.mode ?? "build",
    root,
    outDir,
    base: options.base,
    options,
    loadModule:
      input.loadModule ??
      (async (moduleId) => {
        throw new Error(
          `[ox-content] Cannot load ${JSON.stringify(moduleId)} without a module loader. ` +
            "Pass loadModule to createOxContentMarkdownRenderer().",
        );
      }),
    assets:
      input.assets ?? createAssetsContext(options, outDir, undefined, undefined, undefined, root),
  });
}
