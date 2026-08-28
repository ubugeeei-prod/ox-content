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

/** What to do with a `$…$` run KaTeX cannot parse. */
export type MathErrorPolicy = "literal" | "error" | "render";

/** One `$…$` run KaTeX refused, for the caller to report. */
export interface MathRenderFailure {
  /** The TeX between the delimiters. */
  tex: string;
  /** Whether it was written as block math. */
  block: boolean;
  /** KaTeX's own message. */
  message: string;
}

/**
 * Replaces rust `ox-math` placeholders with static KaTeX HTML.
 * Leaves the escaped TeX fallback when `katex` is not installed.
 *
 * `onError` decides what a run KaTeX cannot parse becomes. Prose that
 * quotes math syntax is picked up as math by the `$…$` heuristics, and the
 * default — putting the source back the way it was written — keeps that
 * page readable instead of stamping red error text into the middle of a
 * sentence. Failures are collected either way, so the caller can warn.
 */
export async function renderKatexMath(
  html: string,
  onError: MathErrorPolicy = "literal",
  failures?: MathRenderFailure[],
): Promise<string> {
  if (!html.includes("data-ox-tex")) {
    return html;
  }

  const katex = loadKatex();
  if (!katex) {
    warnMissingKatexOnce();
    return html;
  }

  return html.replace(MATH_TAG, (_match, tag: string, kind: string, encoded: string) => {
    const block = kind === "block";
    const tex = decodeHtmlAttr(encoded);
    const options = {
      displayMode: block,
      trust: false,
      output: "htmlAndMathml",
    } as const;

    let rendered: string;
    try {
      rendered = katex.renderToString(tex, { ...options, throwOnError: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      failures?.push({ tex, block, message });
      if (onError === "error") {
        throw new Error(`${message} (in ${block ? "$$" : "$"}${tex}${block ? "$$" : "$"})`);
      }
      if (onError === "literal") {
        // Put the source back exactly as authored, delimiters included, so
        // the sentence reads the way its author wrote it.
        return escapeHtmlText(block ? `$$${tex}$$` : `$${tex}$`);
      }
      rendered = katex.renderToString(tex, { ...options, throwOnError: false });
    }
    return `<${tag} class="ox-math ox-math-${kind}">${rendered}</${tag}>`;
  });
}

function escapeHtmlText(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
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

const NAMED_ENTITIES: Record<string, string> = {
  amp: "&",
  quot: '"',
  apos: "'",
  lt: "<",
  gt: ">",
  nbsp: "\u00a0",
};

/**
 * Decodes the attribute back to the TeX the author wrote.
 *
 * The placeholder is escaped by Rust and re-serialized by the rehype passes
 * that run before this one, and those two do not agree on a spelling — an
 * apostrophe leaves Rust untouched and comes back from rehype as `&#x27;`.
 * A general decoder covers whichever spelling arrives; chained `replaceAll`
 * calls over a fixed list did not, and the leftover `&#x27;` reached KaTeX
 * as five literal characters.
 *
 * One left-to-right pass, so `&amp;lt;` decodes to `&lt;` rather than `<`.
 */
function decodeHtmlAttr(value: string): string {
  return value.replace(
    /&(#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z][a-zA-Z0-9]*);/g,
    (match, body: string) => {
      if (body.startsWith("#")) {
        const code =
          body[1] === "x" || body[1] === "X"
            ? Number.parseInt(body.slice(2), 16)
            : Number.parseInt(body.slice(1), 10);
        return Number.isFinite(code) && code >= 0 && code <= 0x10ffff
          ? String.fromCodePoint(code)
          : match;
      }
      return NAMED_ENTITIES[body.toLowerCase()] ?? match;
    },
  );
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
