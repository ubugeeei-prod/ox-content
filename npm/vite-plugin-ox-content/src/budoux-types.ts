export type BudouxLanguage = "ja" | "zh-hans" | "zh-hant" | "th";

export interface BudouxParser {
  parse(text: string): string[];
}

export interface BudouxOptions {
  /**
   * Enable build-time BudouX segmentation when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;

  /**
   * Default BudouX parser language.
   *
   * @default "ja"
   */
  language?: BudouxLanguage;

  /**
   * Separator inserted between BudouX phrases.
   *
   * @default "\u200b"
   */
  separator?: string;

  /**
   * Custom build-time parser. When supplied, ox-content does not import the
   * `budoux` package.
   */
  parser?: BudouxParser;
}

export interface ResolvedBudouxOptions {
  enabled: boolean;
  language: BudouxLanguage;
  separator: string;
  parser?: BudouxParser;
}
