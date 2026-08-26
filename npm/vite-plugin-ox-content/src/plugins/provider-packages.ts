import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";
import { packageMetaFromJson, type PackageMeta } from "./provider-package-metadata";

const PACKAGE_TAG =
  /<(npmpackage|cratesio|pypi|dockerhub)\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/\1\s*>/gi;
const DEFAULT_TIMEOUT = 10_000;
const DEFAULT_CACHE_TTL = 3_600_000;

export interface ProviderPackageEmbedOptions {
  /** Fetch package metadata at build time. @default true */
  fetch?: boolean;
  /** Metadata request timeout in milliseconds. @default 10000 */
  timeout?: number;
  /** Cache fetched metadata in memory for the current process. @default true */
  cache?: boolean;
  /** Cache TTL in milliseconds. @default 3600000 */
  cacheTTL?: number;
}

export interface ResolvedProviderPackageEmbedOptions {
  fetch: boolean;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
}

export interface PackageRegistryReference {
  provider: "npm" | "crates.io" | "pypi" | "docker-hub";
  apiUrl: string;
  name: string;
  ecosystem: string;
  version?: string;
}

interface CacheRecord {
  data: PackageMeta | null;
  timestamp: number;
}

export type ProviderPackageFetch = (input: string, init?: RequestInit) => Promise<Response>;

const memoryCache = new Map<string, CacheRecord>();
const inflight = new Map<string, Promise<PackageMeta | null>>();

export function clearProviderPackageCache(): void {
  memoryCache.clear();
  inflight.clear();
}

export function resolveProviderPackageEmbedOptions(
  options: ProviderPackageEmbedOptions = {},
): ResolvedProviderPackageEmbedOptions {
  return {
    fetch: options.fetch ?? true,
    timeout: options.timeout ?? DEFAULT_TIMEOUT,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? DEFAULT_CACHE_TTL,
  };
}

export function normalizeProviderPackageOptions(
  options: boolean | ProviderPackageEmbedOptions | undefined,
): ProviderPackageEmbedOptions | false {
  if (!options) return false;
  return options === true ? {} : options;
}

export async function enrichProviderPackageEmbeds(
  html: string,
  options: ProviderPackageEmbedOptions | false,
  fetchImpl: ProviderPackageFetch = fetch,
): Promise<string> {
  if (!options || !/<(?:npmpackage|cratesio|pypi|dockerhub)\b/i.test(html)) return html;
  const resolved = resolveProviderPackageEmbedOptions(options);
  if (!resolved.fetch) return html;

  const matches = Array.from(html.matchAll(PACKAGE_TAG), (match) => ({
    tag: match[1] ?? "",
    attrs: match[2] ?? "",
    body: match[3] ?? "",
    full: match[0],
    index: match.index ?? 0,
    end: (match.index ?? 0) + match[0].length,
  }));
  const enriched = await Promise.all(matches.map((match) => enrichTag(match, resolved, fetchImpl)));

  let output = "";
  let cursor = 0;
  for (let index = 0; index < matches.length; index += 1) {
    const match = matches[index]!;
    output += html.slice(cursor, match.index);
    output += enriched[index];
    cursor = match.end;
  }
  return output + html.slice(cursor);
}

export function parsePackageRegistryReference(
  tag: string,
  input: string,
): PackageRegistryReference | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.username || url.password) return null;
    const segments = safeSegments(url);
    if (!segments) return null;
    switch (tag.toLowerCase()) {
      case "npmpackage":
        return npmReference(url, segments);
      case "cratesio":
        return cratesReference(url, segments);
      case "pypi":
        return pypiReference(url, segments);
      case "dockerhub":
        return dockerReference(url, segments);
      default:
        return null;
    }
  } catch {
    return null;
  }
}

async function enrichTag(
  match: { tag: string; attrs: string; body: string; full: string },
  options: ResolvedProviderPackageEmbedOptions,
  fetchImpl: ProviderPackageFetch,
): Promise<string> {
  const href =
    readAttr(match.attrs, "url") ?? readAttr(match.attrs, "href") ?? readAttr(match.attrs, "src");
  if (!href) return match.full;
  const reference = parsePackageRegistryReference(match.tag, href);
  if (!reference) return match.full;
  const meta = await fetchPackageMeta(reference, options, fetchImpl);
  if (!meta) return match.full;

  let attrs = match.attrs;
  attrs = appendAttr(attrs, "packageName", reference.name);
  attrs = appendAttr(attrs, "version", meta.version ?? reference.version);
  attrs = appendAttr(attrs, "title", meta.title ?? reference.name);
  attrs = appendAttr(attrs, "description", meta.description);
  attrs = appendAttr(attrs, "license", meta.license);
  attrs = appendAttr(attrs, "repository", meta.repository);
  attrs = appendAttr(attrs, "downloads", meta.downloads);
  attrs = appendAttr(attrs, "stars", meta.stars);
  attrs = appendAttr(attrs, "dateTime", meta.dateTime);
  attrs = appendAttr(attrs, "dateLabel", meta.dateLabel);
  return `<${match.tag}${attrs}>${match.body}</${match.tag}>`;
}

async function fetchPackageMeta(
  reference: PackageRegistryReference,
  options: ResolvedProviderPackageEmbedOptions,
  fetchImpl: ProviderPackageFetch,
): Promise<PackageMeta | null> {
  const key = `${reference.provider}:${reference.apiUrl}:${reference.version ?? ""}`;
  const now = Date.now();
  if (options.cache) {
    const cached = memoryCache.get(key);
    if (cached && now - cached.timestamp < options.cacheTTL) return cached.data;
  }
  const pending = inflight.get(key);
  if (pending) return pending;

  const request = requestPackageMeta(reference, options, fetchImpl).finally(() => {
    if (inflight.get(key) === request) inflight.delete(key);
  });
  inflight.set(key, request);
  const data = await request;
  if (options.cache) memoryCache.set(key, { data, timestamp: now });
  return data;
}

async function requestPackageMeta(
  reference: PackageRegistryReference,
  options: ResolvedProviderPackageEmbedOptions,
  fetchImpl: ProviderPackageFetch,
): Promise<PackageMeta | null> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetchImpl(reference.apiUrl, {
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) {
      warnPackageFallback(reference, String(response.status));
      return null;
    }
    const value = await response.json();
    return packageMetaFromJson(value, reference);
  } catch (error) {
    warnPackageFallback(reference, error instanceof Error ? error.message : "unknown error");
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

function warnPackageFallback(reference: PackageRegistryReference, reason: string): void {
  console.warn(
    `[ox-content] Failed to fetch ${reference.provider} metadata for ${reference.name}: ${reason}; rendering a link-only package card.`,
  );
}

function npmReference(url: URL, segments: string[]): PackageRegistryReference | null {
  if (!["www.npmjs.com", "npmjs.com"].includes(url.hostname) || segments[0] !== "package") {
    return null;
  }
  const scoped = segments[1]?.startsWith("@");
  const name = scoped ? `${segments[1]}/${segments[2]}` : segments[1];
  if (!name || !safeNpmPackage(name)) return null;
  const versionIndex = scoped ? 3 : 2;
  const version =
    segments[versionIndex] === "v" ? safeVersion(segments[versionIndex + 1]) : undefined;
  return {
    provider: "npm",
    apiUrl: `https://registry.npmjs.org/${encodeURIComponent(name)}`,
    name,
    ecosystem: "npm",
    ...(version ? { version } : {}),
  };
}

function cratesReference(url: URL, segments: string[]): PackageRegistryReference | null {
  const name = segments[0] === "crates" ? safeName(segments[1]) : undefined;
  if (url.hostname !== "crates.io" || !name) return null;
  const version = safeVersion(segments[2]);
  return {
    provider: "crates.io",
    apiUrl: `https://crates.io/api/v1/crates/${encodeURIComponent(name)}`,
    name,
    ecosystem: "crates.io",
    ...(version ? { version } : {}),
  };
}

function pypiReference(url: URL, segments: string[]): PackageRegistryReference | null {
  const name = segments[0] === "project" ? safeName(segments[1]) : undefined;
  if (url.hostname !== "pypi.org" || !name) return null;
  const version = safeVersion(segments[2]);
  const suffix = version ? `/${encodeURIComponent(version)}` : "";
  return {
    provider: "pypi",
    apiUrl: `https://pypi.org/pypi/${encodeURIComponent(name)}${suffix}/json`,
    name,
    ecosystem: "PyPI",
    ...(version ? { version } : {}),
  };
}

function dockerReference(url: URL, segments: string[]): PackageRegistryReference | null {
  if (url.hostname !== "hub.docker.com") return null;
  const parsed =
    segments[0] === "_" && segments[1]
      ? { namespace: "library", repo: segments[1], rest: segments.slice(2) }
      : segments[0] === "r" && segments[1] && segments[2]
        ? { namespace: segments[1], repo: segments[2], rest: segments.slice(3) }
        : segments[0] === "repository" && segments[1] === "docker" && segments[2] && segments[3]
          ? { namespace: segments[2], repo: segments[3], rest: segments.slice(4) }
          : null;
  if (!parsed || !safeName(parsed.namespace) || !safeName(parsed.repo)) return null;
  const rawTag =
    parsed.rest[0] === "tags"
      ? (parsed.rest[1] ?? url.searchParams.get("name"))
      : url.searchParams.get("name");
  const tag = safeVersion(rawTag);
  const name = `${parsed.namespace}/${parsed.repo}`;
  return {
    provider: "docker-hub",
    apiUrl: `https://hub.docker.com/v2/repositories/${encodeURIComponent(parsed.namespace)}/${encodeURIComponent(parsed.repo)}`,
    name,
    ecosystem: "Docker Hub",
    ...(tag ? { version: tag } : {}),
  };
}

function appendAttr(attrs: string, name: string, value: string | undefined): string {
  if (!value || readAttr(attrs, name)) return attrs;
  return `${attrs} ${name}="${escapeProviderArticleAttr(value)}"`;
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? decodeProviderArticleAttr(value) : undefined;
}

function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function safeNpmPackage(value: string): boolean {
  return /^(?:@[a-z0-9][a-z0-9._~-]*\/)?[a-z0-9][a-z0-9._~-]*$/i.test(value);
}

function safeName(value: string | undefined): string | undefined {
  return value && /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(value) ? value : undefined;
}

function safeVersion(value: string | null | undefined): string | undefined {
  return value && /^[A-Za-z0-9][A-Za-z0-9._+:@-]{0,127}$/.test(value) ? value : undefined;
}
