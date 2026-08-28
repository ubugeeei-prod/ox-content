import { rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const distDir = join(packageRoot, "dist");

const declarations = `/**
 * Per-control flags for \`ssg.readerChrome\`.
 *
 * Omitted fields stay on when the feature itself is enabled.
 */
export interface ReaderChromeOptions {
  /**
   * Copy button on fenced code blocks. The clipboard is read in the browser,
   * never at build time.
   *
   * @default true
   */
  copy?: boolean;

  /**
   * Icon and \`rel="noopener noreferrer"\` on outbound \`http(s)\` links.
   * Relative, hash, and same-document links are left alone.
   *
   * @default true
   */
  externalLinks?: boolean;

  /**
   * Back-to-top control that appears after the page is scrolled.
   *
   * @default true
   */
  backToTop?: boolean;
}

/**
 * Resolved reader chrome. \`false\` means no extra markup or JS.
 */
export type ResolvedReaderChrome =
  | false
  | {
      copy: boolean;
      externalLinks: boolean;
      backToTop: boolean;
    };

type EnabledReaderChrome = Exclude<ResolvedReaderChrome, false>;

export type ReaderChromeInput =
  | boolean
  | ReaderChromeOptions
  | EnabledReaderChrome
  | undefined;

/**
 * Resolve the public reader-chrome option shape.
 */
export declare function resolveReaderChromeInput(
  input?: ReaderChromeInput,
): ResolvedReaderChrome;

/**
 * Apply the same native reader-chrome HTML transform used by the built-in SSG.
 */
export declare function applyReaderChromeHtml(
  html: string,
  input?: ReaderChromeInput,
): string;

/**
 * Root attributes consumed by the browser runtime.
 */
export declare function readerChromeAttributes(
  input?: ReaderChromeInput,
): Readonly<Record<string, "">>;

/**
 * Render the root attributes as a leading-space HTML fragment.
 */
export declare function renderReaderChromeAttributes(input?: ReaderChromeInput): string;

/**
 * CSS shared by the built-in SSG and custom hosts.
 */
export declare function readerChromeCss(input?: ReaderChromeInput): string;

/**
 * Auto-initializing script shared by the built-in SSG and custom hosts.
 */
export declare function readerChromeScript(input?: ReaderChromeInput): string;

/**
 * Inline stylesheet tag for hosts that do not import
 * \`@ox-content/vite-plugin/styles/reader-chrome.css\`.
 */
export declare function renderReaderChromeStyleTag(input?: ReaderChromeInput): string;

/**
 * Inline auto-init script for static hosts that do not bundle
 * \`@ox-content/vite-plugin/reader-chrome/client\`.
 */
export declare function renderReaderChromeScriptTag(input?: ReaderChromeInput): string;

export declare function readerChromeIsEnabled(
  chrome: ResolvedReaderChrome,
): chrome is Exclude<ResolvedReaderChrome, false>;

export declare function readerChromeNeedsJs(chrome: ResolvedReaderChrome): boolean;
`;

await Promise.all([
  writeFile(join(distDir, "reader-chrome.d.mts"), declarations),
  writeFile(join(distDir, "reader-chrome.d.cts"), declarations),
  rm(join(distDir, "reader-chrome.d.mts.map"), { force: true }),
  rm(join(distDir, "reader-chrome.d.cts.map"), { force: true }),
  rm(join(distDir, "reader-chrome2.d.mts"), { force: true }),
  rm(join(distDir, "reader-chrome2.d.cts"), { force: true }),
  rm(join(distDir, "reader-chrome2.d.mts.map"), { force: true }),
  rm(join(distDir, "reader-chrome2.d.cts.map"), { force: true }),
]);
