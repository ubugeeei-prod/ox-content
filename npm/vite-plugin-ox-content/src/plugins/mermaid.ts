/**
 * Mermaid Plugin - Native Rust renderer via NAPI
 *
 * Renders mermaid code blocks to SVG using the native Rust renderer
 * via NAPI. Delegates to the NAPI `transformMermaid` function which
 * extracts mermaid code blocks from HTML and renders them using mmdc.
 */

import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { importNapiModule } from "../napi";

export interface MermaidOptions {
  /** Mermaid theme. Default: "neutral" */
  theme?: "default" | "dark" | "forest" | "neutral" | "base";
}

/** Cached NAPI bindings */
let napiBindings: {
  transformMermaid: (html: string, mmdcPath: string) => { html: string; errors: string[] };
} | null = null;

let napiLoadAttempted = false;

async function loadNapi() {
  if (napiLoadAttempted) return napiBindings;
  napiLoadAttempted = true;
  try {
    const binding = (await importNapiModule()) as unknown as NonNullable<typeof napiBindings>;
    if (typeof binding.transformMermaid !== "function") {
      napiBindings = null;
      return null;
    }
    napiBindings = binding;
    return binding;
  } catch {
    napiBindings = null;
    return null;
  }
}

let cachedMmdcPath: string | null | undefined;
let missingMmdcWarned = false;

function resolveMmdcPath(): string | null {
  if (cachedMmdcPath !== undefined) return cachedMmdcPath;

  for (const resolver of createMmdcResolvers()) {
    try {
      const entry = resolver.resolve("@mermaid-js/mermaid-cli");
      const cliPath = join(dirname(entry), "cli.js");
      if (existsSync(cliPath)) {
        cachedMmdcPath = cliPath;
        return cachedMmdcPath;
      }
    } catch {
      // Try the next resolver.
    }
  }

  // Fallback: node_modules/.bin/mmdc relative to cwd
  const binPath = join(process.cwd(), "node_modules", ".bin", "mmdc");
  if (existsSync(binPath)) {
    cachedMmdcPath = binPath;
    return cachedMmdcPath;
  }

  cachedMmdcPath = null;
  return null;
}

function createMmdcResolvers(): NodeJS.Require[] {
  // Resolve from the consumer first, then from this package. The second lookup
  // matters under pnpm strict linking: docs apps can depend on the plugin
  // without directly depending on mermaid-cli.
  const consumerRequire = createRequire(join(process.cwd(), "noop.js"));
  const resolvers = [consumerRequire];

  try {
    resolvers.push(createRequire(consumerRequire.resolve("@ox-content/vite-plugin")));
  } catch {
    // If the package is used from source without its package name resolvable,
    // the consumer resolver and bin fallback still cover direct installs.
  }

  return resolvers;
}

/**
 * Transforms mermaid code blocks in HTML to rendered SVG diagrams.
 * Uses the native Rust NAPI transformMermaid function.
 */
export async function transformMermaidStatic(
  html: string,
  _options?: MermaidOptions,
): Promise<string> {
  const napi = await loadNapi();
  if (!napi) {
    return html;
  }

  const mmdcPath = resolveMmdcPath();
  if (!mmdcPath) {
    warnMissingMmdcOnce();
    return html;
  }

  try {
    const result = napi.transformMermaid(html, mmdcPath);
    for (const error of result.errors) {
      console.warn("[ox-content] Mermaid render error:", error);
    }
    return result.html;
  } catch (err) {
    console.warn("[ox-content] Mermaid transform error:", err);
    return html;
  }
}

function warnMissingMmdcOnce(): void {
  if (missingMmdcWarned) {
    return;
  }

  missingMmdcWarned = true;
  console.warn("[ox-content] mmdc not found; skipping Mermaid rendering.");
}

/**
 * @deprecated No longer used. Mermaid rendering is now done at build time via NAPI.
 */
export const mermaidClientScript = "";
