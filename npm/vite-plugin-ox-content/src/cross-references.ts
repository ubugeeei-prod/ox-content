import type { OxContentOptions, ResolvedOptions } from "./types";
import {
  appendAttr,
  appendDataAttrs,
  escapeAttr,
  escapeHtml,
  escapeUrlFragment,
  expectedKind,
  figureCaption,
  findFirstImageWithId,
  readAttr,
  shouldTrackTarget,
  textContent,
  transformText,
  transformTextOutsideCitationGroups,
} from "./cross-reference-html";
import type {
  CrossReferenceEntry,
  CrossReferenceFailureMode,
  ResolvedCrossReferencesOptions,
} from "./cross-reference-types";

export type {
  CrossReferenceEntry,
  CrossReferenceFailureMode,
  CrossReferenceKind,
  CrossReferenceLabelOptions,
  CrossReferencesOptions,
  ResolvedCrossReferencesOptions,
} from "./cross-reference-types";

interface CrossReferenceTarget extends CrossReferenceEntry {
  position: number;
}

interface CrossReferenceDiagnostic {
  policy: CrossReferenceFailureMode;
  message: string;
}

const disabled: ResolvedCrossReferencesOptions = {
  enabled: false,
  missing: "error",
  duplicates: "error",
  mismatches: "error",
  labels: { figure: "Figure", table: "Table", section: "Section" },
};

const TEXT_REFERENCE_RE = /(^|[^\w@/[])@([A-Za-z][A-Za-z0-9_-]*)\b/g;
export function resolveCrossReferencesOptions(
  options: OxContentOptions["crossReferences"],
): ResolvedOptions["crossReferences"] {
  if (!options) return { ...disabled, labels: { ...disabled.labels } };
  if (options === true) return { ...disabled, enabled: true, labels: { ...disabled.labels } };
  return {
    enabled: options.enabled ?? true,
    missing: options.missing === "warn" ? "warn" : "error",
    duplicates: options.duplicates === "warn" ? "warn" : "error",
    mismatches: options.mismatches === "warn" ? "warn" : "error",
    labels: {
      figure: options.labels?.figure ?? disabled.labels.figure,
      table: options.labels?.table ?? disabled.labels.table,
      section: options.labels?.section ?? disabled.labels.section,
    },
  };
}

export function transformCrossReferences(
  html: string,
  options: ResolvedCrossReferencesOptions | undefined,
): { html: string; references: CrossReferenceEntry[] } {
  if (!options?.enabled) return { html, references: [] };

  const diagnostics: CrossReferenceDiagnostic[] = [];
  const targets = new Map<string, CrossReferenceTarget>();
  let nextHtml = annotateSections(html, options, targets, diagnostics);
  nextHtml = annotateFiguresAndImages(nextHtml, options, targets, diagnostics);
  nextHtml = applyTrailingTableLabels(nextHtml);
  nextHtml = annotateTables(nextHtml, options, targets, diagnostics);
  flushDiagnostics(diagnostics);

  nextHtml = replaceReferences(nextHtml, options, targets, diagnostics);
  flushDiagnostics(diagnostics);

  return {
    html: nextHtml,
    references: [...targets.values()]
      .sort((a, b) => a.position - b.position)
      .map(({ position: _position, ...reference }) => reference),
  };
}

function annotateSections(
  html: string,
  options: ResolvedCrossReferencesOptions,
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
): string {
  const counters = [0, 0, 0, 0, 0, 0];
  return html.replace(
    /<h([1-6])\b([^>]*)>([\s\S]*?)<\/h\1>/gi,
    (full: string, rawDepth: string, attrs: string, body: string, offset: number) => {
      const depth = Number(rawDepth);
      counters[depth - 1] += 1;
      counters.fill(0, depth);
      const id = readAttr(attrs, "id");
      if (!id || !shouldTrackTarget(id)) return full;

      const number = counters.slice(0, depth).filter(Boolean).join(".");
      const text = `${options.labels.section} ${number || counters[depth - 1]}`;
      registerTarget(targets, diagnostics, options.duplicates, {
        id,
        kind: "section",
        number,
        label: options.labels.section,
        text,
        href: `#${escapeUrlFragment(id)}`,
        title: textContent(body),
        position: offset,
      });
      return `<h${rawDepth}${appendDataAttrs(attrs, "section", number, text)}>${body}</h${rawDepth}>`;
    },
  );
}

function annotateFiguresAndImages(
  html: string,
  options: ResolvedCrossReferencesOptions,
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
): string {
  let count = 0;
  let output = "";
  let cursor = 0;
  const blockRe = /<figure\b([^>]*)>([\s\S]*?)<\/figure>|<img\b([^>]*)>/gi;
  for (const match of html.matchAll(blockRe)) {
    const offset = match.index ?? 0;
    output += html.slice(cursor, offset);

    if (match[1] !== undefined) {
      const attrs = match[1] ?? "";
      const body = match[2] ?? "";
      const figureId = readAttr(attrs, "id");
      const image = figureId ? null : findFirstImageWithId(body);
      const id = figureId ?? image?.id;
      if (!id || !shouldTrackTarget(id)) {
        output += match[0];
        cursor = offset + match[0].length;
        continue;
      }

      count += 1;
      const number = String(count);
      const text = `${options.labels.figure} ${number}`;
      registerTarget(targets, diagnostics, options.duplicates, {
        id,
        kind: "figure",
        number,
        label: options.labels.figure,
        text,
        href: `#${escapeUrlFragment(id)}`,
        title: figureCaption(body) ?? image?.alt,
        position: offset,
      });

      if (figureId) {
        output += `<figure${appendDataAttrs(attrs, "figure", number, text)}>${body}</figure>`;
        cursor = offset + match[0].length;
        continue;
      }
      const nextImage = `<img${appendDataAttrs(image.attrs, "figure", number, text)}>`;
      output += `<figure${attrs}>${body.slice(0, image.start)}${nextImage}${body.slice(image.end)}</figure>`;
      cursor = offset + match[0].length;
      continue;
    }

    const attrs = match[3] ?? "";
    if (readAttr(attrs, "data-ox-xref-kind")) {
      output += match[0];
      cursor = offset + match[0].length;
      continue;
    }
    const id = readAttr(attrs, "id");
    if (!id || !shouldTrackTarget(id)) {
      output += match[0];
      cursor = offset + match[0].length;
      continue;
    }

    count += 1;
    const number = String(count);
    const text = `${options.labels.figure} ${number}`;
    registerTarget(targets, diagnostics, options.duplicates, {
      id,
      kind: "figure",
      number,
      label: options.labels.figure,
      text,
      href: `#${escapeUrlFragment(id)}`,
      title: readAttr(attrs, "alt"),
      position: offset,
    });
    output += `<img${appendDataAttrs(attrs, "figure", number, text)}>`;
    cursor = offset + match[0].length;
  }
  return output + html.slice(cursor);
}

function annotateTables(
  html: string,
  options: ResolvedCrossReferencesOptions,
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
): string {
  let count = 0;
  return html.replace(/<table\b([^>]*)>/gi, (full: string, attrs: string, offset: number) => {
    const id = readAttr(attrs, "id");
    if (!id || !shouldTrackTarget(id)) return full;

    count += 1;
    const number = String(count);
    const text = `${options.labels.table} ${number}`;
    registerTarget(targets, diagnostics, options.duplicates, {
      id,
      kind: "table",
      number,
      label: options.labels.table,
      text,
      href: `#${escapeUrlFragment(id)}`,
      position: offset,
    });
    return `<table${appendDataAttrs(attrs, "table", number, text)}>`;
  });
}

function applyTrailingTableLabels(html: string): string {
  const liftedParagraphLabels = html.replace(
    /(<table\b[^>]*>[\s\S]*?<\/table>)\s*<p>\{#([A-Za-z][A-Za-z0-9_-]*)\}<\/p>/gi,
    (full: string, table: string, id: string) => {
      const open = /^<table\b([^>]*)>/i.exec(table);
      if (!open || readAttr(open[1] ?? "", "id")) return full;
      const nextOpen = `<table${appendAttr(open[1] ?? "", "id", id)}>`;
      return nextOpen + table.slice(open[0].length);
    },
  );
  return liftedParagraphLabels.replace(
    /<table\b([^>]*)>([\s\S]*?)<\/table>/gi,
    (full: string, attrs: string, body: string) => {
      if (readAttr(attrs, "id")) return full;
      const label = trailingTableCellLabel(body);
      if (!label) return full;
      return `<table${appendAttr(attrs, "id", label.id)}>${label.body}</table>`;
    },
  );
}

function trailingTableCellLabel(body: string): { id: string; body: string } | null {
  const match =
    /(\s*<tr>\s*<td\b([^>]*)>\s*<\/td>(?:\s*<td\b[^>]*>\s*<\/td>)*\s*<\/tr>)(\s*<\/tbody>\s*)$/i.exec(
      body,
    );
  if (!match) return null;
  const id = readAttr(match[2] ?? "", "id");
  if (!id || expectedKind(id) !== "table") return null;
  return {
    id,
    body: body.slice(0, match.index) + (match[3] ?? ""),
  };
}

function replaceReferences(
  html: string,
  options: ResolvedCrossReferencesOptions,
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
): string {
  return transformText(html, (text) => {
    return transformTextOutsideCitationGroups(text, (segment) =>
      replaceReferenceSegment(segment, options, targets, diagnostics),
    );
  });
}

function replaceReferenceSegment(
  text: string,
  options: ResolvedCrossReferencesOptions,
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
): string {
  return text.replace(TEXT_REFERENCE_RE, (full, prefix: string, id: string) => {
    const expected = expectedKind(id);
    if (!expected) return full;

    const target = targets.get(id);
    if (!target) {
      diagnostics.push({
        policy: options.missing,
        message: `missing cross-reference target "${id}"`,
      });
      return full;
    }
    if (target.kind !== expected) {
      diagnostics.push({
        policy: options.mismatches,
        message: `cross-reference "${id}" expects ${expected} but found ${target.kind}`,
      });
      return full;
    }

    return `${prefix}<a class="ox-xref ox-xref-${target.kind}" href="${target.href}" data-ox-xref-id="${escapeAttr(target.id)}" data-ox-xref-kind="${target.kind}">${escapeHtml(target.text)}</a>`;
  });
}

function registerTarget(
  targets: Map<string, CrossReferenceTarget>,
  diagnostics: CrossReferenceDiagnostic[],
  policy: CrossReferenceFailureMode,
  target: CrossReferenceTarget,
): void {
  if (targets.has(target.id)) {
    diagnostics.push({ policy, message: `duplicate cross-reference target "${target.id}"` });
    return;
  }
  targets.set(target.id, target);
}

function flushDiagnostics(diagnostics: CrossReferenceDiagnostic[]): void {
  const errors = diagnostics.filter((diagnostic) => diagnostic.policy === "error");
  if (errors.length > 0) {
    throw new Error(`[ox-content] Cross-reference diagnostics:\n${formatDiagnostics(errors)}`);
  }
  const warnings = diagnostics.filter((diagnostic) => diagnostic.policy === "warn");
  if (warnings.length > 0) {
    console.warn(`[ox-content] Cross-reference diagnostics:\n${formatDiagnostics(warnings)}`);
  }
  diagnostics.length = 0;
}

function formatDiagnostics(diagnostics: CrossReferenceDiagnostic[]): string {
  return diagnostics.map((diagnostic) => `- ${diagnostic.message}`).join("\n");
}
