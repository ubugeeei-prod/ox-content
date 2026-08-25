/**
 * Resolve MDX component imports relative to the document that declared them.
 *
 * Only `./` and `../` specifiers become island bindings. Bare, package, and
 * remote specifiers are reported and ignored. A specifier that leaves the
 * configured content root is rejected with a diagnostic.
 */

import fs from "node:fs";
import path from "node:path";
import type { MdxImport, MdxImportSpecifierKind } from "./types";

export interface ResolveDocumentComponentImportsInput {
  imports: readonly MdxImport[];
  documentPath: string;
  contentRoot?: string;
  srcDir?: string;
}

export interface ResolvedDocumentComponentImport {
  localName: string;
  specifier: string;
  resolvedPath: string;
  importPathRelativeToDocument: string;
  imported: string;
  kind: Exclude<MdxImportSpecifierKind, "namespace">;
}

export type DocumentImportDiagnosticCode = "not-relative" | "escapes-root" | "duplicate-binding";

export interface DocumentImportDiagnostic {
  code: DocumentImportDiagnosticCode;
  message: string;
  specifier: string;
  localName?: string;
}

export interface ResolveDocumentComponentImportsResult {
  bindings: ResolvedDocumentComponentImport[];
  diagnostics: DocumentImportDiagnostic[];
}

export function resolveContentRootPath(input: {
  contentRoot?: string;
  srcDir?: string;
  root?: string;
}): string {
  if (input.contentRoot) {
    return path.resolve(input.contentRoot);
  }
  const root = input.root ?? process.cwd();
  return path.resolve(root, input.srcDir ?? ".");
}

export function stripViteQuery(id: string): string {
  return id.split("?")[0].split("#")[0];
}

export function resolveDocumentComponentImports(
  input: ResolveDocumentComponentImportsInput,
): ResolveDocumentComponentImportsResult {
  const documentPath = stripViteQuery(input.documentPath);
  const documentDir = path.dirname(documentPath);
  const contentRoot = resolveContentRootPath(input);
  const diagnostics: DocumentImportDiagnostic[] = [];
  const candidates: ResolvedDocumentComponentImport[] = [];

  for (const statement of input.imports) {
    const specifier = statement.source;
    if (!isRelativeSpecifier(specifier)) {
      diagnostics.push({
        code: "not-relative",
        message: `Document component import "${specifier}" is not relative and was ignored.`,
        specifier,
      });
      continue;
    }

    for (const spec of statement.specifiers) {
      if (spec.kind === "namespace") {
        continue;
      }

      const resolvedPath = resolveExistingPath(path.resolve(documentDir, specifier));
      if (!isInsideRoot(resolvedPath, contentRoot)) {
        diagnostics.push({
          code: "escapes-root",
          message: `Document component import "${specifier}" escapes the content root.`,
          specifier,
          localName: spec.local,
        });
        continue;
      }

      candidates.push({
        localName: spec.local,
        specifier,
        resolvedPath,
        importPathRelativeToDocument: toDocumentRelativeImport(documentDir, resolvedPath),
        imported: spec.imported,
        kind: spec.kind,
      });
    }
  }

  const counts = new Map<string, number>();
  for (const binding of candidates) {
    counts.set(binding.localName, (counts.get(binding.localName) ?? 0) + 1);
  }

  const bindings: ResolvedDocumentComponentImport[] = [];
  const reportedDuplicates = new Set<string>();
  for (const binding of candidates) {
    if ((counts.get(binding.localName) ?? 0) > 1) {
      if (!reportedDuplicates.has(binding.localName)) {
        reportedDuplicates.add(binding.localName);
        diagnostics.push({
          code: "duplicate-binding",
          message: `Document component name "${binding.localName}" is imported more than once.`,
          specifier: binding.specifier,
          localName: binding.localName,
        });
      }
      continue;
    }
    bindings.push(binding);
  }

  return { bindings, diagnostics };
}

function isRelativeSpecifier(source: string): boolean {
  return source.startsWith("./") || source.startsWith("../");
}

function resolveExistingPath(filePath: string): string {
  try {
    return fs.realpathSync(filePath);
  } catch {
    return path.normalize(filePath);
  }
}

function isInsideRoot(resolvedPath: string, root: string): boolean {
  const relative = path.relative(resolveExistingPath(root), resolvedPath);
  return (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative))
  );
}

function toDocumentRelativeImport(documentDir: string, resolvedPath: string): string {
  const relative = path.relative(documentDir, resolvedPath).replace(/\\/g, "/");
  return relative.startsWith(".") ? relative : `./${relative}`;
}
