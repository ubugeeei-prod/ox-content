import type { OxContentOptions, ResolvedOptions } from "./types";
import { importNapiModuleSync } from "./napi";
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

  // Numbering, annotation, and reference rewriting all happen in one native
  // call. Rust returns diagnostics rather than raising, because which of them
  // are fatal is this layer's policy, not the transform's.
  const napi = importNapiModuleSync() as unknown as NativeCrossReferenceModule;
  const output = napi.transformCrossReferences(html, {
    enabled: true,
    missing: options.missing,
    duplicates: options.duplicates,
    mismatches: options.mismatches,
    labels: options.labels,
  });
  flushDiagnostics(output.diagnostics);

  return { html: output.html, references: output.references };
}

interface NativeCrossReferenceModule {
  transformCrossReferences(
    html: string,
    options: {
      enabled: boolean;
      missing: CrossReferenceFailureMode;
      duplicates: CrossReferenceFailureMode;
      mismatches: CrossReferenceFailureMode;
      labels: { figure: string; table: string; section: string };
    },
  ): { html: string; references: CrossReferenceEntry[]; diagnostics: CrossReferenceDiagnostic[] };
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
