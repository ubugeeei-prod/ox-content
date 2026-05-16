/**
 * i18n plugin for Ox Content.
 *
 * Provides:
 * - Dictionary loading and validation at build time
 * - Virtual module for i18n config
 * - Build-time i18n checking
 * - Locale-aware routing middleware for dev server
 */

import * as path from "path";
import * as fs from "fs";
import type { Plugin, ViteDevServer } from "vite";
import { importNapiModule } from "./napi";
import type { I18nOptions, ResolvedI18nOptions, LocaleConfig, ResolvedOptions } from "./types";

/**
 * Resolves i18n options with defaults.
 */
export function resolveI18nOptions(
  options: I18nOptions | false | undefined,
): ResolvedI18nOptions | false {
  if (options === false) return false;
  if (!options || !options.enabled) {
    return false;
  }

  const defaultLocale = options.defaultLocale ?? "en";
  const locales: LocaleConfig[] = options.locales ?? [{ code: defaultLocale, name: defaultLocale }];

  // Ensure default locale is in the locales list
  if (!locales.some((l) => l.code === defaultLocale)) {
    locales.unshift({ code: defaultLocale, name: defaultLocale });
  }

  return {
    enabled: true,
    dir: options.dir ?? "content/i18n",
    defaultLocale,
    locales,
    hideDefaultLocale: options.hideDefaultLocale ?? true,
    check: options.check ?? true,
    functionNames: options.functionNames ?? ["t", "$t"],
  };
}

/**
 * Creates the i18n sub-plugin for the Vite plugin array.
 */
export function createI18nPlugin(resolvedOptions: ResolvedOptions): Plugin {
  const i18nOptions = resolvedOptions.i18n;
  let root = process.cwd();

  return {
    name: "ox-content:i18n",

    configResolved(config) {
      root = config.root;
    },

    resolveId(id) {
      if (id === "virtual:ox-content/i18n") {
        return "\0virtual:ox-content/i18n";
      }
      return null;
    },

    load(id) {
      if (id === "\0virtual:ox-content/i18n") {
        if (!i18nOptions) {
          return `export const i18n = { enabled: false }; export default i18n;`;
        }

        return generateI18nModule(i18nOptions, root);
      }
      return null;
    },

    async buildStart() {
      if (!i18nOptions || !i18nOptions.check) return;

      const dictDir = path.resolve(root, i18nOptions.dir);
      if (!fs.existsSync(dictDir)) {
        console.warn(`[ox-content:i18n] Dictionary directory not found: ${dictDir}`);
        return;
      }

      try {
        const { loadDictionaries, checkI18n, extractTranslationKeys } = await importNapiModule();

        // Load and validate dictionaries
        const loadResult = loadDictionaries(dictDir);
        if (loadResult.errors.length > 0) {
          for (const error of loadResult.errors) {
            console.warn(`[ox-content:i18n] ${error}`);
          }
          return;
        }

        console.log(
          `[ox-content:i18n] Loaded ${loadResult.localeCount} locales: ${loadResult.locales.join(", ")}`,
        );

        // Collect translation keys from source files
        const collectedKeys = collectKeysFromSource(root, extractTranslationKeys, i18nOptions);

        const checkResult = checkI18n(dictDir, collectedKeys);
        if (checkResult.errorCount > 0 || checkResult.warningCount > 0) {
          for (const diag of checkResult.diagnostics) {
            if (diag.severity === "error") {
              console.error(`[ox-content:i18n] ${diag.message}`);
            } else if (diag.severity === "warning") {
              console.warn(`[ox-content:i18n] ${diag.message}`);
            }
          }
        }
      } catch {
        // NAPI binding not available; skip checks
      }
    },

    configureServer(server: ViteDevServer) {
      if (!i18nOptions) return;

      // Watch dictionary directory for changes
      const dictDir = path.resolve(root, i18nOptions.dir);
      if (fs.existsSync(dictDir)) {
        server.watcher.add(dictDir);

        server.watcher.on("change", (filePath: string) => {
          if (!filePath.startsWith(dictDir)) return;
          if (!/\.(json|yaml|yml)$/.test(filePath)) return;

          // Invalidate the virtual module
          const mod = server.moduleGraph.getModuleById("\0virtual:ox-content/i18n");
          if (mod) {
            server.moduleGraph.invalidateModule(mod);
          }

          // Trigger full reload
          server.ws.send({ type: "full-reload" });
        });
      }

      // Add locale routing middleware
      server.middlewares.use((req, _res, next) => {
        if (!req.url) return next();

        // Parse locale from URL
        const url = req.url;
        const localeMatch = url.match(/^\/([A-Za-z]{2,3}(?:-[A-Za-z0-9]+)*)(\/|$)/);

        if (localeMatch) {
          const localeCode = localeMatch[1];
          const isKnown = i18nOptions.locales.some((l) => l.code === localeCode);
          if (isKnown) {
            // Set locale header for downstream middleware
            (req as any).__oxLocale = localeCode;
          }
        } else if (i18nOptions.hideDefaultLocale) {
          // No locale prefix: use default locale
          (req as any).__oxLocale = i18nOptions.defaultLocale;
        }

        next();
      });
    },
  };
}

/**
 * Generates the virtual module for i18n configuration.
 */
export function generateI18nModule(options: ResolvedI18nOptions, root: string): string {
  const dictDir = path.resolve(root, options.dir);
  const localesJson = JSON.stringify(options.locales);
  const defaultLocale = JSON.stringify(options.defaultLocale);

  // Load dictionaries synchronously for the virtual module
  let dictionariesCode = "{}";

  // Try NAPI-based loading first (supports JSON + YAML)
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const napi = require("@ox-content/napi");
    if (napi.loadDictionariesFlat) {
      const dictData = napi.loadDictionariesFlat(dictDir);
      dictionariesCode = JSON.stringify(dictData);
    } else {
      dictionariesCode = JSON.stringify(loadDictionariesFallback(options, dictDir));
    }
  } catch {
    // NAPI not available — fallback to TS-based JSON loading
    try {
      dictionariesCode = JSON.stringify(loadDictionariesFallback(options, dictDir));
    } catch {
      // Fallback to empty dictionaries
    }
  }

  return `
export const i18nConfig = {
  enabled: true,
  defaultLocale: ${defaultLocale},
  locales: ${localesJson},
  hideDefaultLocale: ${JSON.stringify(options.hideDefaultLocale)},
};

export const dictionaries = ${dictionariesCode};

export function t(key, params, locale) {
  const dict = dictionaries[locale || i18nConfig.defaultLocale] || {};
  let message = dict[key];
  if (!message) {
    const fallback = dictionaries[i18nConfig.defaultLocale] || {};
    message = fallback[key] || key;
  }
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      message = message.split('{$' + k + '}').join(String(v));
    }
  }
  return message;
}

export function getLocaleFromPath(pathname) {
  const match = pathname.match(new RegExp('^/([A-Za-z]{2,3}(?:-[A-Za-z0-9]+)*)(/|$)'));
  if (match) {
    const code = match[1];
    if (i18nConfig.locales.some(l => l.code === code)) {
      return code;
    }
  }
  return i18nConfig.defaultLocale;
}

export function localePath(pathname, locale) {
  const current = getLocaleFromPath(pathname);
  let clean = pathname;
  if (current !== i18nConfig.defaultLocale || !i18nConfig.hideDefaultLocale) {
    const prefix = '/' + current;
    if (clean === prefix) clean = '/';
    else if (clean.startsWith(prefix + '/')) clean = clean.slice(prefix.length);
  }
  if (locale === i18nConfig.defaultLocale && i18nConfig.hideDefaultLocale) {
    return clean || '/';
  }
  return '/' + locale + (clean.startsWith('/') ? clean : '/' + clean);
}

const formatterCache = new Map();

function getFormatter(kind, locale, options) {
  const key = kind + ':' + locale + ':' + JSON.stringify(options || {});
  if (!formatterCache.has(key)) {
    formatterCache.set(key, new Intl[kind](locale, options));
  }
  return formatterCache.get(key);
}

export function getLocaleMeta(locale) {
  const code = locale || i18nConfig.defaultLocale;
  return i18nConfig.locales.find(l => l.code === code) || { code, name: code, dir: 'ltr' };
}

export function formatDate(value, options, locale) {
  return getFormatter('DateTimeFormat', locale || i18nConfig.defaultLocale, options).format(
    value instanceof Date ? value : new Date(value),
  );
}

export function formatDateParts(value, options, locale) {
  return getFormatter('DateTimeFormat', locale || i18nConfig.defaultLocale, options).formatToParts(
    value instanceof Date ? value : new Date(value),
  );
}

export function formatNumber(value, options, locale) {
  return getFormatter('NumberFormat', locale || i18nConfig.defaultLocale, options).format(value);
}

export function formatNumberParts(value, options, locale) {
  return getFormatter('NumberFormat', locale || i18nConfig.defaultLocale, options).formatToParts(value);
}

export function formatRelativeTime(value, unit, options, locale) {
  return getFormatter('RelativeTimeFormat', locale || i18nConfig.defaultLocale, options).format(value, unit);
}

export function formatList(values, options, locale) {
  return getFormatter('ListFormat', locale || i18nConfig.defaultLocale, options).format(values);
}

export function formatListParts(values, options, locale) {
  return getFormatter('ListFormat', locale || i18nConfig.defaultLocale, options).formatToParts(values);
}

export function formatDisplayName(value, type, options, locale) {
  if (!Intl.DisplayNames) return String(value);
  const displayType = type || 'language';
  return getFormatter('DisplayNames', locale || i18nConfig.defaultLocale, { type: displayType, ...options }).of(value) || String(value);
}

export function createIntl(locale, defaults = {}) {
  const meta = getLocaleMeta(locale);
  const code = meta.code;
  return {
    locale: code,
    meta,
    dir: meta.dir || 'ltr',
    date: (value, options) => formatDate(value, { ...defaults.date, ...options }, code),
    dateParts: (value, options) => formatDateParts(value, { ...defaults.date, ...options }, code),
    number: (value, options) => formatNumber(value, { ...defaults.number, ...options }, code),
    numberParts: (value, options) => formatNumberParts(value, { ...defaults.number, ...options }, code),
    relativeTime: (value, unit, options) => formatRelativeTime(value, unit, { ...defaults.relativeTime, ...options }, code),
    list: (values, options) => formatList(values, { ...defaults.list, ...options }, code),
    listParts: (values, options) => formatListParts(values, { ...defaults.list, ...options }, code),
    displayName: (value, type, options) => formatDisplayName(value, type, { ...defaults.displayName, ...options }, code),
  };
}

export default {
  i18nConfig,
  dictionaries,
  t,
  getLocaleFromPath,
  localePath,
  getLocaleMeta,
  createIntl,
  formatDate,
  formatDateParts,
  formatNumber,
  formatNumberParts,
  formatRelativeTime,
  formatList,
  formatListParts,
  formatDisplayName,
};
  `;
}

/**
 * Flattens a nested object into dot-separated keys.
 */
function flattenObject(
  obj: Record<string, unknown>,
  prefix: string,
  result: Record<string, string>,
): void {
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = `${prefix}.${key}`;
    if (typeof value === "string") {
      result[fullKey] = value;
    } else if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      flattenObject(value as Record<string, unknown>, fullKey, result);
    } else {
      result[fullKey] = String(value);
    }
  }
}

/**
 * Fallback dictionary loading using TS-based JSON file reading.
 */
function loadDictionariesFallback(
  options: ResolvedI18nOptions,
  dictDir: string,
): Record<string, Record<string, string>> {
  const dictData: Record<string, Record<string, string>> = {};

  for (const locale of options.locales) {
    const localeDir = path.join(dictDir, locale.code);
    if (!fs.existsSync(localeDir)) continue;

    const files = fs.readdirSync(localeDir);
    const localeDict: Record<string, string> = {};

    for (const file of files) {
      if (!file.endsWith(".json")) continue;
      const filePath = path.join(localeDir, file);
      const content = fs.readFileSync(filePath, "utf-8");
      const namespace = path.basename(file, ".json");

      try {
        const data = JSON.parse(content);
        flattenObject(data, namespace, localeDict);
      } catch {
        // Skip invalid JSON files
      }
    }

    dictData[locale.code] = localeDict;
  }

  return dictData;
}

/**
 * Collects translation keys from source files using NAPI extractTranslationKeys.
 */
function collectKeysFromSource(
  root: string,
  extractTranslationKeys: (
    source: string,
    filePath: string,
    functionNames?: string[],
  ) => Array<{ key: string }>,
  options: ResolvedI18nOptions,
): string[] {
  const srcDir = path.resolve(root, "src");
  const keys = new Set<string>();

  // Scan TS/JS/TSX/JSX files in src/
  if (fs.existsSync(srcDir)) {
    walkDir(srcDir, /\.(ts|tsx|js|jsx)$/, (filePath) => {
      const source = fs.readFileSync(filePath, "utf-8");
      const usages = extractTranslationKeys(source, filePath, options.functionNames);
      for (const usage of usages) {
        keys.add(usage.key);
      }
    });
  }

  // Scan Markdown files for {{t('key')}} patterns
  const contentDir = path.resolve(root, "content");
  if (fs.existsSync(contentDir)) {
    const tPattern = /\{\{t\(['"]([^'"]+)['"]\)\}\}/g;
    walkDir(contentDir, /\.(md|mdx)$/, (filePath) => {
      const content = fs.readFileSync(filePath, "utf-8");
      let match;
      while ((match = tPattern.exec(content)) !== null) {
        keys.add(match[1]);
      }
      tPattern.lastIndex = 0;
    });
  }

  return Array.from(keys);
}

/**
 * Recursively walks a directory, calling the callback for files matching the pattern.
 */
function walkDir(dir: string, pattern: RegExp, callback: (filePath: string) => void): void {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === ".git") continue;
      walkDir(fullPath, pattern, callback);
    } else if (pattern.test(entry.name)) {
      callback(fullPath);
    }
  }
}
