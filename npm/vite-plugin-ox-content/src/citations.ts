import { readFile } from "node:fs/promises";
import path from "node:path";
import type { OxContentOptions, ResolvedOptions } from "./types";
import { escapeAttr, escapeUrlFragment, transformText } from "./cross-reference-html";
import {
  bibliographyPlainText,
  citationId,
  getBibliographyEntry,
  renderBibliography,
  renderCitationLink,
  type CslItem,
} from "./citation-format";
import type {
  BibliographyEntry,
  CitationFailureMode,
  CitationReference,
  ResolvedCitationsOptions,
} from "./citation-types";

export type {
  BibliographyEntry,
  CitationFailureMode,
  CitationReference,
  CitationsOptions,
  ResolvedCitationsOptions,
} from "./citation-types";

interface CitationDiagnostic {
  policy: CitationFailureMode;
  message: string;
}

const disabled: ResolvedCitationsOptions = {
  enabled: false,
  bibliography: [],
  appendBibliography: true,
  missing: "error",
  duplicates: "error",
  malformed: "error",
  bibliographyTitle: "References",
};

const CITATION_KEY_RE = /^[A-Za-z][A-Za-z0-9_.:-]*$/;
const BRACKET_RE = /\[([^\]\n]+)\]/g;

export function resolveCitationsOptions(
  options: OxContentOptions["citations"],
): ResolvedOptions["citations"] {
  if (!options) return { ...disabled, bibliography: [] };
  if (options === true) return { ...disabled, enabled: true, bibliography: [] };
  return {
    enabled: options.enabled ?? true,
    bibliography: toArray(options.bibliography),
    rootDir: options.rootDir,
    appendBibliography: options.appendBibliography ?? true,
    missing: options.missing === "warn" ? "warn" : "error",
    duplicates: options.duplicates === "warn" ? "warn" : "error",
    malformed: options.malformed === "warn" ? "warn" : "error",
    bibliographyTitle: options.bibliographyTitle ?? disabled.bibliographyTitle,
  };
}

export async function transformCitations(
  html: string,
  options: ResolvedCitationsOptions | undefined,
): Promise<{ html: string; citations: CitationReference[]; bibliography: BibliographyEntry[] }> {
  if (!options?.enabled) return { html, citations: [], bibliography: [] };

  const diagnostics: CitationDiagnostic[] = [];
  const sourceItems = await loadBibliography(options, diagnostics);
  const entries = new Map<string, BibliographyEntry>();
  const citations: CitationReference[] = [];

  let nextHtml = transformText(html, (text) =>
    text.replace(BRACKET_RE, (full: string, rawBody: string) => {
      const body = rawBody.trim();
      if (!body.startsWith("@") && !body.startsWith("-@")) return full;

      const parsed = parseCitationGroup(body, options, diagnostics);
      if (!parsed) return full;

      const sourceEntries: Array<{ key: string; suppressAuthor: boolean; item: CslItem }> = [];
      for (const cite of parsed) {
        const item = sourceItems.get(cite.key);
        if (!item) {
          diagnostics.push({
            policy: options.missing,
            message: `missing citation key "${cite.key}"`,
          });
          return full;
        }
        sourceEntries.push({ ...cite, item });
      }

      const links: string[] = [];
      for (const cite of sourceEntries) {
        const entry = getBibliographyEntry(entries, cite.key, cite.item);
        const id = citationId(cite.key, citations.length + 1);
        const reference: CitationReference = {
          id,
          key: cite.key,
          index: entry.index,
          label: entry.label,
          href: `#${escapeUrlFragment(entry.id)}`,
          bibliographyId: entry.id,
          suppressAuthor: cite.suppressAuthor,
        };
        citations.push(reference);
        links.push(renderCitationLink(reference, entry, parsed.length === 1));
      }

      const label = citations
        .slice(citations.length - parsed.length)
        .map((ref) => ref.label)
        .join(", ");
      const inner = parsed.length === 1 ? links[0]! : `[${links.join("; ")}]`;
      return `<span class="ox-cite" role="group" aria-label="Citations ${escapeAttr(label)}">${inner}</span>`;
    }),
  );

  flushDiagnostics(diagnostics);

  const bibliography = [...entries.values()].sort((a, b) => a.index - b.index);
  if (options.appendBibliography && bibliography.length > 0) {
    nextHtml += renderBibliography(bibliography, options.bibliographyTitle);
  }

  return { html: nextHtml, citations, bibliography };
}

export async function collectCitationSearchText(
  markdown: string,
  options: ResolvedCitationsOptions | undefined,
): Promise<string> {
  if (!options?.enabled) return "";

  const diagnostics: CitationDiagnostic[] = [];
  const sourceItems = await loadBibliography(options, diagnostics);
  const keys = citationKeysFromMarkdown(markdown, options, diagnostics);
  const seen = new Set<string>();
  const parts: string[] = [];

  for (const key of keys) {
    const item = sourceItems.get(key);
    if (!item) {
      diagnostics.push({
        policy: options.missing,
        message: `missing citation key "${key}"`,
      });
      continue;
    }
    if (seen.has(key)) continue;
    seen.add(key);
    parts.push(bibliographyPlainText(item));
  }

  flushDiagnostics(diagnostics);
  return parts.filter(Boolean).join("\n");
}

function parseCitationGroup(
  body: string,
  options: ResolvedCitationsOptions,
  diagnostics: CitationDiagnostic[],
): Array<{ key: string; suppressAuthor: boolean }> | null {
  const citations = body.split(";").map((part) => part.trim());
  if (citations.length === 0 || citations.some((part) => part.length === 0)) {
    diagnostics.push({ policy: options.malformed, message: `malformed citation "[${body}]"` });
    return null;
  }

  const parsed: Array<{ key: string; suppressAuthor: boolean }> = [];
  for (const citation of citations) {
    const suppressAuthor = citation.startsWith("-@");
    const key = citation.slice(suppressAuthor ? 2 : 1);
    if ((!citation.startsWith("@") && !suppressAuthor) || !CITATION_KEY_RE.test(key)) {
      diagnostics.push({ policy: options.malformed, message: `malformed citation "[${body}]"` });
      return null;
    }
    parsed.push({ key, suppressAuthor });
  }
  return parsed;
}

function citationKeysFromMarkdown(
  markdown: string,
  options: ResolvedCitationsOptions,
  diagnostics: CitationDiagnostic[],
): string[] {
  const keys: string[] = [];
  const visibleMarkdown = stripMarkdownCitationProtectedText(markdown);
  for (const match of visibleMarkdown.matchAll(BRACKET_RE)) {
    const body = (match[1] ?? "").trim();
    if (!body.startsWith("@") && !body.startsWith("-@")) continue;
    if (isLinkLabel(visibleMarkdown, match)) continue;
    const parsed = parseCitationGroup(body, options, diagnostics);
    if (!parsed) continue;
    keys.push(...parsed.map((citation) => citation.key));
  }
  return keys;
}

/**
 * A bracketed span that a destination or reference immediately follows is a
 * Markdown link label, not a citation group.
 *
 * The HTML pass never sees these — by then the label is inside an `<a>` and has
 * no brackets left — but the search-index pass reads the Markdown source, where
 * a linked scoped package name is indistinguishable from a citation by its
 * opening `@` alone. `[@ox-content/vite-plugin](./packages/vite-plugin.md)`
 * reported a malformed citation and failed the whole index.
 */
function isLinkLabel(markdown: string, match: RegExpMatchArray): boolean {
  const end = (match.index ?? 0) + match[0].length;
  return markdown[end] === "(" || markdown[end] === "[";
}

function stripMarkdownCitationProtectedText(markdown: string): string {
  return markdown
    .replace(/^([ \t]*)(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\2[ \t]*$/gm, "")
    .replace(/^(?: {4}|\t).+$/gm, "")
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/<(pre|code|script|style|textarea|a)\b[\s\S]*?<\/\1>/gi, "")
    .replace(/`+[^`\n]*`+/g, "");
}

async function loadBibliography(
  options: ResolvedCitationsOptions,
  diagnostics: CitationDiagnostic[],
): Promise<Map<string, CslItem>> {
  const items = new Map<string, CslItem>();
  for (const spec of options.bibliography) {
    const file = resolveBibliographyPath(spec, options, diagnostics);
    if (!file) continue;
    let parsed: unknown;
    try {
      parsed = JSON.parse(await readFile(file, "utf8"));
    } catch (error) {
      diagnostics.push({
        policy: options.malformed,
        message: `failed to read CSL JSON bibliography "${spec}": ${messageOf(error)}`,
      });
      continue;
    }

    const entries = Array.isArray(parsed)
      ? parsed
      : isRecord(parsed) && Array.isArray(parsed.items)
        ? parsed.items
        : null;
    if (!entries) {
      diagnostics.push({
        policy: options.malformed,
        message: `CSL JSON bibliography "${spec}" must be an array of items`,
      });
      continue;
    }

    for (const entry of entries) {
      if (!isRecord(entry) || typeof entry.id !== "string" || !CITATION_KEY_RE.test(entry.id)) {
        diagnostics.push({
          policy: options.malformed,
          message: `CSL JSON bibliography "${spec}" contains an item without a valid id`,
        });
        continue;
      }
      if (items.has(entry.id)) {
        diagnostics.push({
          policy: options.duplicates,
          message: `duplicate citation key "${entry.id}"`,
        });
        continue;
      }
      items.set(entry.id, entry as unknown as CslItem);
    }
  }
  return items;
}

function resolveBibliographyPath(
  spec: string,
  options: ResolvedCitationsOptions,
  diagnostics: CitationDiagnostic[],
): string | null {
  if (spec.includes("\0") || /^[A-Za-z][A-Za-z0-9+.-]*:/.test(spec)) {
    diagnostics.push({
      policy: options.malformed,
      message: `bibliography path "${spec}" must be a local file path`,
    });
    return null;
  }
  const root = path.resolve(options.rootDir ?? process.cwd());
  const resolved = path.isAbsolute(spec) ? path.resolve(spec) : path.resolve(root, spec);
  const relative = path.relative(root, resolved);
  if (relative === "" || relative.startsWith("..") || path.isAbsolute(relative)) {
    diagnostics.push({
      policy: options.malformed,
      message: `bibliography path "${spec}" escapes rootDir`,
    });
    return null;
  }
  return resolved;
}

function flushDiagnostics(diagnostics: CitationDiagnostic[]): void {
  const errors = diagnostics.filter((diagnostic) => diagnostic.policy === "error");
  if (errors.length > 0) {
    throw new Error(`[ox-content] ${errors.map((diagnostic) => diagnostic.message).join("\n")}`);
  }
  const warnings = diagnostics.filter((diagnostic) => diagnostic.policy === "warn");
  if (warnings.length > 0) {
    console.warn(`[ox-content] Citation warnings:\n${warnings.map((w) => w.message).join("\n")}`);
  }
}

function toArray(value: string | string[] | undefined): string[] {
  if (!value) return [];
  return Array.isArray(value) ? value : [value];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
