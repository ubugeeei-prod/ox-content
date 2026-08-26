import { escapeAttr, escapeHtml } from "./cross-reference-html";
import type { BibliographyEntry, CitationReference } from "./citation-types";

export interface CslItem {
  id: string;
  title?: string;
  author?: CslName[];
  editor?: CslName[];
  issued?: { "date-parts"?: unknown; raw?: unknown };
  URL?: string;
  DOI?: string;
  publisher?: string;
  "container-title"?: string;
}

interface CslName {
  family?: string;
  given?: string;
  literal?: string;
}

export function getBibliographyEntry(
  entries: Map<string, BibliographyEntry>,
  key: string,
  item: CslItem,
): BibliographyEntry {
  const existing = entries.get(key);
  if (existing) return existing;

  const index = entries.size + 1;
  const html = bibliographyHtml(item);
  const entry: BibliographyEntry = {
    key,
    id: `ref-${slugWithHash(key)}`,
    index,
    label: String(index),
    title: item.title ?? key,
    authors: authorsOf(item),
    year: yearOf(item),
    url: safeUrl(item.URL),
    doi: item.DOI,
    html,
  };
  entries.set(key, entry);
  return entry;
}

export function renderCitationLink(
  reference: CitationReference,
  entry: BibliographyEntry,
  includeBrackets: boolean,
): string {
  const label = includeBrackets ? `[${reference.label}]` : reference.label;
  return `<a class="ox-cite__ref" id="${escapeAttr(reference.id)}" href="${escapeAttr(reference.href)}" data-ox-citation-key="${escapeAttr(reference.key)}" data-ox-citation-index="${reference.index}" aria-label="Citation ${reference.label}: ${escapeAttr(entry.title)}">${label}</a>`;
}

export function renderBibliography(entries: BibliographyEntry[], title: string): string {
  const titleId = "ox-bibliography-title";
  const items = entries
    .map(
      (entry) =>
        `<li class="ox-bibliography__item" id="${escapeAttr(entry.id)}" data-ox-citation-key="${escapeAttr(entry.key)}" value="${entry.index}">${entry.html}</li>`,
    )
    .join("");
  return `<section class="ox-bibliography" aria-labelledby="${titleId}"><h2 class="ox-bibliography__title" id="${titleId}">${escapeHtml(title)}</h2><ol class="ox-bibliography__list">${items}</ol></section>`;
}

export function bibliographyPlainText(item: CslItem): string {
  return [
    authorsOf(item).join(", "),
    yearOf(item),
    item.title ?? item.id,
    item["container-title"],
    item.publisher,
    item.DOI ? `doi:${item.DOI}` : undefined,
    safeUrl(item.URL),
  ]
    .filter((part): part is string => Boolean(part))
    .join(" ");
}

export function citationId(key: string, index: number): string {
  return `cite-${slugWithHash(key)}-${index}`;
}

function bibliographyHtml(item: CslItem): string {
  const authors = authorsOf(item).join(", ");
  const year = yearOf(item);
  const title = item.title ?? item.id;
  const parts = [
    authors ? `${escapeHtml(authors)}${year ? ` (${escapeHtml(year)})` : ""}.` : undefined,
    `<cite>${escapeHtml(title)}</cite>.`,
    item["container-title"] ? escapeHtml(item["container-title"]) + "." : undefined,
    item.publisher ? escapeHtml(item.publisher) + "." : undefined,
    citationLink(item),
  ];
  return parts.filter(Boolean).join(" ");
}

function citationLink(item: CslItem): string | undefined {
  const url = safeUrl(item.URL);
  if (url) {
    return `<a class="ox-bibliography__url" href="${escapeAttr(url)}">${escapeHtml(url)}</a>.`;
  }
  if (item.DOI && /^[^\s<>"]+$/.test(item.DOI)) {
    const doi = `https://doi.org/${item.DOI}`;
    return `<a class="ox-bibliography__doi" href="${escapeAttr(doi)}">doi:${escapeHtml(item.DOI)}</a>.`;
  }
  return undefined;
}

function authorsOf(item: CslItem): string[] {
  return (item.author ?? item.editor ?? []).map(formatName).filter((name) => name.length > 0);
}

function formatName(name: CslName): string {
  if (name.literal) return name.literal;
  return [name.given, name.family].filter(Boolean).join(" ");
}

function yearOf(item: CslItem): string | undefined {
  const first = item.issued?.["date-parts"];
  const year = Array.isArray(first) && Array.isArray(first[0]) ? first[0][0] : undefined;
  return typeof year === "number" || typeof year === "string" ? String(year) : undefined;
}

function safeUrl(value: unknown): string | undefined {
  if (typeof value !== "string" || !/^https?:\/\//.test(value)) return undefined;
  return value;
}

function slugWithHash(value: string): string {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return `${slug || "item"}-${hash(value)}`;
}

function hash(value: string): string {
  let hashValue = 0x811c9dc5;
  for (let index = 0; index < value.length; index += 1) {
    hashValue ^= value.charCodeAt(index);
    hashValue = Math.imul(hashValue, 0x01000193);
  }
  return (hashValue >>> 0).toString(36);
}
