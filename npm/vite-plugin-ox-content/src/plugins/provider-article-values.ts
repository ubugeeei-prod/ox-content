/**
 * Value shaping for provider article cards: reading and writing tag
 * attributes, and narrowing the loosely-typed JSON a provider returns.
 */

import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";
import type { ArticleMeta } from "./provider-articles";

export function appendAttr(attrs: string, name: string, value: string | undefined): string {
  if (!value || readAttr(attrs, name)) return attrs;
  return `${attrs} ${name}="${escapeProviderArticleAttr(value)}"`;
}

export function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? decodeProviderArticleAttr(value) : undefined;
}

export function namesList(value: unknown): string | undefined {
  if (!Array.isArray(value)) return undefined;
  const names = value.map((item) => trimmed(record(item)?.name)).filter(Boolean);
  return names.length ? names.join(", ") : undefined;
}

export function count(value: unknown): string | undefined {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? String(Math.floor(value))
    : undefined;
}

export function dateLabel(value: string | undefined): string | undefined {
  return value && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : undefined;
}

export function excerpt(value: string | undefined): string | undefined {
  const normalized = value
    ?.replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^#+\s*/gm, "")
    .replace(/\s+/g, " ")
    .trim();
  if (!normalized) return undefined;
  return normalized.length <= 280 ? normalized : `${normalized.slice(0, 279).trimEnd()}...`;
}

export function compactMeta(meta: ArticleMeta): ArticleMeta {
  return Object.fromEntries(Object.entries(meta).filter(([, value]) => value)) as ArticleMeta;
}

export function safeHttps(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

export function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

export function safeSlug(value: string | undefined): boolean {
  return Boolean(value && /^[A-Za-z0-9][A-Za-z0-9_-]{1,127}$/.test(value));
}

export function safeId(value: string | undefined): boolean {
  return Boolean(value && /^[A-Za-z0-9][A-Za-z0-9_-]{5,127}$/.test(value));
}

export function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

export function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
