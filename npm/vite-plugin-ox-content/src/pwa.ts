/**
 * Opt-in web app manifest and conservative service worker.
 *
 * The Vite plugin writes those files during SSG without adding a NAPI surface.
 * Enabling `offline` injects a tiny client script that registers `sw.js`.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { PwaOptions, ResolvedPwaOptions } from "./types";

const MISSING_SITE_URL =
  "[ox-content] pwa is enabled but ssg.siteUrl is not set; manifest.webmanifest and sw.js were not written";

const DEFAULT_THEME_COLOR = "#000000";
const DEFAULT_BACKGROUND_COLOR = "#ffffff";
const MANIFEST_NAME = "manifest.webmanifest";
const SERVICE_WORKER_NAME = "sw.js";

/** Inputs for rendering PWA file bodies. */
export interface PwaRenderInput {
  options?: ResolvedPwaOptions | null;
  siteUrl?: string;
  siteName?: string;
  base?: string;
}

/** Rendered PWA bodies, or a skip warning. */
export interface PwaRenderResult {
  manifest?: string;
  serviceWorker?: string;
  warning?: string;
}

/** Inputs for writing PWA files next to generated HTML. */
export interface WritePwaFilesInput extends PwaRenderInput {
  outDir: string;
  base: string;
}

/**
 * Resolves `pwa` with defaults.
 *
 * `false` / omitted stays off. `true` enables the manifest and offline
 * service worker. An object enables the feature and overrides only the
 * fields the site set.
 */
export function resolvePwaOptions(value: boolean | PwaOptions | undefined): ResolvedPwaOptions {
  if (!value) {
    return { enabled: false, offline: true };
  }
  if (value === true) {
    return { enabled: true, offline: true };
  }
  return {
    enabled: true,
    offline: value.offline ?? true,
    name: value.name,
    shortName: value.shortName,
    themeColor: value.themeColor,
    backgroundColor: value.backgroundColor,
    startUrl: value.startUrl,
  };
}

/** Builds manifest / service-worker bodies without writing files. */
export function generatePwa(input: PwaRenderInput): PwaRenderResult {
  if (!input.options?.enabled) {
    return {};
  }
  if (!hasSiteUrl(input.siteUrl)) {
    return { warning: MISSING_SITE_URL };
  }

  const base = normalizeBase(input.base);
  const name = sanitizeManifestText(input.options.name ?? input.siteName ?? "");
  const shortName = sanitizeManifestText(input.options.shortName ?? name);
  const startUrl = sanitizeStartUrl(input.options.startUrl, base);
  const themeColor = sanitizeColor(input.options.themeColor, DEFAULT_THEME_COLOR);
  const backgroundColor = sanitizeColor(input.options.backgroundColor, DEFAULT_BACKGROUND_COLOR);

  const result: PwaRenderResult = {
    manifest: `${escapeJsonScript(
      JSON.stringify(
        {
          name,
          short_name: shortName,
          start_url: startUrl,
          scope: base,
          display: "standalone",
          background_color: backgroundColor,
          theme_color: themeColor,
        },
        null,
        2,
      ),
    )}\n`,
  };
  if (input.options.offline) {
    result.serviceWorker = generateServiceWorker(base);
  }
  return result;
}

/** Writes enabled PWA files into `outDir`. */
export async function writePwaFiles(
  input: WritePwaFilesInput,
): Promise<{ files: string[]; warning?: string }> {
  const generated = generatePwa(input);
  if (generated.warning) {
    return { files: [], warning: generated.warning };
  }

  const outputs: Array<[string, string]> = [
    [generated.manifest, MANIFEST_NAME],
    [generated.serviceWorker, SERVICE_WORKER_NAME],
  ].filter((entry): entry is [string, string] => entry[0] != null);
  if (outputs.length === 0) {
    return { files: [] };
  }

  await fs.mkdir(input.outDir, { recursive: true });
  const files: string[] = [];
  for (const [body, name] of outputs) {
    const outputPath = path.join(input.outDir, name);
    await fs.writeFile(outputPath, body, "utf8");
    files.push(outputPath);
  }
  return { files };
}

/**
 * Injects `rel=manifest` (and the service-worker register script when offline)
 * into a themed HTML document. Bare / fragment HTML is left unchanged.
 */
export function injectPwaPageTags(
  html: string,
  input: { options?: ResolvedPwaOptions | null; base?: string },
): string {
  if (!input.options?.enabled || !isThemedDocument(html)) {
    return html;
  }

  const base = normalizeBase(input.base);
  const manifestHref = escapeAttribute(`${base}${MANIFEST_NAME}`);
  const themeColor = sanitizeColor(input.options.themeColor, DEFAULT_THEME_COLOR);
  const headTags = [
    `<link rel="manifest" href="${manifestHref}">`,
    `<meta name="theme-color" content="${escapeAttribute(themeColor)}">`,
  ].join("\n  ");

  let next = insertBeforeTag(html, "</head>", `  ${headTags}\n`);
  if (input.options.offline) {
    const swHref = JSON.stringify(`${base}${SERVICE_WORKER_NAME}`);
    const script = `<script>if("serviceWorker"in navigator)navigator.serviceWorker.register(${swHref})</script>`;
    next = insertBeforeTag(next, "</body>", `  ${script}\n`);
  }
  return next;
}

function generateServiceWorker(base: string): string {
  const assetPrefix = JSON.stringify(`${base}assets/`);
  return `/* ox-content PWA service worker */
const CACHE = "ox-content-pwa-v1";
const ASSET_PREFIX = ${assetPrefix};

self.addEventListener("install", (event) => {
  event.waitUntil(self.skipWaiting());
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  if (isHashedAsset(url.pathname)) {
    event.respondWith(cacheFirst(request));
    return;
  }

  if (isHtmlPage(request)) {
    event.respondWith(networkFirst(request));
  }
});

function isHashedAsset(pathname) {
  if (!pathname.startsWith(ASSET_PREFIX)) return false;
  return /-[0-9a-f]{8,}\\.[a-z0-9]+$/i.test(pathname);
}

function isHtmlPage(request) {
  if (request.mode === "navigate") return true;
  const accept = request.headers.get("accept") || "";
  return accept.includes("text/html");
}

async function cacheFirst(request) {
  const cache = await caches.open(CACHE);
  const cached = await cache.match(request);
  if (cached) return cached;
  const response = await fetch(request);
  if (response.ok) cache.put(request, response.clone());
  return response;
}

async function networkFirst(request) {
  const cache = await caches.open(CACHE);
  try {
    const response = await fetch(request);
    if (response.ok) cache.put(request, response.clone());
    return response;
  } catch (error) {
    const cached = await cache.match(request);
    if (cached) return cached;
    throw error;
  }
}
`;
}

function hasSiteUrl(siteUrl: string | undefined): boolean {
  return Boolean(siteUrl && siteUrl.trim());
}

function normalizeBase(base: string | undefined): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}

function sanitizeManifestText(value: string): string {
  return value.split(/\s+/u).filter(Boolean).join(" ");
}

function sanitizeColor(value: string | undefined, fallback: string): string {
  if (!value) {
    return fallback;
  }
  const trimmed = value.trim();
  if (/^#[0-9A-Fa-f]{3,8}$/.test(trimmed)) {
    return trimmed;
  }
  if (/^[a-zA-Z][a-zA-Z0-9-]{0,31}$/.test(trimmed)) {
    return trimmed;
  }
  return fallback;
}

function sanitizeStartUrl(value: string | undefined, base: string): string {
  if (!value) {
    return base;
  }
  const trimmed = value.trim();
  if (!trimmed.startsWith("/") || trimmed.startsWith("//")) {
    return base;
  }
  if (/[\n\r\t<>"'`]/.test(trimmed)) {
    return base;
  }
  if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(trimmed)) {
    return base;
  }
  return trimmed;
}

function isThemedDocument(html: string): boolean {
  return /<\/head>/i.test(html) && /<\/body>/i.test(html);
}

function insertBeforeTag(html: string, tag: string, snippet: string): string {
  const index = html.toLowerCase().lastIndexOf(tag.toLowerCase());
  if (index === -1) {
    return html;
  }
  return `${html.slice(0, index)}${snippet}${html.slice(index)}`;
}

function escapeJsonScript(value: string): string {
  return value.replace(/[<>]/g, (ch) => (ch === "<" ? "\\u003c" : "\\u003e"));
}

function escapeAttribute(value: string): string {
  return value.replace(/[&<>"']/g, (ch) => {
    switch (ch) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      default:
        return "&#39;";
    }
  });
}
