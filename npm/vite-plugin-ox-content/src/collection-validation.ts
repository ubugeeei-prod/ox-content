import * as path from "node:path";
import type {
  CollectionEntry,
  CollectionManifest,
  CollectionValidationContext,
  CollectionValidationResult,
  ResolvedOptions,
} from "./types";

export interface CollectionValidationDiagnostic {
  collection: string;
  source: string;
  documentPath: string;
  path: string;
  message: string;
}

export class CollectionValidationError extends Error {
  readonly diagnostics: CollectionValidationDiagnostic[];

  constructor(diagnostics: readonly CollectionValidationDiagnostic[]) {
    super(formatCollectionValidationError(diagnostics));
    this.name = "CollectionValidationError";
    this.diagnostics = [...diagnostics];
  }
}

export async function validateCollectionManifest(
  root: string,
  options: ResolvedOptions,
  manifest: CollectionManifest,
): Promise<void> {
  const diagnostics: CollectionValidationDiagnostic[] = [];
  const srcRoot = path.resolve(root, options.srcDir);

  for (const [collectionName, entries] of Object.entries(manifest.collections)) {
    const validate = options.collections.collections[collectionName]?.validate;
    if (!validate) continue;

    for (const entry of entries) {
      const context = createValidationContext(collectionName, entry, srcRoot);
      let result: CollectionValidationResult;
      try {
        result = await validate(context);
      } catch (error) {
        result = error instanceof Error ? error.message : String(error);
      }

      for (const message of normalizeValidationMessages(result)) {
        diagnostics.push({
          collection: collectionName,
          source: entry.source,
          documentPath: context.documentPath,
          path: entry.path,
          message,
        });
      }
    }
  }

  if (diagnostics.length > 0) {
    throw new CollectionValidationError(diagnostics);
  }
}

function createValidationContext(
  collectionName: string,
  entry: CollectionEntry,
  srcRoot: string,
): CollectionValidationContext {
  const entryForValidation = {
    ...entry,
    frontmatter: { ...entry.frontmatter },
  };
  return {
    collection: collectionName,
    source: entry.source,
    documentPath: path.resolve(srcRoot, entry.source),
    path: entry.path,
    stem: entry.stem,
    frontmatter: entryForValidation.frontmatter,
    entry: entryForValidation,
  };
}

function normalizeValidationMessages(result: CollectionValidationResult): string[] {
  if (!result) {
    return [];
  }
  if (typeof result === "string") {
    return result.trim() ? [result] : [];
  }
  return result.filter((message) => message.trim());
}

function formatCollectionValidationError(
  diagnostics: readonly CollectionValidationDiagnostic[],
): string {
  const count = diagnostics.length;
  const header = `[ox-content] Collection validation failed with ${count} diagnostic${
    count === 1 ? "" : "s"
  }.`;
  return [
    header,
    ...diagnostics.map(
      (diagnostic) =>
        `- ${diagnostic.collection}: ${diagnostic.source} (${diagnostic.path}): ${diagnostic.message}`,
    ),
  ].join("\n");
}
