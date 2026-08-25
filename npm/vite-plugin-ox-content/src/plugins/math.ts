/**
 * Build-time KaTeX rendering for opt-in `$…$` / `$$…$$` math.
 *
 * KaTeX is an optional peer. Sites that never enable `math` do not install it,
 * and the published plugin does not bundle or depend on it.
 */

import { createRequire } from "node:module";
import { dirname, join } from "node:path";

export const KATEX_ASSET_DIR = "__ox_katex__";

type KatexModule = {
  renderToString(
    tex: string,
    options?: {
      displayMode?: boolean;
      throwOnError?: boolean;
      trust?: boolean;
      output?: "html" | "mathml" | "htmlAndMathml";
    },
  ): string;
};

const MATH_TAG =
  /<(span|div) class="ox-math ox-math-(inline|block)" data-ox-tex="([^"]*)">[\s\S]*?<\/\1>/g;

let missingWarned = false;

/**
 * Replaces rust `ox-math` placeholders with static KaTeX HTML.
 * Leaves the escaped TeX fallback when `katex` is not installed.
 */
export async function renderKatexMath(html: string): Promise<string> {
  if (!html.includes("data-ox-tex")) {
    return html;
  }

  const katex = loadKatex();
  if (!katex) {
    warnMissingKatexOnce();
    return html;
  }

  return html.replace(MATH_TAG, (_match, tag: string, kind: string, encoded: string) => {
    const rendered = katex.renderToString(decodeHtmlAttr(encoded), {
      displayMode: kind === "block",
      throwOnError: false,
      trust: false,
      output: "htmlAndMathml",
    });
    return `<${tag} class="ox-math ox-math-${kind}">${rendered}</${tag}>`;
  });
}

/** Directory that contains `katex.min.css` and `fonts/`, or `null`. */
export function resolveKatexDist(): string | null {
  for (const resolver of createKatexResolvers()) {
    try {
      return join(dirname(resolver.resolve("katex/package.json")), "dist");
    } catch {
      // Try the next resolver.
    }
  }
  return null;
}

export function resetKatexWarningForTests(): void {
  missingWarned = false;
}

function loadKatex(): KatexModule | null {
  for (const resolver of createKatexResolvers()) {
    try {
      const loaded = resolver(resolver.resolve("katex")) as {
        default?: KatexModule;
      } & KatexModule;
      if (typeof loaded.renderToString === "function") {
        return loaded;
      }
      if (loaded.default && typeof loaded.default.renderToString === "function") {
        return loaded.default;
      }
    } catch {
      // Try the next resolver.
    }
  }
  return null;
}

function createKatexResolvers(): NodeJS.Require[] {
  const consumerRequire = createRequire(join(process.cwd(), "noop.js"));
  const resolvers = [consumerRequire];
  try {
    resolvers.push(createRequire(consumerRequire.resolve("@ox-content/vite-plugin")));
  } catch {
    // Source checkouts still resolve from this file and from cwd.
  }
  resolvers.push(createRequire(import.meta.url));
  return resolvers;
}

function decodeHtmlAttr(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

function warnMissingKatexOnce(): void {
  if (missingWarned) {
    return;
  }
  missingWarned = true;
  console.warn(
    "[ox-content] math is enabled but `katex` was not found. " +
      "Install it with `npm i -D katex` to render LaTeX; " +
      "escaped TeX placeholders are left as-is.",
  );
}
