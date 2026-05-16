import { createRequire } from "node:module";
import type { CSpellUserSettings, ValidationIssue } from "cspell-lib";

const require = createRequire(import.meta.url);

const SUPPORTED_MARKDOWN_LINT_LANGUAGES = ["en", "ja", "zh", "fr", "de", "pl"] as const;
const DEFAULT_LANGUAGES = ["en"] as const;
const DEFAULT_RULES = {
  duplicateHeadings: true,
  headingIncrement: true,
  maxConsecutiveBlankLines: 1,
  repeatedPunctuation: true,
  repeatedWords: true,
  spellcheck: true,
  trailingSpaces: true,
} as const;
const DEFAULT_CSPELL_IMPORTS = {
  de: "@cspell/dict-de-de/cspell-ext.json",
  en: "@cspell/dict-en_us/cspell-ext.json",
  fr: "@cspell/dict-fr-fr/cspell-ext.json",
  pl: "@cspell/dict-pl_pl/cspell-ext.json",
} as const satisfies Partial<Record<MarkdownLintLanguage, string>>;

export type MarkdownLintLanguage = (typeof SUPPORTED_MARKDOWN_LINT_LANGUAGES)[number];
export type MarkdownLintSeverity = "error" | "warning" | "info";

/**
 * Opt-in standard dictionary sources.
 *
 * The default provider uses CSpell dictionary packages because those packages
 * are actively maintained and expose locale-specific dictionaries in a stable
 * config format. Languages without a bundled preset can still be added through
 * custom `imports`.
 */
export interface MarkdownLintStandardDictionaryOptions {
  /**
   * Standard dictionary provider.
   * @default "cspell"
   */
  provider?: "cspell";

  /**
   * Languages whose default standard dictionaries should be enabled.
   *
   * Built-in preset package mappings currently exist for `en`, `fr`, `de`,
   * and `pl`. For other languages, use `imports`.
   *
   * @default []
   */
  languages?: MarkdownLintLanguage[];

  /**
   * Additional CSpell-compatible imports.
   *
   * This can point at installed packages like
   * `@cspell/dict-fr-fr/cspell-ext.json` or local CSpell config files.
   */
  imports?: string[];

  /**
   * Base URL or path used when resolving `imports`.
   *
   * @default new URL(".", import.meta.url)
   */
  resolveImportsRelativeTo?: string | URL;
}

/**
 * Additional dictionary configuration for the Markdown linter.
 */
export interface MarkdownLintDictionaryOptions {
  /**
   * Words ignored across all configured languages.
   */
  words?: string[];

  /**
   * Extra words to allow per language.
   */
  byLanguage?: Partial<Record<MarkdownLintLanguage, string[]>>;

  /**
   * Words that should never produce diagnostics.
   */
  ignoredWords?: string[];

  /**
   * Opt-in standard dictionary datasets.
   *
   * By default the linter stays on a minimal built-in dictionary. Enable this
   * to load larger locale dictionaries from a standard external source.
   */
  standard?: MarkdownLintStandardDictionaryOptions | false;
}

/**
 * Rule switches for Markdown linting.
 */
export interface MarkdownLintRuleOptions {
  /**
   * Report headings that repeat the same visible text.
   * @default true
   */
  duplicateHeadings?: boolean;

  /**
   * Report heading depth jumps such as `#` -> `###`.
   * @default true
   */
  headingIncrement?: boolean;

  /**
   * Maximum number of blank lines allowed in a row.
   * @default 1
   */
  maxConsecutiveBlankLines?: number;

  /**
   * Report duplicated terminal punctuation such as `!!` or `？？`.
   * @default true
   */
  repeatedPunctuation?: boolean;

  /**
   * Report adjacent repeated words in visible prose.
   * @default true
   */
  repeatedWords?: boolean;

  /**
   * Enable built-in multilingual spellchecking.
   * @default true
   */
  spellcheck?: boolean;

  /**
   * Report trailing spaces.
   * @default true
   */
  trailingSpaces?: boolean;
}

/**
 * Options for linting Markdown documents.
 */
export interface MarkdownLintOptions {
  /**
   * Languages enabled for spellchecking.
   *
   * When `dictionary.standard.languages` is provided and this option is
   * omitted, those languages are used instead.
   *
   * @default ['en']
   */
  languages?: MarkdownLintLanguage[];

  /**
   * Rule configuration.
   */
  rules?: MarkdownLintRuleOptions;

  /**
   * Built-in and opt-in standard dictionary overrides.
   */
  dictionary?: MarkdownLintDictionaryOptions;
}

/**
 * A single Markdown lint diagnostic.
 */
export interface MarkdownLintDiagnostic {
  /**
   * Stable rule identifier.
   */
  ruleId: string;

  /**
   * Diagnostic severity.
   */
  severity: MarkdownLintSeverity;

  /**
   * Human-readable explanation.
   */
  message: string;

  /**
   * 1-indexed line number.
   */
  line: number;

  /**
   * 1-indexed start column.
   */
  column: number;

  /**
   * 1-indexed end line.
   */
  endLine: number;

  /**
   * 1-indexed end column.
   */
  endColumn: number;

  /**
   * Language used for spellchecking, when relevant.
   */
  language?: MarkdownLintLanguage;

  /**
   * Suggested replacements, when available.
   */
  suggestions?: string[];
}

/**
 * Markdown lint report.
 */
export interface MarkdownLintResult {
  /**
   * All collected diagnostics.
   */
  diagnostics: MarkdownLintDiagnostic[];

  /**
   * Number of error diagnostics.
   */
  errorCount: number;

  /**
   * Number of warning diagnostics.
   */
  warningCount: number;

  /**
   * Number of info diagnostics.
   */
  infoCount: number;
}

interface NormalizedStandardDictionaryOptions {
  imports: string[];
  languages: MarkdownLintLanguage[];
  provider: "cspell";
  resolveImportsRelativeTo: string | URL;
}

interface InternalNormalizedMarkdownLintOptions {
  dictionary: Omit<MarkdownLintDictionaryOptions, "standard"> & {
    standard: NormalizedStandardDictionaryOptions | false;
  };
  languages: MarkdownLintLanguage[];
  rules: Required<MarkdownLintRuleOptions>;
}

interface NapiMarkdownLintLanguageWords {
  language: MarkdownLintLanguage;
  words: string[];
}

interface NapiMarkdownLintOptions {
  dictionary?: {
    byLanguage?: NapiMarkdownLintLanguageWords[];
    ignoredWords?: string[];
    words?: string[];
  };
  languages?: MarkdownLintLanguage[];
  rules?: Required<MarkdownLintRuleOptions>;
}

interface NapiMarkdownLintResult extends MarkdownLintResult {
  maskedDocument: string;
}

interface NapiMarkdownLintModule {
  lintMarkdownDocuments?: (
    sources: string[],
    options?: NapiMarkdownLintOptions,
  ) => NapiMarkdownLintResult[];
  lintMarkdown: (source: string, options?: NapiMarkdownLintOptions) => NapiMarkdownLintResult;
}

let napiBinding: NapiMarkdownLintModule | null | undefined;
let cspellLibPromise: Promise<typeof import("cspell-lib")> | undefined;

/**
 * Lints Markdown prose with the Rust-backed built-in rule engine.
 */
export function lintMarkdown(
  source: string,
  options: MarkdownLintOptions = {},
): MarkdownLintResult {
  const normalizedOptions = normalizeLintOptions(options);
  return lintMarkdownWithNormalizedOptions(source, normalizedOptions);
}

/**
 * Async Markdown linter that supports opt-in standard dictionaries.
 */
export async function lintMarkdownAsync(
  source: string,
  options: MarkdownLintOptions = {},
): Promise<MarkdownLintResult> {
  const normalizedOptions = normalizeLintOptions(options);
  const [result] = await lintMarkdownDocumentsWithNormalizedOptions([source], normalizedOptions);
  return result ?? createEmptyLintResult();
}

/**
 * Internal batched Markdown linting entry point used by file-based workflows.
 */
export async function lintMarkdownDocumentsAsync(
  sources: string[],
  options: MarkdownLintOptions = {},
): Promise<MarkdownLintResult[]> {
  const normalizedOptions = normalizeLintOptions(options);
  return lintMarkdownDocumentsWithNormalizedOptions(sources, normalizedOptions);
}

function lintMarkdownWithNormalizedOptions(
  source: string,
  normalizedOptions: InternalNormalizedMarkdownLintOptions,
): MarkdownLintResult {
  if (normalizedOptions.dictionary.standard) {
    throw new Error(
      "[ox-content] lintMarkdownAsync is required when dictionary.standard is enabled.",
    );
  }

  const napi = loadNapiBindingSync();
  return stripMaskedDocument(
    napi.lintMarkdown(source, toNapiMarkdownLintOptions(normalizedOptions)),
  );
}

async function lintMarkdownDocumentsWithNormalizedOptions(
  sources: string[],
  normalizedOptions: InternalNormalizedMarkdownLintOptions,
): Promise<MarkdownLintResult[]> {
  if (sources.length === 0) {
    return [];
  }

  const napi = loadNapiBindingSync();
  const napiOptions = toNapiMarkdownLintOptions(
    normalizedOptions,
    Boolean(normalizedOptions.dictionary.standard),
  );
  const builtInResults =
    typeof napi.lintMarkdownDocuments === "function"
      ? napi.lintMarkdownDocuments(sources, napiOptions)
      : sources.map((source) => napi.lintMarkdown(source, napiOptions));

  if (!normalizedOptions.rules.spellcheck || !normalizedOptions.dictionary.standard) {
    return builtInResults.map(stripMaskedDocument);
  }

  const standardDiagnostics = await runStandardSpellcheckDocuments(
    builtInResults.map((result) => result.maskedDocument),
    normalizedOptions,
  );

  return builtInResults.map((result, index) =>
    summarizeDiagnostics(
      sortDiagnostics(result.diagnostics.concat(standardDiagnostics[index] ?? [])),
    ),
  );
}

function loadNapiBindingSync(): NapiMarkdownLintModule {
  if (napiBinding) {
    return napiBinding;
  }

  if (napiBinding === null) {
    throw new Error(
      "[ox-content] @ox-content/napi is required for Markdown linting. Please ensure the NAPI module is built.",
    );
  }

  try {
    const loaded = require("@ox-content/napi") as NapiMarkdownLintModule & {
      default?: Partial<NapiMarkdownLintModule>;
    };
    napiBinding =
      loaded.default && typeof loaded.default === "object"
        ? { ...loaded.default, ...loaded }
        : loaded;

    return napiBinding;
  } catch {
    napiBinding = null;
    throw new Error(
      "[ox-content] @ox-content/napi is required for Markdown linting. Please ensure the NAPI module is built.",
    );
  }
}

function toNapiMarkdownLintOptions(
  options: InternalNormalizedMarkdownLintOptions,
  disableBuiltinSpellcheck = false,
): NapiMarkdownLintOptions {
  const byLanguage = Object.entries(options.dictionary.byLanguage ?? {}).map(
    ([language, words]): NapiMarkdownLintLanguageWords => ({
      language: language as MarkdownLintLanguage,
      words,
    }),
  );

  return {
    dictionary: {
      byLanguage,
      ignoredWords: options.dictionary.ignoredWords,
      words: options.dictionary.words,
    },
    languages: options.languages,
    rules: {
      ...options.rules,
      spellcheck: disableBuiltinSpellcheck ? false : options.rules.spellcheck,
    },
  };
}

function stripMaskedDocument(result: NapiMarkdownLintResult): MarkdownLintResult {
  return {
    diagnostics: result.diagnostics,
    errorCount: result.errorCount,
    infoCount: result.infoCount,
    warningCount: result.warningCount,
  };
}

function normalizeLintOptions(options: MarkdownLintOptions): InternalNormalizedMarkdownLintOptions {
  const languages = options.languages?.filter((language): language is MarkdownLintLanguage =>
    SUPPORTED_MARKDOWN_LINT_LANGUAGES.includes(language),
  ) ??
    options.dictionary?.standard?.languages?.filter((language): language is MarkdownLintLanguage =>
      SUPPORTED_MARKDOWN_LINT_LANGUAGES.includes(language),
    ) ?? [...DEFAULT_LANGUAGES];

  const standard = normalizeStandardDictionaryOptions(options.dictionary?.standard, languages);

  return {
    dictionary: {
      ...options.dictionary,
      standard,
    },
    languages: [...new Set(languages)],
    rules: {
      duplicateHeadings: options.rules?.duplicateHeadings ?? DEFAULT_RULES.duplicateHeadings,
      headingIncrement: options.rules?.headingIncrement ?? DEFAULT_RULES.headingIncrement,
      maxConsecutiveBlankLines:
        options.rules?.maxConsecutiveBlankLines ?? DEFAULT_RULES.maxConsecutiveBlankLines,
      repeatedPunctuation: options.rules?.repeatedPunctuation ?? DEFAULT_RULES.repeatedPunctuation,
      repeatedWords: options.rules?.repeatedWords ?? DEFAULT_RULES.repeatedWords,
      spellcheck: options.rules?.spellcheck ?? DEFAULT_RULES.spellcheck,
      trailingSpaces: options.rules?.trailingSpaces ?? DEFAULT_RULES.trailingSpaces,
    },
  };
}

function normalizeStandardDictionaryOptions(
  standard: MarkdownLintDictionaryOptions["standard"],
  fallbackLanguages: MarkdownLintLanguage[],
): NormalizedStandardDictionaryOptions | false {
  if (!standard) {
    return false;
  }

  const languages =
    standard.languages?.filter((language): language is MarkdownLintLanguage =>
      SUPPORTED_MARKDOWN_LINT_LANGUAGES.includes(language),
    ) ?? fallbackLanguages;
  const customImports = standard.imports ?? [];
  const missingPresetLanguages = languages.filter((language) => !DEFAULT_CSPELL_IMPORTS[language]);

  if (missingPresetLanguages.length > 0 && customImports.length === 0) {
    throw new Error(
      `[ox-content] No bundled standard dictionary preset exists for ${missingPresetLanguages.join(
        ", ",
      )}. Provide dictionary.standard.imports to enable those languages.`,
    );
  }

  const imports = [
    ...languages
      .map((language) => DEFAULT_CSPELL_IMPORTS[language])
      .filter((value): value is string => Boolean(value)),
    ...customImports,
  ];

  if (imports.length === 0) {
    throw new Error(
      "[ox-content] dictionary.standard requires at least one bundled preset language or custom import.",
    );
  }

  return {
    imports: [...new Set(imports)],
    languages: [...new Set(languages)],
    provider: standard.provider ?? "cspell",
    resolveImportsRelativeTo: standard.resolveImportsRelativeTo ?? new URL(".", import.meta.url),
  };
}

async function runStandardSpellcheckDocuments(
  maskedDocuments: string[],
  options: InternalNormalizedMarkdownLintOptions,
): Promise<MarkdownLintDiagnostic[][]> {
  const standard = options.dictionary.standard;

  if (!standard || maskedDocuments.length === 0) {
    return maskedDocuments.map(() => []);
  }

  try {
    const { spellCheckDocument } = await loadCspellLib();
    const locale = standard.languages.join(",");
    const settings = createStandardSpellcheckSettings(options, locale);

    return Promise.all(
      maskedDocuments.map(async (maskedDocument, index) => {
        if (maskedDocument.trim().length === 0) {
          return [];
        }

        const result = await spellCheckDocument(
          {
            languageId: "plaintext",
            locale,
            text: maskedDocument,
            uri: `file:///ox-content-lint-${index}.md`,
          },
          {
            generateSuggestions: true,
            noConfigSearch: true,
            numSuggestions: 3,
            resolveImportsRelativeTo: standard.resolveImportsRelativeTo,
          },
          settings,
        );

        return result.issues.map((issue) =>
          mapStandardIssueToDiagnostic(issue, standard.languages),
        );
      }),
    );
  } catch (error) {
    const imports = standard.imports.join(", ");
    const message =
      imports.length > 0
        ? `[ox-content] Failed to load standard dictionaries from ${imports}. Verify the imports and install the referenced CSpell packages.`
        : "[ox-content] Failed to load the configured standard dictionaries.";

    throw new Error(message, {
      cause: error,
    });
  }
}

function createStandardSpellcheckSettings(
  options: InternalNormalizedMarkdownLintOptions,
  locale: string,
): CSpellUserSettings {
  return {
    import: options.dictionary.standard ? options.dictionary.standard.imports : [],
    ignoreWords: options.dictionary.ignoredWords,
    language: locale,
    version: "0.2",
    words: [
      ...(options.dictionary.words ?? []),
      ...Object.values(options.dictionary.byLanguage ?? {}).flat(),
    ],
  };
}

async function loadCspellLib(): Promise<typeof import("cspell-lib")> {
  cspellLibPromise ??= import("cspell-lib");
  return cspellLibPromise;
}

function mapStandardIssueToDiagnostic(
  issue: ValidationIssue,
  languages: MarkdownLintLanguage[],
): MarkdownLintDiagnostic {
  const line = issue.line.position.line + 1;
  const column = issue.offset - issue.line.offset + 1;
  const length = issue.length ?? issue.text.length;

  return {
    column,
    endColumn: column + length,
    endLine: line,
    language: inferStandardIssueLanguage(issue.text, languages),
    line,
    message: `Unknown word "${issue.text}".`,
    ruleId: "spellcheck",
    severity: "warning",
    suggestions: issue.suggestions?.slice(0, 3),
  };
}

function inferStandardIssueLanguage(
  word: string,
  languages: MarkdownLintLanguage[],
): MarkdownLintLanguage | undefined {
  if (/[\p{Script=Hiragana}\p{Script=Katakana}]/u.test(word) && languages.includes("ja")) {
    return "ja";
  }

  if (/[\p{Script=Han}]/u.test(word)) {
    if (languages.includes("zh") && !languages.includes("ja")) {
      return "zh";
    }
    if (languages.includes("ja") && !languages.includes("zh")) {
      return "ja";
    }
  }

  if (/[\p{Script=Latin}]/u.test(word)) {
    const latinLanguages = languages.filter(
      (language): language is Exclude<MarkdownLintLanguage, "ja" | "zh"> =>
        language !== "ja" && language !== "zh",
    );

    if (latinLanguages.length === 1) {
      return latinLanguages[0];
    }

    return inferLatinLanguageFromCharacters(word, latinLanguages);
  }

  return undefined;
}

function inferLatinLanguageFromCharacters(
  word: string,
  languages: Exclude<MarkdownLintLanguage, "ja" | "zh">[],
): Exclude<MarkdownLintLanguage, "ja" | "zh"> | undefined {
  if (languages.includes("pl") && /[ąćęłńóśźż]/iu.test(word)) {
    return "pl";
  }

  if (languages.includes("de") && /[äöüß]/iu.test(word)) {
    return "de";
  }

  if (languages.includes("fr") && /[àâæçéèêëîïôœùûüÿ]/iu.test(word)) {
    return "fr";
  }

  return undefined;
}

function summarizeDiagnostics(diagnostics: MarkdownLintDiagnostic[]): MarkdownLintResult {
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;

  for (const diagnostic of diagnostics) {
    if (diagnostic.severity === "error") {
      errorCount += 1;
    } else if (diagnostic.severity === "warning") {
      warningCount += 1;
    } else {
      infoCount += 1;
    }
  }

  return { diagnostics, errorCount, infoCount, warningCount };
}

function createEmptyLintResult(): MarkdownLintResult {
  return summarizeDiagnostics([]);
}

function sortDiagnostics(diagnostics: MarkdownLintDiagnostic[]): MarkdownLintDiagnostic[] {
  return [...diagnostics].sort((left, right) => {
    if (left.line !== right.line) {
      return left.line - right.line;
    }

    if (left.column !== right.column) {
      return left.column - right.column;
    }

    return left.ruleId.localeCompare(right.ruleId);
  });
}
