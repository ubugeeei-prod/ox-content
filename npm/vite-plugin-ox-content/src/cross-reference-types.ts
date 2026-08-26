export type CrossReferenceKind = "figure" | "table" | "section";
export type CrossReferenceFailureMode = "error" | "warn";

export interface CrossReferenceLabelOptions {
  figure?: string;
  table?: string;
  section?: string;
}

export interface CrossReferencesOptions {
  enabled?: boolean;
  missing?: CrossReferenceFailureMode;
  duplicates?: CrossReferenceFailureMode;
  mismatches?: CrossReferenceFailureMode;
  labels?: CrossReferenceLabelOptions;
}

export interface ResolvedCrossReferencesOptions {
  enabled: boolean;
  missing: CrossReferenceFailureMode;
  duplicates: CrossReferenceFailureMode;
  mismatches: CrossReferenceFailureMode;
  labels: Required<CrossReferenceLabelOptions>;
}

export interface CrossReferenceEntry {
  id: string;
  kind: CrossReferenceKind;
  number: string;
  label: string;
  text: string;
  href: string;
  title?: string;
}
