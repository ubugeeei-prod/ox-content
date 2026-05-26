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
        const { checkI18nProject } = await importNapiModule();
        const checkResult = checkI18nProject(
          dictDir,
          [path.resolve(root, "src"), path.resolve(root, "content")],
          i18nOptions.functionNames,
          i18nOptions.defaultLocale,
        );
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
  const config = {
    defaultLocale: options.defaultLocale,
    locales: options.locales,
    hideDefaultLocale: options.hideDefaultLocale,
  };

  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const napi = require("@ox-content/napi") as {
      generateI18nModule?: (dictDir: string, runtimeConfig: typeof config) => string;
    };

    if (typeof napi.generateI18nModule === "function") {
      return napi.generateI18nModule(dictDir, config);
    }
  } catch (error) {
    throw new Error(
      `[ox-content:i18n] Failed to load @ox-content/napi for i18n module generation: ${String(error)}`,
    );
  }

  throw new Error(
    "[ox-content:i18n] @ox-content/napi does not expose generateI18nModule. Please rebuild the NAPI package.",
  );
}
