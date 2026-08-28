import { importNapiModuleSync } from "./napi";
import type { ReaderChromeOptions, ResolvedReaderChrome } from "./reader-chrome-options";

export type { ReaderChromeOptions, ResolvedReaderChrome };

type EnabledReaderChrome = Exclude<ResolvedReaderChrome, false>;

export type ReaderChromeInput = boolean | ReaderChromeOptions | EnabledReaderChrome | undefined;

const EMPTY_ATTRIBUTES = Object.freeze({}) as Readonly<Record<string, "">>;

/**
 * Resolve the public reader-chrome option shape.
 *
 * `true` mirrors `ssg.readerChrome: true`: copy, outbound-link decoration, and
 * back-to-top are enabled. Object fields default on, so `{ copy: false }`
 * disables only the copy button.
 */
export function resolveReaderChromeInput(input: ReaderChromeInput = true): ResolvedReaderChrome {
  if (!input) {
    return false;
  }
  if (input === true) {
    return { copy: true, externalLinks: true, backToTop: true };
  }
  return {
    copy: input.copy ?? true,
    externalLinks: input.externalLinks ?? true,
    backToTop: input.backToTop ?? true,
  };
}

/**
 * Apply the same native reader-chrome HTML transform used by the built-in SSG.
 *
 * This rewrites rendered article HTML. It wraps fenced `<pre>` blocks for copy
 * controls and decorates outbound links when enabled. Back-to-top remains a
 * document-shell control, so it is exposed through the asset helpers.
 */
export function applyReaderChromeHtml(html: string, input: ReaderChromeInput = true): string {
  const chrome = resolveReaderChromeInput(input);
  if (!chrome || (!chrome.copy && !chrome.externalLinks)) {
    return html;
  }
  return importNapiModuleSync().applySsgReaderChromeHtml(html, {
    copy: chrome.copy,
    externalLinks: chrome.externalLinks,
    backToTop: chrome.backToTop,
  });
}

/**
 * Root attributes consumed by the browser runtime.
 */
export function readerChromeAttributes(
  input: ReaderChromeInput = true,
): Readonly<Record<string, "">> {
  const chrome = resolveReaderChromeInput(input);
  if (!readerChromeIsEnabled(chrome)) {
    return EMPTY_ATTRIBUTES;
  }

  return {
    "data-ox-reader-chrome": "",
    ...(chrome.copy ? { "data-ox-copy": "" as const } : {}),
    ...(chrome.externalLinks ? { "data-ox-external-links": "" as const } : {}),
    ...(chrome.backToTop ? { "data-ox-back-to-top": "" as const } : {}),
  };
}

/**
 * Render the root attributes as a leading-space HTML fragment.
 *
 * Useful in template strings: `<article${renderReaderChromeAttributes()}>`.
 */
export function renderReaderChromeAttributes(input: ReaderChromeInput = true): string {
  return Object.keys(readerChromeAttributes(input))
    .map((name) => ` ${name}`)
    .join("");
}

/**
 * CSS shared by the built-in SSG and custom hosts.
 */
export function readerChromeCss(input: ReaderChromeInput = true): string {
  return readerChromeIsEnabled(resolveReaderChromeInput(input))
    ? importNapiModuleSync().getSsgReaderChromeCss()
    : "";
}

/**
 * Auto-initializing script shared by the built-in SSG and custom hosts.
 */
export function readerChromeScript(input: ReaderChromeInput = true): string {
  return readerChromeNeedsJs(resolveReaderChromeInput(input))
    ? importNapiModuleSync().getSsgReaderChromeScript()
    : "";
}

/**
 * Inline stylesheet tag for hosts that do not import
 * `@ox-content/vite-plugin/styles/reader-chrome.css`.
 */
export function renderReaderChromeStyleTag(input: ReaderChromeInput = true): string {
  const css = readerChromeCss(input);
  return css ? `<style data-ox-style="reader-chrome">\n${css}</style>` : "";
}

/**
 * Inline auto-init script for static hosts that do not bundle
 * `@ox-content/vite-plugin/reader-chrome/client`.
 */
export function renderReaderChromeScriptTag(input: ReaderChromeInput = true): string {
  const js = readerChromeScript(input);
  return js ? `<script data-ox-script="reader-chrome">\n${js}</script>` : "";
}

export function readerChromeIsEnabled(
  chrome: ResolvedReaderChrome,
): chrome is Exclude<ResolvedReaderChrome, false> {
  return Boolean(chrome && (chrome.copy || chrome.externalLinks || chrome.backToTop));
}

export function readerChromeNeedsJs(chrome: ResolvedReaderChrome): boolean {
  return Boolean(chrome && (chrome.copy || chrome.backToTop));
}
