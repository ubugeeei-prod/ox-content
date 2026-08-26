export type CitationFailureMode = "error" | "warn";

export interface CitationsOptions {
  enabled?: boolean;
  bibliography?: string | string[];
  rootDir?: string;
  appendBibliography?: boolean;
  missing?: CitationFailureMode;
  duplicates?: CitationFailureMode;
  malformed?: CitationFailureMode;
  bibliographyTitle?: string;
}

export interface ResolvedCitationsOptions {
  enabled: boolean;
  bibliography: string[];
  rootDir?: string;
  appendBibliography: boolean;
  missing: CitationFailureMode;
  duplicates: CitationFailureMode;
  malformed: CitationFailureMode;
  bibliographyTitle: string;
}

export interface CitationReference {
  id: string;
  key: string;
  index: number;
  label: string;
  href: string;
  bibliographyId: string;
  suppressAuthor: boolean;
}

export interface BibliographyEntry {
  key: string;
  id: string;
  index: number;
  label: string;
  title: string;
  authors: string[];
  year?: string;
  url?: string;
  doi?: string;
  html: string;
}
