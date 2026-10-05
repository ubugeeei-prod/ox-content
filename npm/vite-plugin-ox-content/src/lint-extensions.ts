import type { MarkdownLintResult } from "./lint";

/** An edit whose start/end are UTF-8 byte offsets into the original document. */
export interface MarkdownLintFix {
  start: number;
  end: number;
  text: string;
}

/** Textlint-inspired native prose rules. Every rule is opt-in. */
export interface MarkdownLintTextRules {
  /** Maximum Unicode code points per sentence, across soft line breaks. */
  sentenceLength?: number;
  /** Maximum Japanese commas (、) per sentence. */
  maxTen?: number;
  noExclamationQuestionMark?: boolean;
  noTodo?: boolean;
  /** Literal, case-sensitive phrases. ASCII terms respect word boundaries. */
  terminology?: { term: string; replacement: string }[];
}

export interface MarkdownLintStructureRules {
  /** @default true */
  emptyHeadings?: boolean;
  /** @default false */
  firstHeadingH1?: boolean;
  /** @default false */
  singleH1?: boolean;
  /** @default false */
  codeFenceLanguage?: boolean;
  /** @default true */
  codeFenceClosed?: boolean;
  /** @default true */
  emptyLinks?: boolean;
  /** @default false */
  imageAlt?: boolean;
  /** @default false */
  finalNewline?: boolean;
}

export interface MarkdownLintFixResult {
  output: string;
  appliedFixes: number;
  result: MarkdownLintResult;
}

export const STRUCTURE_DEFAULTS = {
  emptyHeadings: true,
  firstHeadingH1: false,
  singleH1: false,
  codeFenceLanguage: false,
  codeFenceClosed: true,
  emptyLinks: true,
  imageAlt: false,
  finalNewline: false,
} as const;

/** Native markdownlint 0.41.1 rules, aliases and tags. */
export interface MarkdownlintConfig {
  $schema?: string;
  default?: boolean;
  [ruleOrTag: string]:
    | boolean
    | string
    | { enabled?: boolean; severity?: "error" | "warning"; [parameter: string]: unknown }
    | undefined;
}
